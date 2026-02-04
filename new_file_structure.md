# Reorganized VFX Project Structure

This is your VFX Bevy project reorganized with better separation of concerns.

## Directory Structure

```
src/
├── lib.rs                      # Plugin definition and configuration
├── main.rs                     # Application entry point
├── prelude.rs                  # Public API re-exports
│
├── components/                 # ECS Components
│   ├── mod.rs
│   ├── vfx.rs                 # Main Vfx component with lifecycle hooks
│   ├── sprite_index.rs        # Sprite index tracking
│   └── markers.rs             # VfxBroadcast, VfxGhostBuffer markers
│
├── resources/                  # ECS Resources  
│   ├── mod.rs
│   ├── mesh_tag_allocator.rs # Tag allocation and recycling system
│   ├── effect_storage.rs     # GPU buffer storage for effects
│   ├── material_handles.rs   # Material resource handles
│   └── atlas_config.rs       # Texture atlas configuration
│
├── materials/                  # Bevy 2D Materials
│   ├── mod.rs
│   ├── vfx_material.rs       # Standard per-entity VFX material
│   └── broadcast_material.rs # Shared broadcast material for many entities
│
├── effects/                    # Effect System (Core VFX Logic)
│   ├── mod.rs
│   ├── lifetime.rs           # Effect lifetime and looping
│   ├── phase.rs              # Sub-effect timing windows
│   ├── wave.rs               # Wave oscillation (sine, square, etc.) + envelopes
│   ├── spatial.rs            # Spatial transforms (offset, scale, rotate, skew)
│   ├── color.rs              # Color effects with blend modes
│   ├── alpha.rs              # Alpha/transparency effects
│   ├── effect.rs             # Combined effect structure
│   ├── effect_stack.rs       # Effect stack container
│   └── builder.rs            # EffectBuilder and EffectModifier trait
│
├── systems/                    # ECS Systems
│   ├── mod.rs
│   ├── sync.rs               # Sync Vfx to internal components
│   ├── storage.rs            # Update GPU storage buffers
│   ├── pruning.rs            # Prune expired effects
│   ├── setup.rs              # Asset setup systems
│   ├── broadcast_update.rs   # Broadcast material updates
│   └── camera.rs             # Camera spawning and controls
│
├── input/                      # Input Handling
│   ├── mod.rs
│   ├── vfx_controls.rs       # Standard VFX keyboard controls
│   └── broadcast_controls.rs # Broadcast effect keyboard controls
│
├── spawners/                   # Entity Spawning Helpers
│   ├── mod.rs
│   ├── vfx_spawner.rs        # VFX entity spawning functions
│   └── broadcast_spawner.rs  # Broadcast entity spawning functions
│
└── hooks/                      # Component Lifecycle Hooks
    ├── mod.rs
    ├── hydrate.rs            # Component addition hook
    └── dehydrate.rs          # Component removal hook
```

## Key Improvements

### 1. **Separation of Concerns**
Each module has a single, clear responsibility:
- `components/` - Only ECS component definitions
- `resources/` - Only global resources
- `effects/` - Pure effect logic, no ECS
- `systems/` - ECS system functions
- `materials/` - Bevy materials
- `hooks/` - Lifecycle management

### 2. **Logical Grouping**
Related functionality is grouped together:
- All effect-related types in `effects/`
- All input handling in `input/`
- All spawning logic in `spawners/`

### 3. **Scalability**
Easy to extend:
- Add new effects → `effects/` directory
- Add new systems → `systems/` directory  
- Add new components → `components/` directory

### 4. **Discoverability**
Clear naming makes it obvious where code belongs:
- Want to modify effects? → `effects/`
- Want to add input? → `input/`
- Want to change lifecycle? → `hooks/`

### 5. **Modularity**
Clean module boundaries with explicit re-exports through `prelude.rs` for public API.

## Migration from Old Structure

### Old Files → New Locations

- `vfx.rs` → Split across `effects/` (lifetime, phase, wave, spatial, color, alpha, effect, effect_stack, builder)
- `render.rs` → Split into:
  - `components/vfx.rs` (Vfx component)
  - `resources/effect_storage.rs` (EffectStorageData)
  - `resources/atlas_config.rs` (AtlasDimensions)
  - `materials/vfx_material.rs` (VfxMaterial)
- `systems.rs` → Split into `systems/` (sync, storage, pruning, setup)
- `broadcast_material.rs` → Split into:
  - `materials/broadcast_material.rs` (Material)
  - `systems/broadcast_update.rs` (Systems)
  - `input/broadcast_controls.rs` (Input)
- `camera.rs` → `systems/camera.rs`
- `lib.rs` → Kept mostly the same, now imports from submodules
- `preludes.rs` → Renamed to `prelude.rs`, simplified

### Import Changes

Old:
```rust
use crate::internal_prelude::*;
use crate::vfx::*;
```

New:
```rust
use my_bevy_game::prelude::*;  // For users
use crate::effects::*;          // For internal use
use crate::components::*;       // For internal use
```

## Usage

The public API is unchanged! Users still interact through:

```rust
use my_bevy_game::prelude::*;

// Same API as before
commands.spawn(Vfx::new(sprite_index));

vfx.push_effect(
    EffectBuilder::one_shot(time.elapsed_secs(), 1.0)
        .color(LinearRgba::RED)
        .with(Wave::sine(1.0, 0.5, 0.5))
        .build()
);
```

## Notes

- All existing functionality is preserved
- Public API remains the same
- Internal organization is much cleaner
- Easier to navigate and maintain
- Ready for future expansion

## Next Steps

1. Copy the `src/` directory to your project root
2. Update your `Cargo.toml` if needed (should work as-is)
3. Build and test: `cargo build`
4. Enjoy your cleaner codebase!
