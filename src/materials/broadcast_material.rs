use crate::internal_prelude::*;

/// Material for broadcasting a single shared EffectStack to many entities.
/// Unlike VfxMaterial which uses a storage buffer indexed by mesh tag,
/// this material holds one EffectStack as a uniform that all instances share.
///
/// Use this when you want 10,000+ entities to animate with the same effect,
/// achieving better performance through uniform memory access patterns.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct VfxBroadcastMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,

    #[uniform(2)]
    pub effect_stack: EffectStack,

    #[uniform(3)]
    pub atlas_dimensions: AtlasDimensions,
}

impl Material2d for VfxBroadcastMaterial {
    fn vertex_shader() -> bevy::shader::ShaderRef {
        "shaders/vfx_broadcast.wgsl".into()
    }
    fn fragment_shader() -> bevy::shader::ShaderRef {
        "shaders/vfx_broadcast.wgsl".into()
    }
    fn alpha_mode(&self) -> bevy::sprite_render::AlphaMode2d {
        bevy::sprite_render::AlphaMode2d::Blend
    }
}
