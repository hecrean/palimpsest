use bevy::{
    prelude::*,
    reflect::TypeUuid,
    input::mouse::{MouseButtonInput, MouseMotion, MouseWheel},
    window::CursorMoved,
    render::{render_resource::{AsBindGroup, ShaderRef}, render_asset::RenderAssets, camera::RenderTarget, renderer::RenderQueue},

};

use crate::camera::pan_orbit_camera::OrbitCamera;



#[derive(AsBindGroup, TypeUuid, Debug, Clone, Component)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct CustomMaterial {
    #[uniform(0)]
    pub color: Color,
    #[texture(1)]
    #[sampler(2)]
    pub color_texture: Option<Handle<Image>>,
    pub alpha_mode: AlphaMode,
}

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl Material for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/custom_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}


