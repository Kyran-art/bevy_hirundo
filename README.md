# bevy_hirundo 🐦‍⬛
2D visual effects for Bevy game engine.


*Note: This crate is currently under development.*


Hirundo is a wave-driven 2D VFX toolkit for Bevy, focused on expressive composition and scalable performance.


## Prerequisites
* Texture
  - The image file, containing your sprites.
  - Ideally, a *Power of Two* sized atlas (GPU's prefer it).

# Quick Start
1. Add `bevy_hirundo = "<version>"` to your `Cargo.toml`
2. Import
```rust
use bevy_hirundo::prelude::*;
```
3. Add plugin
```rust
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()), // recommended for fidelity
            HirundoPlugin::default()
                .with_texture("YourTextureName.png") // from your assets folder
                .with_atlas(AtlasDimensions { // example dimensions from the demo
                    texture_size: Vec2::new(1024.0, 1024.0), // 
                    cell_size: Vec2::new(40.0, 40.0),
                    sprite_size: Vec2::new(32.0, 32.0),
                    padding: Vec2::new(4.0, 4.0),
                }),
        ))
        .add_systems(Startup, spawn_entity_with_vfx)
        .add_systems(Update, play_effect) 
        .run();
}
```

### Use
1. Spawn an entity with a `Vfx` component
```rust
fn spawn_entity_with_vfx(mut commands: Commands) {
    commands.spawn((Transform::default(), Vfx::with_sprite(3)));
}
```
2. Query for `Vfx`
```rust
fn play_effect(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Vfx>,
)
```
3. Create, and push effect
```rust
if input.just_pressed(KeyCode::KeyP) {
    for mut vfx in &mut query {
        let pulse_effect = EffectBuilder::one_shot(time.elapsed_secs(), 1.0) // create
            .color(random_color) // rng
            .with(Wave::sine(1.0, -0.5, 0.5)) // pulses from origin to target color
            .with(BlendMode::Multiply) // darkens
            .build(); // constructs and returns an `Effect`
        vfx.push_effect(pulse_effect); // push
    }
```

### Result
![Multiplicative Color Interpolation](assets/hirundo_pulse_example.gif)

# Performance
Testing on an Iris-Xe integrated GPU, 500 entities with unique effects existed simultaneously, with reasonable and stable framerate.

Broadcasted effects are much more scalable, handling 20,000 entities easily.

Hirundo uses separate shader paths and materials for unique and broadcasted effects respectively, to avoid per-entity uniform pressure at scale.

### Philosophy and API
The builder exposes a single `.with(EffectModifier)` entry point to keep the surface area small while allowing arbitrarily deep composition.

Hirundo’s VFX are wave-driven, as most visual ideas can be expressed through combinations of wave kinds and parameters.
