# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Hirundo** (bevy_hirundo) is a Bevy 0.18 VFX (Visual Effects) library for 2D sprite animations. Named after the genus of swallows, Hirundo provides GPU-accelerated, wave-driven visual effects that can be applied to thousands of sprites simultaneously with minimal performance overhead.

## Build & Run Commands

```bash
# Run the main application (500 unique VFX entities demo)
cargo run

# Build for release
cargo run --release

# Run tests
cargo test

# Clean build artifacts
cargo clean
```

**Note**: The project uses `dynamic_linking` feature for faster compile times in development. Uses `rust-lld.exe` as linker on Windows for improved link speeds.

## Architecture Overview

### VFX System Architecture

The VFX system implements two rendering approaches optimized for different use cases:

#### 1. Per-Entity VFX (Storage Buffer)
- **Files**: `src/materials/vfx_material.rs`, `assets/shaders/vfx.wgsl`
- Uses GPU storage buffer indexed by `MeshTag` (unique ID per entity)
- Supports up to `MAX_VFX_ENTITIES` (500) independent entities
- Each entity can have its own sprite and effects
- Component: `Vfx` with lifecycle hooks for automatic setup/teardown
- **Performance**: Good for 100-500 entities with individual effects

#### 2. Broadcast VFX (Uniform)
- **Files**: `src/materials/broadcast_material.rs`, `assets/shaders/vfx_broadcast.wgsl`
- Uses a single shared `EffectStack` uniform across all entities
- All entities share the same effects but can have different sprites
- Component: `VfxBroadcast` marker
- **Performance**: Excellent for 10,000+ entities with synchronized effects

### Effect System Design

**Effect Stack**: Each entity (or broadcast material) has an `EffectStack` containing up to 6 simultaneous `Effect`s.

**Effect Structure** (`src/effects/effect_stack.rs`):
- `Lifetime`: Controls timing (one-shot or looping)
- Up to 3 `ColorEffect`s: RGB manipulation with blend modes (Lerp, Add, Multiply, Screen, HSV)
- 1 `AlphaEffect`: Transparency control
- Up to 3 `SpatialEffect`s: Vertex transformations (offset, scale, rotation, skew)

**Wave System** (`src/effects/wave.rs`): Each sub-effect is driven by a `Wave`:
- Types: Sine, Square, Triangle, Saw, Constant
- Parameters: frequency, amplitude, bias, phase
- Envelopes: Attack-Hold-Release modulation for both amplitude and frequency
- Each wave can have exponential growth/decay curves

**Compositing Modes** (for color effects):
- `Contributive` (default): Weighted average, capped by strongest effect
- `Additive`: Unrestricted summation of effects
- `Multiplicative`: Sequential application (output of one becomes input of next)

### Module Organization

```
src/
├── lib.rs              # VfxPlugin definition and plugin configuration
├── main.rs             # Demo application with keyboard controls
├── preludes.rs         # Module re-exports (internal vs user-facing)
│
├── effects/            # Effect System (Core VFX Logic)
│   ├── lifetime.rs     # Effect lifetime and looping
│   ├── phase.rs        # Sub-effect timing windows
│   ├── wave.rs         # Wave oscillation (sine, square, etc.) + envelopes
│   ├── envelope.rs     # Attack-Hold-Release envelopes
│   ├── spatial.rs      # Spatial transforms (offset, scale, rotate, skew)
│   ├── color.rs        # Color effects with blend modes
│   ├── alpha.rs        # Alpha/transparency effects
│   ├── effect_stack.rs # Effect and EffectStack structures
│   └── builder.rs      # EffectBuilder and EffectModifier trait
│
├── components/         # ECS Components
│   ├── vfx.rs          # Main Vfx component with lifecycle hooks
│   ├── sprite_index.rs # Sprite index tracking
│   └── markers.rs      # VfxBroadcast, VfxGhostBuffer markers
│
├── resources/          # ECS Resources
│   ├── mesh_tag_allocator.rs # Tag allocation and recycling system
│   ├── effect_storage.rs     # GPU buffer storage for effects
│   ├── material_handles.rs   # Material resource handles
│   └── atlas_config.rs       # Texture atlas configuration
│
├── materials/          # Bevy 2D Materials
│   ├── vfx_material.rs       # Standard per-entity VFX material
│   └── broadcast_material.rs # Shared broadcast material
│
├── systems/            # ECS Systems
│   ├── sync.rs         # Sync Vfx to internal components
│   ├── storage.rs      # Update GPU storage buffers
│   ├── pruning.rs      # Prune expired effects
│   ├── setup.rs        # Asset setup systems
│   ├── broadcast_update.rs # Broadcast material updates
│   └── camera.rs       # Camera spawning and controls
│
├── hooks/              # Component Lifecycle Hooks
│   ├── hydrate.rs      # Component addition hook
│   └── dehydrate.rs    # Component removal hook
│
├── input/              # Input Handling
│   ├── unique_controls.rs    # Per-entity (unique) VFX keyboard controls
│   └── broadcast_controls.rs # Broadcast effect keyboard controls
│
└── spawners/           # Entity Spawning Helpers
    ├── unique_spawner.rs     # Per-entity VFX spawning functions
    └── broadcast_spawner.rs  # Broadcast entity spawning functions
```

### Key Design Patterns

**Component Lifecycle Hooks** (`src/hooks/`):
- `Vfx` component uses `on_add` and `on_remove` hooks for automatic resource management
- Allocates `MeshTag` on spawn, recycles on despawn
- Marks GPU buffer slots as dirty for efficient updates

**Dirty Slot Tracking**:
- `EffectStorageData` maintains `HashSet<usize>` of modified slots
- GPU buffer only updated for changed entities (via `dirty_slots`)
- Hooks and `Changed<Vfx>` queries both mark slots dirty

**Prelude Pattern** (`src/preludes.rs`):
- `internal_prelude`: Full access for internal modules
- `prelude` (user-facing): Exports `Vfx`, `VfxPlugin`, effect builders, and modifiers

**Builder Pattern** (`src/effects/builder.rs`):
- `EffectBuilder` provides fluent API for creating effects
- `.with()` method accepts `EffectModifier` traits to modify the most recent sub-effect
- Spatial effects initialize with `Wave::constant()` (static transform)

## Important Constants

```rust
MAX_FX: usize = 6              // Max simultaneous effects per entity
MAX_SPATIAL_FX: usize = 3      // Max spatial effects per Effect
MAX_COLOR_FX: usize = 3        // Max color effects per Effect
MAX_VFX_ENTITIES: usize = 500  // Storage buffer capacity
```

These constants are defined in `src/preludes.rs` and mirrored in WGSL shaders.

## Performance Considerations

**Archetype Thrashing Warning**: The `Vfx` component should be added once at entity spawn and kept for the entity's lifetime. Repeatedly adding/removing `Vfx` causes archetype fragmentation and progressive performance degradation. To hide entities, use `Visibility::Hidden` or clear effects.

**Ghost Buffer Pattern**: Newly spawned `Vfx` entities are initially `Visibility::Hidden` with `VfxGhostBuffer` marker. After the first update cycle synchronizes GPU data, visibility is restored and the marker removed. This prevents rendering glitches during initialization.

**Dirty Tracking**: Only modified entities trigger GPU buffer updates. Both component lifecycle hooks and `Changed<Vfx>` queries mark slots dirty, but the system avoids double-processing.

## Atlas Configuration

The VFX system expects sprite sheet textures with uniform cell grids:
- Default: `32roguesTextureV2.png` (1024x1024, 32x32 sprites, 40x40 cells with 4px padding)
- Configure via `VfxPlugin::with_atlas()` or individual builder methods

## Demo Controls (main.rs)

- **P**: Add pulsing color effect
- **O**: Squash effect (scale manipulation)
- **I**: Change sprite index
- **U**: Rotation effect
- **Y**: Wobble/skew effect
- **C**: Clear all effects
- **R**: Blue wave (broadcast only)
- **T**: Fade in/out (broadcast only)
- **WASD**: Pan camera
- **Z/X**: Zoom in/out

## Extending the System

**Adding New Effects**:
1. Use `EffectBuilder::one_shot(now, duration)` or `::looping(now, period)`
2. Chain methods for sub-effects: `.color()`, `.alpha()`, `.offset_x()`, `.scale_y()`, `.rotate()`, etc.
3. Modify most recent sub-effect with `.with(modifier)`: `Wave`, `Phase`, `Envelope`, `Anchor`, `BlendMode`, etc.
4. Call `.build()` to construct the `Effect`
5. Push to entity's `Vfx::push_effect()` or broadcast material's `EffectStack::push()`

**Custom Blend Modes**: When adding color effects, specify blend mode with `.with(BlendMode::Add)` for additive blending, `.with(BlendMode::Multiply)` for darkening, etc.

**Wave Envelopes**: Control how wave amplitude or frequency changes over time:
```rust
.with(Envelope::amplitude(0.2, 0.0, 0.8).with_ease_out(4.0))
```

## Shader Modifications

Both shaders (`vfx.wgsl` and `vfx_broadcast.wgsl`) share nearly identical structure. The key difference:
- `vfx.wgsl`: Reads `EffectStack` from storage buffer using `@builtin(instance_index)` → `MeshTag`
- `vfx_broadcast.wgsl`: Reads single `EffectStack` from uniform (same for all instances)

When modifying effect struct layouts, update:
1. Rust types (`src/effects/`) with `#[repr(C)]` and `ShaderType` derive
2. WGSL structs in both shader files (must match Rust layout exactly)
3. Constants if adding slots to effect arrays
