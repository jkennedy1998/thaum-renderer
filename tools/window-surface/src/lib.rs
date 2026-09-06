use std::{
    fs::{self, OpenOptions},
    io::Write,
    mem,
    path::PathBuf,
    sync::Arc,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use bytemuck::{Pod, Zeroable};
use thaum_renderer_domain::{
    texture_post_effect_seed_step, texture_post_effect_texel_uv_size,
    warble_post_effect_breath_phase, POST_EFFECTS_SHADER,
};
use wgpu::{
    CompositeAlphaMode, CurrentSurfaceTexture, PresentMode, SurfaceColorSpace,
    TextureFormat,
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, Force, MouseButton, MouseScrollDelta, Touch, TouchPhase, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowAttributes},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowSurfaceUpscaleMode {
    Nearest,
    Linear,
}

impl WindowSurfaceUpscaleMode {
    fn uses_linear_filter(self) -> bool {
        matches!(self, Self::Linear)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WindowSurfaceConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub clear_color: [f64; 4],
    pub performance_log_path: Option<PathBuf>,
    pub internal_render_scale: f32,
    pub upscale_mode: WindowSurfaceUpscaleMode,
}

impl Default for WindowSurfaceConfig {
    fn default() -> Self {
        Self {
            title: "thaum-renderer".to_string(),
            width: 1280,
            height: 720,
            clear_color: [0.05, 0.06, 0.08, 1.0],
            performance_log_path: None,
            internal_render_scale: 1.0,
            upscale_mode: WindowSurfaceUpscaleMode::Nearest,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct WindowSurfaceScene {
    pub quads: Vec<SurfaceQuad>,
    /// Scene-build generation: bumped by the producer whenever the scene
    /// content actually changed. `update_scene` skips vertex/atlas/palette
    /// re-upload when the revision matches what the GPU already holds, so an
    /// unchanged scene costs no per-frame upload.
    pub revision: u64,
    pub texture_breath: i32,
    pub depth_of_field_enabled: bool,
    pub motion_noise_enabled: bool,
    pub indexed_color_enabled: bool,
    pub indexed_color_palette: Vec<[u8; 4]>,
    pub depth_of_field_minimum_falloff_cells: f32,
    pub fog_span_cells: f32,
    pub fog_nearest_depth_code: u8,
    pub fog_farthest_depth_code: u8,
    pub background_color: [f32; 4],
    pub glyph_atlas: GlyphAtlasSceneData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SurfaceQuadPostEffectBus {
    pub texture_code: u8,
    pub warble_code: u8,
    pub depth_code: u8,
    pub gate_id: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceQuad {
    pub center: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],
    pub local_uv_corners: [[f32; 2]; 4],
    pub warble_uv_corners: [[f32; 2]; 4],
    pub post_effect_bus: SurfaceQuadPostEffectBus,
    /// Glyph-atlas sampling frame: [origin_u, origin_v, tile_u_size,
    /// tile_v_size]. A negative origin_u marks a pass-through quad that does
    /// not sample the atlas (sprites, debug quads).
    pub atlas_uv: [f32; 4],
}

pub const SURFACE_QUAD_NO_ATLAS: [f32; 4] = [-1.0, -1.0, 0.0, 0.0];

impl SurfaceQuad {
    fn vertices(&self) -> [SurfaceVertex; 6] {
        let half_width = self.size[0] * 0.5;
        let half_height = self.size[1] * 0.5;
        let left = self.center[0] - half_width;
        let right = self.center[0] + half_width;
        let bottom = self.center[1] - half_height;
        let top = self.center[1] + half_height;

        [
            SurfaceVertex::new(
                self,
                [left, bottom],
                self.local_uv_corners[0],
                self.warble_uv_corners[0],
            ),
            SurfaceVertex::new(
                self,
                [right, bottom],
                self.local_uv_corners[1],
                self.warble_uv_corners[1],
            ),
            SurfaceVertex::new(
                self,
                [right, top],
                self.local_uv_corners[2],
                self.warble_uv_corners[2],
            ),
            SurfaceVertex::new(
                self,
                [left, bottom],
                self.local_uv_corners[0],
                self.warble_uv_corners[0],
            ),
            SurfaceVertex::new(
                self,
                [right, top],
                self.local_uv_corners[2],
                self.warble_uv_corners[2],
            ),
            SurfaceVertex::new(
                self,
                [left, top],
                self.local_uv_corners[3],
                self.warble_uv_corners[3],
            ),
        ]
    }
}

/// One thresholded glyph tile for the GPU atlas. Tiles are uniform-sized
/// (12×16 for the thaum typegrid) and laid out row-major by the consumer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlyphAtlasTile {
    pub glyph: u32,
    pub weight_index: u32,
    pub alpha: Arc<Vec<u8>>,
}

/// Glyph atlas carried by the scene. Consumers rasterize on CPU once per
/// distinct tile and the surface uploads tiles only when the key set changes.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GlyphAtlasSceneData {
    pub tile_width: u32,
    pub tile_height: u32,
    pub columns: u32,
    pub rows: u32,
    pub tiles: Vec<GlyphAtlasTile>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct WindowSurfaceInput {
    pub pressed_keys: Vec<KeyCode>,
    pub just_pressed_keys: Vec<KeyCode>,
    /// Cursor position in clip space (-1..1, x right, y up, origin at the
    /// window center), matching the same space `SurfaceQuad::center` renders
    /// into. `None` when the cursor has never moved into the window or has
    /// left it.
    pub cursor_position: Option<[f32; 2]>,
    /// Clip-space position of a primary (left) mouse button press that
    /// happened this frame. `None` on every frame without a new click.
    pub just_clicked: Option<[f32; 2]>,
    /// Whether the primary (left) mouse button is currently held down,
    /// persisting across frames for as long as it stays pressed — a
    /// consumer combines this with `cursor_position` to drive a drag
    /// session (e.g. a module's move/resize gizmo) after the initial click.
    pub pointer_down: bool,
    /// Clip-space position of a secondary (right) mouse button press that
    /// happened this frame. `None` on every frame without a new right click.
    pub just_right_clicked: Option<[f32; 2]>,
    /// Whether the secondary (right) mouse button is currently held down.
    pub right_pointer_down: bool,
    /// Pen/touch pressure normalized to 0..1 from the last pointer event that
    /// reported a calibrated force (Windows Ink WM_POINTER pen arrives here as
    /// `WindowEvent::Touch`). `None` until a pressure-carrying pointer is seen;
    /// plain mouse input leaves the last value in place.
    pub pointer_pressure: Option<f32>,
    /// Net horizontal mouse-wheel delta gathered this frame.
    pub wheel_delta_x: f32,
    /// Net vertical mouse-wheel delta gathered this frame.
    pub wheel_delta_y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WindowSurfaceFrameContext {
    pub surface_size: SurfaceSize,
    pub input: WindowSurfaceInput,
}

fn clip_position_from_physical_cursor(
    position: winit::dpi::PhysicalPosition<f64>,
    surface_size: SurfaceSize,
) -> [f32; 2] {
    let nx = (position.x / surface_size.width.max(1) as f64) * 2.0 - 1.0;
    let ny = 1.0 - (position.y / surface_size.height.max(1) as f64) * 2.0;
    [nx as f32, ny as f32]
}

impl From<&WindowSurfaceConfig> for SurfaceSize {
    fn from(value: &WindowSurfaceConfig) -> Self {
        Self {
            width: value.width,
            height: value.height,
        }
    }
}

pub fn run_window_surface(config: WindowSurfaceConfig, scene: WindowSurfaceScene) -> Result<()> {
    let scene = std::sync::Arc::new(scene);
    run_window_surface_with_scene_provider(config, move |_| Ok(scene.clone()))
}

pub fn run_window_surface_with_scene_provider(
    config: WindowSurfaceConfig,
    mut scene_provider: impl FnMut(SurfaceSize) -> Result<SharedWindowSurfaceScene> + 'static,
) -> Result<()> {
    run_window_surface_with_frame_provider(config, move |frame| scene_provider(frame.surface_size))
}

/// One shared scene per call: producers that reuse an unchanged scene
/// (boot's scene cache) hand back the same `Arc` instead of cloning quad
/// data every frame.
pub type SharedWindowSurfaceScene = std::sync::Arc<WindowSurfaceScene>;

pub fn run_window_surface_with_frame_provider(
    config: WindowSurfaceConfig,
    scene_provider: impl FnMut(WindowSurfaceFrameContext) -> Result<SharedWindowSurfaceScene>
        + 'static,
) -> Result<()> {
    let event_loop = EventLoop::new()?;
    let mut app = WindowSurfaceApp::new(config, Box::new(scene_provider));
    event_loop.run_app(&mut app)?;
    Ok(())
}

struct WindowSurfaceApp {
    config: WindowSurfaceConfig,
    scene_provider: Box<dyn FnMut(WindowSurfaceFrameContext) -> Result<SharedWindowSurfaceScene>>,
    window: Option<Arc<Window>>,
    gpu_surface: Option<GpuSurface>,
    surface_size: SurfaceSize,
    pressed_keys: Vec<KeyCode>,
    just_pressed_keys: Vec<KeyCode>,
    cursor_position: Option<[f32; 2]>,
    just_clicked: Option<[f32; 2]>,
    pointer_down: bool,
    just_right_clicked: Option<[f32; 2]>,
    right_pointer_down: bool,
    pointer_pressure: Option<f32>,
    wheel_delta_x: f32,
    wheel_delta_y: f32,
    performance_log: Option<PerformanceLogState>,
    pending_frame_sample: Option<PerformanceSample>,
    last_redraw_at: Option<Instant>,
}

impl WindowSurfaceApp {
    fn new(
        config: WindowSurfaceConfig,
        scene_provider: Box<dyn FnMut(WindowSurfaceFrameContext) -> Result<SharedWindowSurfaceScene>>,
    ) -> Self {
        let surface_size = SurfaceSize::from(&config);

        let performance_log = config
            .performance_log_path
            .clone()
            .map(PerformanceLogState::new);

        Self {
            config,
            scene_provider,
            window: None,
            gpu_surface: None,
            surface_size,
            pressed_keys: Vec::new(),
            just_pressed_keys: Vec::new(),
            cursor_position: None,
            just_clicked: None,
            pointer_down: false,
            just_right_clicked: None,
            right_pointer_down: false,
            pointer_pressure: None,
            wheel_delta_x: 0.0,
            wheel_delta_y: 0.0,
            performance_log,
            pending_frame_sample: None,
            last_redraw_at: None,
        }
    }

    fn window_attributes(&self) -> WindowAttributes {
        Window::default_attributes()
            .with_title(self.config.title.clone())
            .with_inner_size(LogicalSize::new(self.config.width, self.config.height))
    }

    /// Maps pen/touch pointer events onto the same primary-pointer state the
    /// mouse path drives. Windows Ink pen input (Huion, XP-Pen, Surface, …)
    /// arrives from winit on Windows exclusively as `WindowEvent::Touch` —
    /// winit consumes the WM_POINTER frames and suppresses the synthesized
    /// legacy mouse messages, so without this arm pen movement tracks but
    /// pen-down never registers. Pen hover emits `TouchPhase::Moved` with no
    /// contact, which updates position without pressing.
    fn apply_pointer_touch(&mut self, touch: Touch) {
        let position = clip_position_from_physical_cursor(touch.location, self.surface_size);
        self.cursor_position = Some(position);
        self.pointer_pressure = match touch.force {
            Some(Force::Calibrated { force, max_possible_force, .. }) => {
                let denominator = if max_possible_force > 0.0 { max_possible_force } else { 1.0 };
                Some(((force / denominator) as f32).clamp(0.0, 1.0))
            }
            _ => None,
        };
        match touch.phase {
            TouchPhase::Started => {
                self.pointer_down = true;
                self.just_clicked = Some(position);
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                self.pointer_down = false;
            }
            TouchPhase::Moved => {}
        }
    }
}

impl ApplicationHandler for WindowSurfaceApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = Arc::new(
                event_loop
                    .create_window(self.window_attributes())
                    .expect("failed to create renderer window"),
            );

            let scene = (self.scene_provider)(WindowSurfaceFrameContext {
                surface_size: self.surface_size,
                input: WindowSurfaceInput::default(),
            })
            .expect("failed to build initial renderer scene");
            let initial_sample = PerformanceSample::from_scene(
                self.surface_size,
                scaled_internal_surface_size(self.surface_size, self.config.internal_render_scale),
                &scene,
            );
            let gpu_surface = pollster::block_on(GpuSurface::new(
                window.clone(),
                scene,
                self.config.internal_render_scale,
                self.config.upscale_mode,
            ))
            .expect("failed to create GPU presentation surface");
            if let Some(performance_log) = &mut self.performance_log {
                performance_log.set_adapter(gpu_surface.adapter_label.clone());
            }

            self.pending_frame_sample = Some(initial_sample);

            self.surface_size = SurfaceSize::from(&self.config);
            self.gpu_surface = Some(gpu_surface);
            self.window = Some(window);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                self.surface_size = SurfaceSize {
                    width: size.width,
                    height: size.height,
                };

                if let Some(gpu_surface) = &mut self.gpu_surface {
                    gpu_surface.resize(self.surface_size);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    match event.state {
                        ElementState::Pressed if !event.repeat => {
                            if !self.pressed_keys.contains(&code) {
                                self.pressed_keys.push(code);
                            }
                            self.just_pressed_keys.push(code);
                        }
                        ElementState::Released => {
                            self.pressed_keys.retain(|pressed| *pressed != code);
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = Some(clip_position_from_physical_cursor(
                    position,
                    self.surface_size,
                ));
            }
            WindowEvent::CursorLeft { .. } => {
                self.cursor_position = None;
            }
            WindowEvent::Touch(touch) => {
                self.apply_pointer_touch(touch);
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                self.pointer_down = true;
                if let Some(position) = self.cursor_position {
                    self.just_clicked = Some(position);
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                self.pointer_down = false;
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Right,
                ..
            } => {
                self.right_pointer_down = true;
                if let Some(position) = self.cursor_position {
                    self.just_right_clicked = Some(position);
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Right,
                ..
            } => {
                self.right_pointer_down = false;
            }
            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(x, y) => {
                    self.wheel_delta_x += x;
                    self.wheel_delta_y += y;
                }
                MouseScrollDelta::PixelDelta(position) => {
                    self.wheel_delta_x += position.x as f32;
                    self.wheel_delta_y += position.y as f32;
                }
            },
            WindowEvent::RedrawRequested => {
                // Wall frame pacing: time since the previous RedrawRequested,
                // captured before any of this frame's work so the sample
                // reflects the true interval between presented frames.
                let wall_ms = self
                    .last_redraw_at
                    .replace(Instant::now())
                    .map(|previous| previous.elapsed().as_secs_f64() * 1000.0)
                    .unwrap_or(0.0);
                if let Some(sample) = &mut self.pending_frame_sample {
                    sample.wall_ms = wall_ms;
                }
                if let Some(gpu_surface) = &mut self.gpu_surface {
                    let render_started_at = Instant::now();
                    match gpu_surface.render(self.config.clear_color) {
                        RenderOutcome::Ok => {
                            if let Some(sample) = &mut self.pending_frame_sample {
                                sample.render_ms =
                                    render_started_at.elapsed().as_secs_f64() * 1000.0;
                            }
                            if let (Some(performance_log), Some(sample)) =
                                (&mut self.performance_log, self.pending_frame_sample.take())
                            {
                                performance_log.record(sample);
                            }
                        }
                        RenderOutcome::Reconfigure => gpu_surface.resize(self.surface_size),
                        RenderOutcome::Skip => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(gpu_surface) = &mut self.gpu_surface {
            let scene_build_started_at = Instant::now();
            let scene = match (self.scene_provider)(WindowSurfaceFrameContext {
                surface_size: self.surface_size,
                input: WindowSurfaceInput {
                    pressed_keys: self.pressed_keys.clone(),
                    just_pressed_keys: self.just_pressed_keys.clone(),
                    cursor_position: self.cursor_position,
                    just_clicked: self.just_clicked,
                    pointer_down: self.pointer_down,
                    just_right_clicked: self.just_right_clicked,
                    right_pointer_down: self.right_pointer_down,
                    pointer_pressure: self.pointer_pressure,
                    wheel_delta_x: self.wheel_delta_x,
                    wheel_delta_y: self.wheel_delta_y,
                },
            }) {
                Ok(scene) => scene,
                Err(error) => {
                    panic!("failed to build renderer scene: {error:#}");
                }
            };
            self.just_pressed_keys.clear();
            self.just_clicked = None;
            self.just_right_clicked = None;
            self.wheel_delta_x = 0.0;
            self.wheel_delta_y = 0.0;
            let scene_build_ms = scene_build_started_at.elapsed().as_secs_f64() * 1000.0;
            let mut sample = PerformanceSample::from_scene(
                self.surface_size,
                scaled_internal_surface_size(self.surface_size, self.config.internal_render_scale),
                &scene,
            );
            sample.scene_build_ms = scene_build_ms;

            let scene_upload_started_at = Instant::now();
            gpu_surface.update_scene(&scene);
            sample.scene_upload_ms = scene_upload_started_at.elapsed().as_secs_f64() * 1000.0;
            self.pending_frame_sample = Some(sample);
        }

        if let Some(window) = &self.window {
            window.request_redraw();
        } else {
            event_loop.exit();
        }
    }
}

struct GpuSurface {
    adapter_label: String,
    _instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    internal_render_scale: f32,
    upscale_mode: WindowSurfaceUpscaleMode,
    quad_draw: Option<QuadDraw>,
    /// Revision of the last scene whose frame data was pushed to the GPU.
    uploaded_revision: Option<u64>,
}

impl GpuSurface {
    async fn new(
        window: Arc<Window>,
        scene: SharedWindowSurfaceScene,
        internal_render_scale: f32,
        upscale_mode: WindowSurfaceUpscaleMode,
    ) -> Result<Self> {
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window.clone())
            .context("failed to bind window to GPU surface")?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await
            .context("failed to find a GPU adapter for the renderer surface")?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .context("failed to create renderer GPU device")?;
        let adapter_info = adapter.get_info();

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = pick_surface_format(&surface_capabilities.formats)
            .context("renderer surface exposed no usable texture format")?;
        let present_mode = pick_present_mode(&surface_capabilities.present_modes);
        let alpha_mode = pick_alpha_mode(&surface_capabilities.alpha_modes);
        let size = window.inner_size();

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            color_space: SurfaceColorSpace::Auto,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode,
            desired_maximum_frame_latency: 2,
            alpha_mode,
            view_formats: vec![],
        };

        surface.configure(&device, &surface_config);

        let output_surface_size = SurfaceSize {
            width: surface_config.width,
            height: surface_config.height,
        };
        let internal_surface_size =
            scaled_internal_surface_size(output_surface_size, internal_render_scale);

        let quad_draw = QuadDraw::new(
            &device,
            &queue,
            surface_format,
            output_surface_size,
            internal_surface_size,
            upscale_mode,
            &scene,
        );

        Ok(Self {
            _instance: instance,
            surface,
            device,
            queue,
            surface_config,
            internal_render_scale,
            upscale_mode,
            quad_draw,
            // QuadDraw::new creates buffers but pushes no frame data; the
            // first update_scene call performs the initial upload.
            uploaded_revision: None,
            adapter_label: format!(
                "{} / {:?} / {}",
                adapter_info.name, adapter_info.backend, adapter_info.driver
            ),
        })
    }

    fn resize(&mut self, size: SurfaceSize) {
        if size.width == 0 || size.height == 0 {
            return;
        }

        self.surface_config.width = size.width;
        self.surface_config.height = size.height;
        self.surface.configure(&self.device, &self.surface_config);
    }

    fn update_scene(&mut self, scene: &WindowSurfaceScene) {
        let output_surface_size = SurfaceSize {
            width: self.surface_config.width,
            height: self.surface_config.height,
        };
        let internal_surface_size =
            scaled_internal_surface_size(output_surface_size, self.internal_render_scale);

        // Persistent resources: pipelines, offscreen targets, bind groups, and
        // the vertex buffer survive across frames. They are only rebuilt when
        // the rebuild signature changes (size, format, palette length). Per-
        // frame data (vertices, post uniforms, palette texels) flows through
        // queue writes into the existing buffers instead.
        let needs_rebuild = self
            .quad_draw
            .as_ref()
            .map(|quad_draw| quad_draw.needs_rebuild(
                self.surface_config.format,
                output_surface_size,
                internal_surface_size,
                self.upscale_mode,
                scene.indexed_color_palette.len(),
                &scene.glyph_atlas,
            ))
            .unwrap_or(true);
        if needs_rebuild {
            self.quad_draw = QuadDraw::new(
                &self.device,
                &self.queue,
                self.surface_config.format,
                output_surface_size,
                internal_surface_size,
                self.upscale_mode,
                scene,
            );
        } else if self.uploaded_revision == Some(scene.revision) {
            // Unchanged scene revision: the GPU already holds this frame's
            // vertices, atlas, palette, and post uniforms — skip the
            // re-upload.
            return;
        }
        if let Some(quad_draw) = &mut self.quad_draw {
            quad_draw.update_frame(&self.device, &self.queue, scene);
        }
        self.uploaded_revision = Some(scene.revision);
    }

    fn render(&mut self, clear_color: [f64; 4]) -> RenderOutcome {
        if self.surface_config.width == 0 || self.surface_config.height == 0 {
            return RenderOutcome::Skip;
        }

        let surface_texture = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(surface_texture)
            | CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,
            CurrentSurfaceTexture::Timeout
            | CurrentSurfaceTexture::Occluded
            | CurrentSurfaceTexture::Validation => return RenderOutcome::Skip,
            CurrentSurfaceTexture::Outdated | CurrentSurfaceTexture::Lost => {
                return RenderOutcome::Reconfigure;
            }
        };
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("thaum-renderer-window-surface-encoder"),
            });

        if let Some(quad_draw) = &self.quad_draw {
            quad_draw.record(&mut encoder, &view, clear_color);
        } else {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("thaum-renderer-window-surface-clear-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: clear_color[0],
                            g: clear_color[1],
                            b: clear_color[2],
                            a: clear_color[3],
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }

        self.queue.submit([encoder.finish()]);
        self.queue.present(surface_texture);

        RenderOutcome::Ok
    }
}

struct QuadDraw {
    quad_pipeline: wgpu::RenderPipeline,
    post_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_capacity_bytes: usize,
    vertex_count: u32,
    post_bind_group: wgpu::BindGroup,
    post_uniform_buffer: wgpu::Buffer,
    indexed_palette_texture: wgpu::Texture,
    atlas_texture: wgpu::Texture,
    atlas_bind_group: wgpu::BindGroup,
    uploaded_atlas: GlyphAtlasSceneData,
    color_view: wgpu::TextureView,
    bus_view: wgpu::TextureView,
    meta_view: wgpu::TextureView,
    warble_view: wgpu::TextureView,
    _indexed_palette_view: wgpu::TextureView,
    // Rebuild signature: resources are only recreated when one of these
    // changes; otherwise frames reuse the same GPU objects.
    surface_format: TextureFormat,
    output_surface_size: SurfaceSize,
    internal_surface_size: SurfaceSize,
    upscale_mode: WindowSurfaceUpscaleMode,
    palette_len: usize,
}

impl QuadDraw {
    /// Vertex bytes needed for `quad_count` quads (6 verts × 18 floats each).
    fn vertex_bytes_for(quad_count: usize) -> usize {
        quad_count * 6 * std::mem::size_of::<SurfaceVertex>()
    }

    const VERTEX_CAPACITY_MIN_BYTES: usize = 1 << 20; // 1 MiB floor

    fn needs_rebuild(
        &self,
        surface_format: TextureFormat,
        output_surface_size: SurfaceSize,
        internal_surface_size: SurfaceSize,
        upscale_mode: WindowSurfaceUpscaleMode,
        palette_len: usize,
        atlas: &GlyphAtlasSceneData,
    ) -> bool {
        self.surface_format != surface_format
            || self.output_surface_size != output_surface_size
            || self.internal_surface_size != internal_surface_size
            || self.upscale_mode != upscale_mode
            || self.palette_len != palette_len
            // Atlas grid geometry owns the texture/bind-group shape; a grid
            // change rebuilds resources, key changes only re-upload texels.
            || self.uploaded_atlas.tile_width != atlas.tile_width
            || self.uploaded_atlas.tile_height != atlas.tile_height
            || self.uploaded_atlas.columns != atlas.columns
            || self.uploaded_atlas.rows != atlas.rows
    }

    fn write_atlas_texels(
        queue: &wgpu::Queue,
        atlas_texture: &wgpu::Texture,
        atlas: &GlyphAtlasSceneData,
    ) {
        let columns = atlas.columns.max(1) as usize;
        let rows = atlas.rows.max(1) as usize;
        let tile_width = atlas.tile_width.max(1) as usize;
        let tile_height = atlas.tile_height.max(1) as usize;
        let tile_bytes = tile_width * tile_height;
        let mut data = vec![0u8; columns * rows * tile_bytes];
        // Tiles go to their strided slot: the buffer has one row per texture
        // row (bytes_per_row = columns * tile_width), so a flat per-tile
        // copy would scramble every tile not in the first atlas column.
        for (index, tile) in atlas.tiles.iter().enumerate() {
            let column = index % columns;
            let row = index / columns;
            for tile_row in 0..tile_height {
                let destination = (row * tile_height + tile_row) * columns * tile_width
                    + column * tile_width;
                let source = tile_row * tile_width;
                data[destination..destination + tile_width]
                    .copy_from_slice(&tile.alpha[source..source + tile_width]);
            }
        }
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: atlas_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some((columns * tile_width) as u32),
                rows_per_image: Some((rows * tile_height) as u32),
            },
            wgpu::Extent3d {
                width: (columns * tile_width) as u32,
                height: (rows * tile_height) as u32,
                depth_or_array_layers: 1,
            },
        );
    }

    fn update_atlas(&mut self, queue: &wgpu::Queue, atlas: &GlyphAtlasSceneData) {
        let keys_match = self.uploaded_atlas.tiles.len() == atlas.tiles.len()
            && self
                .uploaded_atlas
                .tiles
                .iter()
                .zip(atlas.tiles.iter())
                .all(|(uploaded, incoming)| {
                    uploaded.glyph == incoming.glyph
                        && uploaded.weight_index == incoming.weight_index
                });
        if keys_match {
            return;
        }

        Self::write_atlas_texels(queue, &self.atlas_texture, atlas);
        self.uploaded_atlas = atlas.clone();
    }

    /// Per-frame update: pushes vertices, post uniforms, and palette texels
    /// into the persistent GPU buffers without recreating any GPU objects.
    fn update_frame(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &WindowSurfaceScene,
    ) {
        queue.write_buffer(
            &self.post_uniform_buffer,
            0,
            bytemuck::bytes_of(&TexturePostEffectUniforms::new(
                scene,
                self.internal_surface_size,
                self.upscale_mode,
            )),
        );

        self.update_atlas(queue, &scene.glyph_atlas);

        if !scene.indexed_color_palette.is_empty() {
            let palette_bytes: Vec<u8> = scene
                .indexed_color_palette
                .iter()
                .flat_map(|rgba| rgba.iter().copied())
                .collect();
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.indexed_palette_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &palette_bytes,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some((scene.indexed_color_palette.len() * 4) as u32),
                    rows_per_image: Some(1),
                },
                wgpu::Extent3d {
                    width: scene.indexed_color_palette.len() as u32,
                    height: 1,
                    depth_or_array_layers: 1,
                },
            );
        }

        let needed_bytes = Self::vertex_bytes_for(scene.quads.len());
        if needed_bytes > self.vertex_capacity_bytes {
            let grown_capacity = needed_bytes + needed_bytes / 2;
            self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("thaum-renderer-surface-quad-vertices"),
                size: grown_capacity as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.vertex_capacity_bytes = grown_capacity;
        }
        if needed_bytes > 0 {
            let vertices: Vec<_> = scene.quads.iter().flat_map(SurfaceQuad::vertices).collect();
            queue.write_buffer(
                &self.vertex_buffer,
                0,
                bytemuck::cast_slice(&vertices),
            );
            self.vertex_count = vertices.len() as u32;
        } else {
            self.vertex_count = 0;
        }
    }

    fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: TextureFormat,
        _output_surface_size: SurfaceSize,
        internal_surface_size: SurfaceSize,
        upscale_mode: WindowSurfaceUpscaleMode,
        scene: &WindowSurfaceScene,
    ) -> Option<Self> {
        if scene.quads.is_empty() {
            return None;
        }

        let color_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("thaum-renderer-offscreen-color"),
            size: wgpu::Extent3d {
                width: internal_surface_size.width.max(1),
                height: internal_surface_size.height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let bus_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("thaum-renderer-offscreen-bus"),
            size: wgpu::Extent3d {
                width: internal_surface_size.width.max(1),
                height: internal_surface_size.height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let meta_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("thaum-renderer-offscreen-meta"),
            size: wgpu::Extent3d {
                width: internal_surface_size.width.max(1),
                height: internal_surface_size.height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let warble_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("thaum-renderer-offscreen-warble"),
            size: wgpu::Extent3d {
                width: internal_surface_size.width.max(1),
                height: internal_surface_size.height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let indexed_palette_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("thaum-renderer-indexed-palette"),
            size: wgpu::Extent3d {
                width: scene.indexed_color_palette.len().max(1) as u32,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let color_view = color_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bus_view = bus_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let meta_view = meta_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let warble_view = warble_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let indexed_palette_view =
            indexed_palette_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let quad_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("thaum-renderer-surface-quad-shader"),
            source: wgpu::ShaderSource::Wgsl(SURFACE_QUAD_SHADER.into()),
        });
        let post_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("thaum-renderer-post-shader"),
            source: wgpu::ShaderSource::Wgsl(POST_EFFECTS_SHADER.into()),
        });

        let atlas_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("thaum-renderer-atlas-bind-group-layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        let quad_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("thaum-renderer-surface-quad-pipeline-layout"),
            bind_group_layouts: &[Some(&atlas_bind_group_layout)],
            immediate_size: 0,
        });

        let atlas_columns = (scene.glyph_atlas.columns.max(1) as usize
            * scene.glyph_atlas.tile_width.max(1) as usize) as u32;
        let atlas_rows = (scene.glyph_atlas.rows.max(1) as usize
            * scene.glyph_atlas.tile_height.max(1) as usize) as u32;
        let atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("thaum-renderer-glyph-atlas"),
            size: wgpu::Extent3d {
                width: atlas_columns,
                height: atlas_rows,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let atlas_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let atlas_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("thaum-renderer-atlas-nearest-sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        let atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("thaum-renderer-atlas-bind-group"),
            layout: &atlas_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&atlas_sampler),
                },
            ],
        });

        // The atlas texture is sized to the scene's grid and seeded with the
        // scene's tiles here; later key-only changes re-upload through
        // update_atlas. Grid-dimension changes go through needs_rebuild.
        Self::write_atlas_texels(queue, &atlas_texture, &scene.glyph_atlas);

        let quad_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("thaum-renderer-surface-quad-pipeline"),            layout: Some(&quad_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &quad_shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Some(SurfaceVertex::layout())],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &quad_shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: TextureFormat::Rgba8Unorm,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: TextureFormat::Rgba8Unorm,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: TextureFormat::Rgba8Unorm,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: TextureFormat::Rgba16Float,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ],
            }),
            multiview_mask: None,
            cache: None,
        });

        let nearest_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("thaum-renderer-post-nearest-sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        let linear_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("thaum-renderer-post-linear-sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        let post_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("thaum-renderer-post-uniforms"),
            size: std::mem::size_of::<TexturePostEffectUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let post_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("thaum-renderer-post-bind-group-layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 7,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        let post_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("thaum-renderer-post-bind-group"),
            layout: &post_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&color_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&bus_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&meta_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&nearest_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&linear_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: post_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(&warble_view),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::TextureView(&indexed_palette_view),
                },
            ],
        });

        let post_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("thaum-renderer-post-pipeline-layout"),
            bind_group_layouts: &[Some(&post_bind_group_layout)],
            immediate_size: 0,
        });

        let post_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("thaum-renderer-post-pipeline"),
            layout: Some(&post_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &post_shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &post_shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        let initial_vertex_capacity = (Self::vertex_bytes_for(scene.quads.len()))
            .max(Self::VERTEX_CAPACITY_MIN_BYTES);
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("thaum-renderer-surface-quad-vertices"),
            size: initial_vertex_capacity as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Some(Self {
            quad_pipeline,
            post_pipeline,
            vertex_buffer,
            vertex_capacity_bytes: initial_vertex_capacity,
            vertex_count: 0,
            post_bind_group,
            post_uniform_buffer,
            indexed_palette_texture,
            atlas_texture,
            atlas_bind_group,
            uploaded_atlas: scene.glyph_atlas.clone(),
            color_view,
            bus_view,
            meta_view,
            warble_view,
            _indexed_palette_view: indexed_palette_view,
            surface_format,
            output_surface_size: _output_surface_size,
            internal_surface_size,
            upscale_mode,
            palette_len: scene.indexed_color_palette.len(),
        })
    }

    fn record(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        surface_view: &wgpu::TextureView,
        clear_color: [f64; 4],
    ) {
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("thaum-renderer-window-surface-quad-pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.color_view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.bus_view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.meta_view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.warble_view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            render_pass.set_pipeline(&self.quad_pipeline);
            render_pass.set_bind_group(0, &self.atlas_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..self.vertex_count, 0..1);
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("thaum-renderer-window-surface-post-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: surface_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: clear_color[0],
                            g: clear_color[1],
                            b: clear_color[2],
                            a: clear_color[3],
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            render_pass.set_pipeline(&self.post_pipeline);
            render_pass.set_bind_group(0, &self.post_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }
    }
}

const POST_EFFECT_TEXEL_UV_PACK_SCALE: f32 = 64.0;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct SurfaceVertex {
    position: [f32; 2],
    color: [f32; 4],
    bus: [f32; 4],
    meta: [f32; 4],
    warble_uv: [f32; 4],
    atlas_uv: [f32; 4],
}

impl SurfaceVertex {
    fn new(
        quad: &SurfaceQuad,
        position: [f32; 2],
        local_uv: [f32; 2],
        warble_uv: [f32; 2],
    ) -> Self {
        let gate_hi = ((quad.post_effect_bus.gate_id >> 8) & 0xff) as u8;
        let gate_lo = (quad.post_effect_bus.gate_id & 0xff) as u8;

        let texel_uv_size = texture_post_effect_texel_uv_size(quad.size);
        let packed_texel_uv_size =
            (texel_uv_size * POST_EFFECT_TEXEL_UV_PACK_SCALE).clamp(0.0, 1.0);

        Self {
            position,
            color: quad.color,
            bus: [
                normalize_byte(quad.post_effect_bus.texture_code),
                normalize_byte(quad.post_effect_bus.warble_code),
                normalize_byte(quad.post_effect_bus.depth_code),
                normalize_byte(gate_hi),
            ],
            meta: [
                local_uv[0],
                local_uv[1],
                packed_texel_uv_size,
                normalize_byte(gate_lo),
            ],
            warble_uv: [warble_uv[0], warble_uv[1], 0.0, 0.0],
            atlas_uv: quad.atlas_uv,
        }
    }

    fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<SurfaceVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: (mem::size_of::<[f32; 2]>() + mem::size_of::<[f32; 4]>())
                        as wgpu::BufferAddress,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: (mem::size_of::<[f32; 2]>() + mem::size_of::<[f32; 4]>() * 2)
                        as wgpu::BufferAddress,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: (mem::size_of::<[f32; 2]>() + mem::size_of::<[f32; 4]>() * 3)
                        as wgpu::BufferAddress,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: (mem::size_of::<[f32; 2]>() + mem::size_of::<[f32; 4]>() * 4)
                        as wgpu::BufferAddress,
                    shader_location: 5,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct TexturePostEffectUniforms {
    values: [f32; 16],
}

impl TexturePostEffectUniforms {
    fn new(
        scene: &WindowSurfaceScene,
        surface_size: SurfaceSize,
        upscale_mode: WindowSurfaceUpscaleMode,
    ) -> Self {
        let depth_of_field_uv_step =
            if scene.depth_of_field_enabled && surface_size.width > 0 && surface_size.height > 0 {
                [
                    1.0 / surface_size.width as f32,
                    1.0 / surface_size.height as f32,
                ]
            } else {
                [0.0, 0.0]
            };

        Self {
            values: [
                texture_post_effect_seed_step(scene.texture_breath) as f32,
                warble_post_effect_breath_phase(scene.texture_breath),
                depth_of_field_uv_step[0],
                depth_of_field_uv_step[1],
                scene.depth_of_field_minimum_falloff_cells,
                scene.fog_span_cells,
                scene.fog_nearest_depth_code as f32,
                scene.fog_farthest_depth_code as f32,
                scene.background_color[0],
                scene.background_color[1],
                scene.background_color[2],
                scene.background_color[3],
                if scene.motion_noise_enabled { 1.0 } else { 0.0 },
                if scene.indexed_color_enabled {
                    1.0
                } else {
                    0.0
                },
                scene.indexed_color_palette.len() as f32,
                if upscale_mode.uses_linear_filter() {
                    1.0
                } else {
                    0.0
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RenderOutcome {
    Ok,
    Reconfigure,
    Skip,
}

#[derive(Debug, Clone)]
struct PerformanceSample {
    width: u32,
    height: u32,
    internal_width: u32,
    internal_height: u32,
    quad_count: usize,
    vertex_count: usize,
    depth_of_field_enabled: bool,
    motion_noise_enabled: bool,
    scene_build_ms: f64,
    scene_upload_ms: f64,
    render_ms: f64,
    /// Wall time since the previous RedrawRequested. Reflects real frame pacing
    /// (vsync included), unlike render_ms which only measures CPU submit cost.
    wall_ms: f64,
}

impl PerformanceSample {
    fn from_scene(
        surface_size: SurfaceSize,
        internal_surface_size: SurfaceSize,
        scene: &WindowSurfaceScene,
    ) -> Self {
        Self {
            width: surface_size.width,
            height: surface_size.height,
            internal_width: internal_surface_size.width,
            internal_height: internal_surface_size.height,
            quad_count: scene.quads.len(),
            vertex_count: scene.quads.len() * 6,
            depth_of_field_enabled: scene.depth_of_field_enabled,
            motion_noise_enabled: scene.motion_noise_enabled,
            scene_build_ms: 0.0,
            scene_upload_ms: 0.0,
            render_ms: 0.0,
            wall_ms: 0.0,
        }
    }

    fn key(&self) -> PerformanceSampleKey {
        PerformanceSampleKey {
            width: self.width,
            height: self.height,
            internal_width: self.internal_width,
            internal_height: self.internal_height,
            depth_of_field_enabled: self.depth_of_field_enabled,
            motion_noise_enabled: self.motion_noise_enabled,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PerformanceSampleKey {
    width: u32,
    height: u32,
    internal_width: u32,
    internal_height: u32,
    depth_of_field_enabled: bool,
    motion_noise_enabled: bool,
}

#[derive(Debug)]
struct PerformanceLogBucket {
    key: PerformanceSampleKey,
    started_at_unix_ms: u128,
    frame_count: u32,
    quad_count_sum: usize,
    vertex_count_sum: usize,
    scene_build_ms_sum: f64,
    scene_upload_ms_sum: f64,
    render_ms_sum: f64,
    wall_ms_sum: f64,
}

impl PerformanceLogBucket {
    fn new(sample: &PerformanceSample) -> Self {
        Self {
            key: sample.key(),
            started_at_unix_ms: unix_time_ms(),
            frame_count: 0,
            quad_count_sum: 0,
            vertex_count_sum: 0,
            scene_build_ms_sum: 0.0,
            scene_upload_ms_sum: 0.0,
            render_ms_sum: 0.0,
            wall_ms_sum: 0.0,
        }
    }

    fn push(&mut self, sample: &PerformanceSample) {
        self.frame_count += 1;
        self.quad_count_sum += sample.quad_count;
        self.vertex_count_sum += sample.vertex_count;
        self.scene_build_ms_sum += sample.scene_build_ms;
        self.scene_upload_ms_sum += sample.scene_upload_ms;
        self.render_ms_sum += sample.render_ms;
        self.wall_ms_sum += sample.wall_ms;
    }
}

#[derive(Debug)]
struct PerformanceLogState {
    path: PathBuf,
    bucket: Option<PerformanceLogBucket>,
    adapter: Option<String>,
}

impl PerformanceLogState {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            bucket: None,
            adapter: None,
        }
    }

    fn set_adapter(&mut self, adapter_label: String) {
        self.adapter = Some(adapter_label);
    }

    fn adapter_json(&self) -> String {
        self.adapter
            .as_deref()
            .unwrap_or("unknown")
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
    }

    fn record(&mut self, sample: PerformanceSample) {
        let sample_key = sample.key();
        let should_flush = self
            .bucket
            .as_ref()
            .map(|bucket| bucket.key != sample_key || bucket.frame_count >= 60)
            .unwrap_or(false);

        if should_flush {
            self.flush();
        }

        let bucket = self
            .bucket
            .get_or_insert_with(|| PerformanceLogBucket::new(&sample));
        bucket.push(&sample);

        if bucket.frame_count >= 60 {
            self.flush();
        }
    }

    fn flush(&mut self) {
        let Some(bucket) = self.bucket.take() else {
            return;
        };

        let average = |sum: f64| sum / bucket.frame_count as f64;
        let avg_total_ms =
            average(bucket.scene_build_ms_sum + bucket.scene_upload_ms_sum + bucket.render_ms_sum);
        let approx_fps = if avg_total_ms > 0.0 {
            1000.0 / avg_total_ms
        } else {
            0.0
        };
        let avg_wall_ms = average(bucket.wall_ms_sum);
        let wall_fps = if avg_wall_ms > 0.0 {
            1000.0 / avg_wall_ms
        } else {
            0.0
        };
        let line = format!(
            concat!(
                "{{",
                "\"timestamp_ms\":{},",
                "\"adapter\":\"{}\",",
                "\"frames\":{},",
                "\"surface\":{{\"width\":{},\"height\":{}}},",
                "\"internal_surface\":{{\"width\":{},\"height\":{}}},",
                "\"effects\":{{\"dof\":{},\"motion_noise\":{}}},",
                "\"avg_quads\":{:.2},",
                "\"avg_vertices\":{:.2},",
                "\"avg_scene_build_ms\":{:.4},",
                "\"avg_scene_upload_ms\":{:.4},",
                "\"avg_render_ms\":{:.4},",
                "\"avg_total_ms\":{:.4},",
                "\"approx_fps\":{:.2},",
                "\"avg_wall_ms\":{:.4},",
                "\"wall_fps\":{:.2}",
                "}}\n"
            ),
            bucket.started_at_unix_ms,
            self.adapter_json(),
            bucket.frame_count,
            bucket.key.width,
            bucket.key.height,
            bucket.key.internal_width,
            bucket.key.internal_height,
            bucket.key.depth_of_field_enabled,
            bucket.key.motion_noise_enabled,
            bucket.quad_count_sum as f64 / bucket.frame_count as f64,
            bucket.vertex_count_sum as f64 / bucket.frame_count as f64,
            average(bucket.scene_build_ms_sum),
            average(bucket.scene_upload_ms_sum),
            average(bucket.render_ms_sum),
            avg_total_ms,
            approx_fps,
            avg_wall_ms,
            wall_fps,
        );

        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            let _ = file.write_all(line.as_bytes());
        }
    }
}

impl Drop for PerformanceLogState {
    fn drop(&mut self) {
        self.flush();
    }
}

fn unix_time_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

fn scaled_internal_surface_size(surface_size: SurfaceSize, scale: f32) -> SurfaceSize {
    let clamped_scale = scale.clamp(0.1, 1.0);
    let snap = |value: u32| -> u32 {
        let scaled = ((value as f32) * clamped_scale).round().max(1.0) as u32;
        if scaled <= 2 {
            scaled
        } else {
            scaled - (scaled % 2)
        }
    };

    SurfaceSize {
        width: snap(surface_size.width.max(1)),
        height: snap(surface_size.height.max(1)),
    }
}

fn pick_surface_format(formats: &[TextureFormat]) -> Option<TextureFormat> {
    formats
        .iter()
        .copied()
        .find(|format| !format.is_srgb())
        .or_else(|| formats.iter().copied().find(TextureFormat::is_srgb))
        .or_else(|| formats.first().copied())
}

fn pick_present_mode(modes: &[PresentMode]) -> PresentMode {
    modes
        .iter()
        .copied()
        .find(|mode| *mode == PresentMode::Fifo)
        .or_else(|| modes.first().copied())
        .unwrap_or(PresentMode::Fifo)
}

fn pick_alpha_mode(modes: &[CompositeAlphaMode]) -> CompositeAlphaMode {
    modes
        .iter()
        .copied()
        .find(|mode| *mode == CompositeAlphaMode::Opaque)
        .or_else(|| modes.first().copied())
        .unwrap_or(CompositeAlphaMode::Opaque)
}

fn normalize_byte(value: u8) -> f32 {
    value as f32 / 255.0
}

const SURFACE_QUAD_SHADER: &str = r#"
@group(0) @binding(0) var atlas_texture: texture_2d<f32>;
@group(0) @binding(1) var atlas_sampler: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) bus: vec4<f32>,
    @location(3) aux_data: vec4<f32>,
    @location(4) warble_uv: vec4<f32>,
    @location(5) atlas_uv: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) bus: vec4<f32>,
    @location(2) aux_data: vec4<f32>,
    @location(3) warble_uv: vec4<f32>,
    @location(4) atlas_uv: vec4<f32>,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @location(1) bus: vec4<f32>,
    @location(2) aux_data: vec4<f32>,
    @location(3) warble_uv: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = vec4<f32>(input.position, 0.0, 1.0);
    output.color = input.color;
    output.bus = input.bus;
    output.aux_data = input.aux_data;
    output.warble_uv = input.warble_uv;
    output.atlas_uv = input.atlas_uv;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> FragmentOutput {
    var output: FragmentOutput;
    var color = input.color;
    // Glyph-atlas quads carry [origin_u, origin_v, tile_u_size, tile_v_size];
    // a negative origin marks a pass-through quad (sprites, debug quads).
    // aux_data.xy is the interpolated cell-local uv (v=0 at tile top).
    if (input.atlas_uv.x >= 0.0) {
        let atlas_uv = input.atlas_uv.xy + input.aux_data.xy * input.atlas_uv.zw;
        let coverage = textureSampleLevel(atlas_texture, atlas_sampler, atlas_uv, 0.0).r;
        let visible = select(0.0, 1.0, coverage >= 0.5);
        color = vec4<f32>(input.color.rgb, input.color.a * visible);
    }
    output.color = color;
    output.bus = input.bus;
    output.aux_data = input.aux_data;
    output.warble_uv = vec4<f32>(input.warble_uv.xy, 0.0, 0.0);
    return output;
}
"#;
