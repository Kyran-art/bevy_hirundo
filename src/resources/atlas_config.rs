use crate::internal_prelude::*;

#[derive(Clone, ShaderType, Debug)]
pub struct AtlasDimensions {
    pub texture_size: Vec2,
    pub cell_size: Vec2,
    pub sprite_size: Vec2,
    pub padding: Vec2,
}
