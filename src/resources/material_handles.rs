use crate::internal_prelude::*;

#[derive(Resource, Deref, DerefMut)]
pub struct VfxMaterialHandle(pub Handle<VfxMaterial>);

#[derive(Resource, Deref, DerefMut)]
pub struct VfxMeshHandle(pub Handle<Mesh>);

/// Resource holding the broadcast material handle
#[derive(Resource, Deref, DerefMut)]
pub struct VfxBroadcastMaterialHandle(pub Handle<VfxBroadcastMaterial>);
