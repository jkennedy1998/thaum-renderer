use anyhow::Result;
use thaum_renderer_domain::{composition_content_hash, Camera, IndexColorClampEffect};
use thaum_renderer_window_surface::{SharedWindowSurfaceScene, SurfaceSize, WindowSurfaceScene};

/// One frame's scene cache. Boot fingerprints everything the scene build
/// reads (composition content hash, camera, breath, surface size, config
/// post-effect knobs); on a fingerprint hit the previously built scene is
/// reused untouched, so an idle program skips quad projection, glyph-atlas
/// construction, and the per-frame GPU upload.
///
/// `hot_reload` bypasses the cache: with asset hot reloading on, font/atlas
/// content can change on disk without any fingerprint input changing.
#[derive(Default)]
pub struct BootSceneCache {
    last: Option<SceneCacheEntry>,
}



struct SceneCacheEntry {
    fingerprint: SceneFingerprint,
    scene: SharedWindowSurfaceScene,
}

/// Composition identity: a producer-maintained revision when set (O(1)
/// comparison), a full content hash otherwise.
#[derive(Debug, Clone, PartialEq)]
enum CompositionIdentity {
    Revision(u64),
    ContentHash(u64),
}

#[derive(Debug, Clone, PartialEq)]
struct SceneFingerprint {
    composition: CompositionIdentity,
    camera: Camera,
    breath: Option<i32>,
    surface_width: u32,
    surface_height: u32,
    depth_of_field_post_effect: bool,
    motion_noise_post_effect: bool,
    depth_of_field_minimum_falloff_cells: u32,
    fog_span_cells: u32,
    surface_cull_bleed_cells: u32,
    debug_texture_post_effect: bool,
    debug_warble_post_effect: bool,
    debug_depth_post_effect: bool,
    index_color_clamp: IndexColorClampEffect,
    background_color: [u64; 4],
}

impl SceneFingerprint {
    fn capture(
        state: &crate::BootState,
        surface_size: SurfaceSize,
        composition_identity: CompositionIdentity,
    ) -> Self {
        Self {
            composition: composition_identity,
            camera: state.camera,
            breath: state.data_lanes.breath(),
            surface_width: surface_size.width,
            surface_height: surface_size.height,
            depth_of_field_post_effect: state.config.depth_of_field_post_effect,
            motion_noise_post_effect: state.config.motion_noise_post_effect,
            depth_of_field_minimum_falloff_cells: state
                .config
                .depth_of_field_minimum_falloff_cells
                .to_bits(),
            fog_span_cells: state.config.fog_span_cells.to_bits(),
            surface_cull_bleed_cells: state.config.surface_cull_bleed_cells.to_bits(),
            debug_texture_post_effect: state.config.debug_texture_post_effect,
            debug_warble_post_effect: state.config.debug_warble_post_effect,
            debug_depth_post_effect: state.config.debug_depth_post_effect,
            index_color_clamp: state.config.index_color_clamp.clone(),
            background_color: state
                .config
                .window
                .clear_color
                .map(|channel| channel.to_bits()),
        }
    }
}

impl BootSceneCache {
    /// Reuses the cached scene when the fingerprint is unchanged; otherwise
    /// runs `build` once, stores the result, and returns it with a bumped
    /// scene revision so `update_scene` re-uploads it. Returns whether the
    /// scene was rebuilt.
    pub fn build_or_reuse(
        &mut self,
        state: &crate::BootState,
        surface_size: SurfaceSize,
        build: impl FnOnce() -> Result<WindowSurfaceScene>,
    ) -> Result<(SharedWindowSurfaceScene, bool)> {
        let composition_identity = if state.composition.revision != 0 {
            CompositionIdentity::Revision(state.composition.revision)
        } else {
            CompositionIdentity::ContentHash(composition_content_hash(&state.composition))
        };
        let fingerprint = SceneFingerprint::capture(state, surface_size, composition_identity);
        if state.config.hot_reload {
            let mut scene = build()?;
            self.store(&fingerprint, &mut scene);
            return Ok((std::sync::Arc::new(scene), true));
        }
        if let Some(entry) = &self.last {
            if entry.fingerprint == fingerprint {
                return Ok((entry.scene.clone(), false));
            }
        }
        let mut scene = build()?;
        self.store(&fingerprint, &mut scene);
        Ok((std::sync::Arc::new(scene), true))
    }

    fn store(&mut self, fingerprint: &SceneFingerprint, scene: &mut WindowSurfaceScene) {
        let next_revision = self
            .last
            .as_ref()
            .map(|entry| entry.scene.revision.wrapping_add(1))
            .unwrap_or(1);
        scene.revision = next_revision;
        self.last = Some(SceneCacheEntry {
            fingerprint: fingerprint.clone(),
            scene: std::sync::Arc::new(scene.clone()),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BootConfig, BootState};
    use thaum_renderer_domain::{Cell, CellGroup, CellGraphic, CellPoint, Composition, WorldPoint};
    use thaum_renderer_window_surface::{SurfaceQuad, SurfaceQuadPostEffectBus};

    fn plain_quad() -> SurfaceQuad {
        SurfaceQuad {
            center: [0.0, 0.0],
            size: [1.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
            local_uv_corners: [[0.0, 0.0]; 4],
            warble_uv_corners: [[0.0, 0.0]; 4],
            post_effect_bus: SurfaceQuadPostEffectBus::default(),
            atlas_uv: [0.0, 0.0, 1.0, 1.0],
        }
    }

    fn scene_with_quad_count(quad_count: usize) -> WindowSurfaceScene {
        let mut scene = WindowSurfaceScene::default();
        scene.quads = (0..quad_count).map(|_| plain_quad()).collect();
        scene
    }

    fn boot_state_with_composition(composition: Composition) -> BootState {
        BootState {
            camera: Camera::default(),
            composition,
            data_lanes: thaum_renderer_domain::DataLanes::with_breath(7),
            config: crate::BootConfig::default(),
            uses_fallback_breath: false,
        }
    }

    fn composition_with_one_glyph() -> Composition {
        let mut group = CellGroup::new(WorldPoint::origin());
        group.insert(Cell {
            position: CellPoint { x: 0, y: 0, z: 0 },
            graphic: CellGraphic::Glyph('A'),
            ..Cell::default()
        });
        Composition::ordered(vec![group])
    }

    #[test]
    fn unchanged_state_reuses_the_cached_scene_without_rebuilding() {
        let state = boot_state_with_composition(composition_with_one_glyph());
        let mut cache = BootSceneCache::default();
        let size = SurfaceSize {
            width: 100,
            height: 100,
        };

        let (first, rebuilt_first) =
            cache.build_or_reuse(&state, size, || Ok(scene_with_quad_count(1))).unwrap();
        let (second, rebuilt_second) =
            cache.build_or_reuse(&state, size, || Ok(scene_with_quad_count(999))).unwrap();

        assert!(rebuilt_first);
        assert!(!rebuilt_second, "identical fingerprint must not rebuild");
        assert_eq!(second.quads.len(), 1, "cached scene content, not the new build");
        assert_eq!(second.revision, first.revision);
    }

    #[test]
    fn composition_content_changes_bump_the_revision_and_rebuild() {
        let state = boot_state_with_composition(composition_with_one_glyph());
        let mut cache = BootSceneCache::default();
        let size = SurfaceSize {
            width: 100,
            height: 100,
        };

        let (_, _) = cache.build_or_reuse(&state, size, || Ok(scene_with_quad_count(1))).unwrap();

        let mut changed_composition = composition_with_one_glyph();
        changed_composition.groups[0]
            .cells
            .get_mut(&CellPoint { x: 0, y: 0, z: 0 })
            .unwrap()
            .graphic = CellGraphic::Glyph('B');
        let changed_state = boot_state_with_composition(changed_composition);
        let (scene, rebuilt) =
            cache.build_or_reuse(&changed_state, size, || Ok(scene_with_quad_count(2))).unwrap();

        assert!(rebuilt);
        assert_eq!(scene.quads.len(), 2);
    }

    #[test]
    fn camera_and_breath_changes_rebuild() {
        let state = boot_state_with_composition(composition_with_one_glyph());
        let mut cache = BootSceneCache::default();
        let size = SurfaceSize {
            width: 100,
            height: 100,
        };
        let (first, _) = cache.build_or_reuse(&state, size, || Ok(scene_with_quad_count(1))).unwrap();

        let mut moved = state.clone();
        moved.camera.position = WorldPoint { x: 1, y: 0, z: 0 };
        let (second, rebuilt_camera) =
            cache.build_or_reuse(&moved, size, || Ok(scene_with_quad_count(1))).unwrap();
        assert!(rebuilt_camera);
        assert_ne!(second.revision, first.revision);

        let mut breathed = moved.clone();
        breathed.data_lanes.set_breath(8);
        let (_, rebuilt_breath) =
            cache.build_or_reuse(&breathed, size, || Ok(scene_with_quad_count(1))).unwrap();
        assert!(rebuilt_breath);
    }

    #[test]
    fn hot_reload_bypasses_the_cache() {
        let state = boot_state_with_composition(composition_with_one_glyph());
        let mut cache = BootSceneCache::default();
        let size = SurfaceSize {
            width: 100,
            height: 100,
        };

        let (_, first_rebuilt) =
            cache.build_or_reuse(&state, size, || Ok(scene_with_quad_count(1))).unwrap();

        let mut hot = state.clone();
        hot.config.hot_reload = true;
        let (second, second_rebuilt) =
            cache.build_or_reuse(&hot, size, || Ok(scene_with_quad_count(3))).unwrap();

        assert!(first_rebuilt);
        assert!(second_rebuilt, "hot reload must rebuild every call");
        assert_eq!(second.quads.len(), 3);
    }
}
