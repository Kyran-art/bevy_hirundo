# Simplified Plugin API - Before & After

## Before (Verbose Boilerplate)

```rust
use bevy::prelude::*;
use bevy_hirundo::{internal_prelude::*, prelude::*};

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(ImagePlugin::default_nearest()),
        VfxPlugin::default(),
        Material2dPlugin::<VfxBroadcastMaterial>::default(), // Manual
    ));
    app.add_systems(PreStartup, setup_vfx_assets);            // Already in plugin!
    app.add_systems(Startup, (spawn_unique_entities, spawn_camera));
    app.add_systems(Update, (control_2d_camera, play_fx));
    app.run();
}
```

**Issues:**
- Imports `internal_prelude` (should be private!)
- Duplicates `setup_vfx_assets` system (plugin already registers it)
- Manually adds broadcast material plugin
- Manually adds camera spawn + controls
- Verbose, multi-statement setup

## After (Minimal Boilerplate)

```rust
use bevy::prelude::*;
use bevy_hirundo::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            VfxPlugin::default().with_camera(), // Opt-in camera + controls
        ))
        .add_systems(Startup, spawn_unique_entities)
        .add_systems(Update, play_fx)  // Demo controls (optional)
        .run();
}
```

**Improvements:**
- Single import: `bevy_hirundo::prelude::*`
- No access to `internal_prelude` (properly private)
- Both unique and broadcast VFX systems always available
- Camera support via `.with_camera()` method (optional)
- Cleaner builder-style configuration
- Single expression with method chaining

## Minimal Example (No Optional Features)

```rust
use bevy::prelude::*;
use bevy_hirundo::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            VfxPlugin::default(),
        ))
        .add_systems(Startup, spawn_my_entities)
        .run();
}

fn spawn_my_entities(mut commands: Commands) {
    commands.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        Vfx::new(42), // sprite index 42
    ));
}
```

Just **4 lines** to initialize the entire VFX system!

## Custom Atlas Configuration

```rust
VfxPlugin::default()
    .with_camera()                 // Optional camera + controls
    .with_texture("my_sprites.png")
    .with_sprite_size(Vec2::new(64.0, 64.0))
    .with_cell_size(Vec2::new(72.0, 72.0))
    .with_padding(Vec2::new(4.0, 4.0))
```

## What Gets Auto-Registered Now

### Always Included (Core VFX)
- `Material2dPlugin::<VfxMaterial>` - Per-entity VFX material
- `setup_vfx_assets` - Asset initialization
- `sync_vfx_to_internal` - Component sync
- `update_effect_storage_buffer` - GPU buffer updates
- `prune_expired_effects` - Lifetime management
- Resource initialization (storage, allocator, handles)

### Opt-In: `.with_broadcast()`
- `Material2dPlugin::<VfxBroadcastMaterial>` - Shared VFX material
- `setup_broadcast_material` - Broadcast asset initialization
- `VfxBroadcastMaterialHandle` resource

### Opt-In: `.with_camera()`
- `spawn_camera` - Creates 2D camera on startup
- `control_2d_camera` - WASD pan, Z/X zoom controls

### User Prelude Exports

The `prelude` now exports only what users need:

**Components**: `Vfx`, `VfxBundle`, `VfxBroadcast`

**Effect Builders**: `EffectBuilder`, `EffectModifier`, `Effect`, `EffectStack`

**Effect Types**: `ColorEffect`, `AlphaEffect`, `SpatialEffect`

**Modifiers**: `Wave`, `WaveKind`, `Envelope`, `Phase`, `Anchor`, `BlendMode`, `CompositeMode`, `Lifetime`

**Resources**: `AtlasDimensions`, `VfxBroadcastMaterialHandle`

**Optional Systems**: `play_fx`, `control_broadcast_fx` (demo input handlers)

**Spawners**: Helper functions from `spawners` module

## Result

Users now have:
- **Minimal surface area** - Only one prelude to import
- **Zero redundancy** - No duplicate system registration
- **Opt-in features** - Broadcast and camera are optional
- **Clear ownership** - `internal_prelude` is truly internal
- **Better defaults** - Everything works out of the box
