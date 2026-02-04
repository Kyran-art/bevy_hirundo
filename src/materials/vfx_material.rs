use crate::internal_prelude::*;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct VfxMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
    #[storage(2, read_only)]
    pub effect_storage: Handle<ShaderStorageBuffer>,
    #[uniform(3)]
    pub atlas_dimensions: AtlasDimensions,
}

impl Material2d for VfxMaterial {
    fn vertex_shader() -> bevy::shader::ShaderRef {
        "shaders/vfx.wgsl".into()
    }
    fn fragment_shader() -> bevy::shader::ShaderRef {
        "shaders/vfx.wgsl".into()
    }
    fn alpha_mode(&self) -> bevy::sprite_render::AlphaMode2d {
        bevy::sprite_render::AlphaMode2d::Blend
    }
}
