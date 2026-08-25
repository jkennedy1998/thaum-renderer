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
    util::DeviceExt, CompositeAlphaMode, CurrentSurfaceTexture, PresentMode, SurfaceColorSpace,
    TextureFormat,
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, WindowEvent},
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
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WindowSurfaceInput {
    pub pressed_keys: Vec<KeyCode>,
    pub just_pressed_keys: Vec<KeyCode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowSurfaceFrameContext {
    pub surface_size: SurfaceSize,
    pub input: WindowSurfaceInput,
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
    run_window_surface_with_scene_provider(config, move |_| Ok(scene.clone()))
}

pub fn run_window_surface_with_scene_provider(
    config: WindowSurfaceConfig,
    mut scene_provider: impl FnMut(SurfaceSize) -> Result<WindowSurfaceScene> + 'static,
) -> Result<()> {
    run_window_surface_with_frame_provider(config, move |frame| scene_provider(frame.surface_size))
}

pub fn run_window_surface_with_frame_provider(
    config: WindowSurfaceConfig,
    scene_provider: impl FnMut(WindowSurfaceFrameContext) -> Result<WindowSurfaceScene> + 'static,
) -> Result<()> {
    let event_loop = EventLoop::new()?;
    let mut app = WindowSurfaceApp::new(config, Box::new(scene_provider));
    event_loop.run_app(&mut app)?;
    Ok(())
}

struct WindowSurfaceApp {
    config: WindowSurfaceConfig,
    scene_provider: Box<dyn FnMut(WindowSurfaceFrameContext) -> Result<WindowSurfaceScene>>,
    window: Option<Arc<Window>>,
    gpu_surface: Option<GpuSurface>,
    surface_size: SurfaceSize,
    pressed_keys: Vec<KeyCode>,
    just_pressed_keys: Vec<KeyCode>,
    performance_log: Option<PerformanceLogState>,
    pending_frame_sample: Option<PerformanceSample>,
}

impl WindowSurfaceApp {
    fn new(
        config: WindowSurfaceConfig,
        scene_provider: Box<dyn FnMut(WindowSurfaceFrameContext) -> Result<WindowSurfaceScene>>,
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
            performance_log,
            pending_frame_sample: None,
        }
    }

    fn window_attributes(&self) -> WindowAttributes {
        Window::default_attributes()
            .with_title(self.config.title.clone())
            .with_inner_size(LogicalSize::new(self.config.width, self.config.height))
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
            WindowEvent::RedrawRequested => {
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
                },
            }) {
                Ok(scene) => scene,
                Err(error) => {
                    panic!("failed to build renderer scene: {error:#}");
                }
            };
            self.just_pressed_keys.clear();
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
    _instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    internal_render_scale: f32,
    upscale_mode: WindowSurfaceUpscaleMode,
    quad_draw: Option<QuadDraw>,
}

impl GpuSurface {
    async fn new(
        window: Arc<Window>,
        scene: WindowSurfaceScene,
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
        self.quad_draw = QuadDraw::new(
            &self.device,
            &self.queue,
            self.surface_config.format,
            output_surface_size,
            internal_surface_size,
            self.upscale_mode,
            scene,
        );
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
    vertex_count: u32,
    post_bind_group: wgpu::BindGroup,
    _post_uniform_buffer: wgpu::Buffer,
    color_view: wgpu::TextureView,
    bus_view: wgpu::TextureView,
    meta_view: wgpu::TextureView,
    warble_view: wgpu::TextureView,
    _indexed_palette_view: wgpu::TextureView,
}

impl QuadDraw {
    fn new(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
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

        if !scene.indexed_color_palette.is_empty() {
            let palette_bytes: Vec<u8> = scene
                .indexed_color_palette
                .iter()
                .flat_map(|rgba| rgba.iter().copied())
                .collect();
            _queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &indexed_palette_texture,
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

        let quad_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("thaum-renderer-surface-quad-pipeline"),
            layout: None,
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
        let post_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("thaum-renderer-post-uniforms"),
            contents: bytemuck::bytes_of(&TexturePostEffectUniforms::new(
                scene,
                internal_surface_size,
                upscale_mode,
            )),
            usage: wgpu::BufferUsages::UNIFORM,
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

        let vertices: Vec<_> = scene.quads.iter().flat_map(SurfaceQuad::vertices).collect();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("thaum-renderer-surface-quad-vertices"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Some(Self {
            quad_pipeline,
            post_pipeline,
            vertex_buffer,
            vertex_count: vertices.len() as u32,
            post_bind_group,
            _post_uniform_buffer: post_uniform_buffer,
            color_view,
            bus_view,
            meta_view,
            warble_view,
            _indexed_palette_view: indexed_palette_view,
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
        }
    }

    fn push(&mut self, sample: &PerformanceSample) {
        self.frame_count += 1;
        self.quad_count_sum += sample.quad_count;
        self.vertex_count_sum += sample.vertex_count;
        self.scene_build_ms_sum += sample.scene_build_ms;
        self.scene_upload_ms_sum += sample.scene_upload_ms;
        self.render_ms_sum += sample.render_ms;
    }
}

#[derive(Debug)]
struct PerformanceLogState {
    path: PathBuf,
    bucket: Option<PerformanceLogBucket>,
}

impl PerformanceLogState {
    fn new(path: PathBuf) -> Self {
        Self { path, bucket: None }
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
        let line = format!(
            concat!(
                "{{",
                "\"timestamp_ms\":{},",
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
                "\"approx_fps\":{:.2}",
                "}}\n"
            ),
            bucket.started_at_unix_ms,
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
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) bus: vec4<f32>,
    @location(3) aux_data: vec4<f32>,
    @location(4) warble_uv: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) bus: vec4<f32>,
    @location(2) aux_data: vec4<f32>,
    @location(3) warble_uv: vec4<f32>,
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
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> FragmentOutput {
    var output: FragmentOutput;
    output.color = input.color;
    output.bus = input.bus;
    output.aux_data = input.aux_data;
    output.warble_uv = input.warble_uv;
    return output;
}
"#;
