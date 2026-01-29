#import bevy_render::globals::Globals;
#import bevy_sprite::mesh2d_functions;

const PI: f32 = 3.141592653589793;
const MAX_FX: u32 = 6;
const MAX_SPATIAL_FX: u32 = 3;
const MAX_COLOR_FX: u32 = 3;

struct AtlasDimensions {
    texture_size: vec2<f32>,
    cell_size: vec2<f32>,
    sprite_size: vec2<f32>,
    padding: vec2<f32>,
}

struct EffectLifetime { 
    enabled: u32, 
    looping: u32, 
    start_time: f32, 
    duration: f32 
}

struct Phase { 
    start: f32, 
    end: f32, 
    _padding: vec2<f32> 
}

struct Envelope { 
    attack: f32, 
    hold: f32, 
    release: f32, 
    growth_mode: u32, 
    growth: f32, 
    enabled: u32, 
    decay_mode: u32, 
    decay: f32 
}

struct Wave {
    kind: u32, 
    freq: f32, 
    amp: f32, 
    bias: f32, 
    phase: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
    amp_envelope: Envelope,
    freq_envelope: Envelope,
}

struct ColorEffect {
    phase: Phase,
    wave: Wave,
    color: vec4<f32>,
    blend_mode: u32,
}

struct AlphaEffect {
    phase: Phase,
    wave: Wave,
    target_alpha: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

struct SpatialEffect {
    phase: Phase,
    wave: Wave,
    manipulation: u32,
    intensity: f32,
    anchor: vec2<f32>,
}

struct Effect {
    lifetime: EffectLifetime,
    color_effects: array<ColorEffect, MAX_COLOR_FX>,
    alpha_effect: AlphaEffect,
    spatial_effects: array<SpatialEffect, MAX_SPATIAL_FX>,
}

struct EffectStack {
    sprite_index: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
    effects: array<Effect, MAX_FX>,
}

struct Varyings {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) acc_mul: vec4<f32>,
    @location(2) @interpolate(flat) acc_add: vec4<f32>,
    @location(3) @interpolate(flat) seq_mul: vec4<f32>,
    @location(4) @interpolate(flat) seq_add: vec4<f32>,
    @location(5) @interpolate(flat) hsv: vec4<f32>,
    @location(6) @interpolate(flat) atlas_uv_offset: vec2<f32>,
    @location(7) @interpolate(flat) uv_scale: vec2<f32>,
}

// Broadcast material bindings: uniform EffectStack instead of storage buffer
@group(0) @binding(1) var<uniform> globals: Globals;
@group(2) @binding(0) var texture: texture_2d<f32>;
@group(2) @binding(1) var texture_sampler: sampler;
@group(2) @binding(2) var<uniform> effect_stack: EffectStack;  // UNIFORM, not storage!
@group(2) @binding(3) var<uniform> atlas_dims: AtlasDimensions;

// Helper to calculate atlas UV offset from sprite index (accounts for padding)
fn get_atlas_uv_offset(sprite_index: u32) -> vec2<f32> {
    let sprites_per_row = floor(atlas_dims.texture_size.x / atlas_dims.cell_size.x);
    let col = f32(sprite_index) % sprites_per_row;
    let row = floor(f32(sprite_index) / sprites_per_row);
    
    let cell_step = atlas_dims.cell_size / atlas_dims.texture_size;
    let padding_uv = atlas_dims.padding / atlas_dims.texture_size;
    
    return vec2<f32>(
        col * cell_step.x + padding_uv.x,
        row * cell_step.y + padding_uv.y
    );
}

fn get_sprite_uv_scale() -> vec2<f32> {
    return atlas_dims.sprite_size / atlas_dims.texture_size;
}

fn master_lifetime(t: f32, m: EffectLifetime) -> f32 {
    if (m.enabled == 0u || m.duration <= 0.0) { return 0.0; }
    let elapsed = t - m.start_time;
    if (m.looping == 1u) { return fract(elapsed / m.duration); }
    if (elapsed < 0.0 || elapsed >= m.duration) { return 0.0; }
    return elapsed / m.duration;
}

fn phase_lifetime(t: f32, p: Phase) -> f32 {
    let s = clamp(p.start, 0.0, 1.0);
    let e = clamp(p.end, 0.0, 1.0);
    if (s >= e || t < s || t > e) { return 0.0; }
    return (t - s) / (e - s);
}

fn eval_envelope_integral(t: f32, env: Envelope) -> vec2<f32> {
    if (env.enabled == 0u) {
        return vec2<f32>(1.0, t);
    }

    let attack = env.attack;
    let hold = env.hold;
    let release = env.release;
    let total = attack + hold + release;

    if (total <= 0.0) {
        return vec2<f32>(1.0, t);
    }

    let nt = clamp(t, 0.0, 1.0) * total;
    var env_val: f32 = 1.0;
    var integral_nt: f32 = 0.0;

    if (nt <= attack) {
        let phase_t = select(0.0, nt / attack, attack > 0.0);
        
        if (env.growth_mode == 1u && abs(env.growth) > 1e-5) {
            let s = env.growth;
            env_val = (exp(phase_t * s) - 1.0) / (exp(s) - 1.0);
        } else {
            env_val = phase_t;
        }
        
        integral_nt = select(0.0, (nt * nt) / (2.0 * attack), attack > 0.0);
    } 
    else if (nt <= attack + hold) {
        env_val = 1.0;
        integral_nt = (attack * 0.5) + (nt - attack);
    } 
    else {
        let s = nt - attack - hold;
        let phase_t = select(0.0, s / release, release > 0.0);
        
        if (env.decay_mode == 1u && abs(env.decay) > 1e-5) {
            let d = env.decay;
            env_val = 1.0 - (exp(phase_t * d) - 1.0) / (exp(d) - 1.0);
        } else {
            env_val = 1.0 - phase_t;
        }
        
        integral_nt = (attack * 0.5) + hold +
                      select(0.0, s - (s * s) / (2.0 * release), release > 0.0);
    }

    let total_area = (attack * 0.5) + hold + (release * 0.5);
    let base_integral_norm = integral_nt / max(total_area, 1e-5);
    let inst = clamp(env_val, 0.0, 1.0);
    var integral_with_modulation: f32 = base_integral_norm;

    if (env.growth_mode == 1u && abs(env.growth) > 1e-5 && attack > 0.0) {
        let s = env.growth;
        let end_t = min(nt, attack);
        let integral_attack = attack * ((exp(s * end_t / attack) - 1.0) - s * end_t / attack) / (s * (exp(s) - 1.0));
        
        var full_integral = attack * 0.5;
        if (env.growth_mode == 1u && abs(env.growth) > 1e-5) {
            full_integral = attack * ((exp(s) - 1.0) - s) / (s * (exp(s) - 1.0));
        }
        
        integral_with_modulation = integral_attack / max(full_integral + hold + (release * 0.5), 1e-5);
    }
    
    if (nt > attack && nt <= attack + hold) {
        let hold_contrib = (nt - attack);
        var attack_contrib = attack * 0.5;
        if (env.growth_mode == 1u && abs(env.growth) > 1e-5) {
            let s = env.growth;
            attack_contrib = attack * ((exp(s) - 1.0) - s) / (s * (exp(s) - 1.0));
        }
        integral_with_modulation = (attack_contrib + hold_contrib) / max(attack_contrib + hold + (release * 0.5), 1e-5);
    }
    
    if (nt > attack + hold && release > 0.0) {
        let s_decay = env.decay;
        let release_t = nt - attack - hold;
        
        var attack_contrib = attack * 0.5;
        if (env.growth_mode == 1u && abs(env.growth) > 1e-5) {
            let s = env.growth;
            attack_contrib = attack * ((exp(s) - 1.0) - s) / (s * (exp(s) - 1.0));
        }
        
        var decay_integral: f32 = 0.0;
        if (env.decay_mode == 1u && abs(s_decay) > 1e-5) {
            decay_integral = release * ((exp(s_decay * release_t / release) - 1.0) - s_decay * release_t / release) / (s_decay * (exp(s_decay) - 1.0));
        } else {
            decay_integral = release_t - (release_t * release_t) / (2.0 * release);
        }
        
        var release_full_integral = release * 0.5;
        if (env.decay_mode == 1u && abs(s_decay) > 1e-5) {
            release_full_integral = release * ((exp(s_decay) - 1.0) - s_decay) / (s_decay * (exp(s_decay) - 1.0));
        }
        
        integral_with_modulation = (attack_contrib + hold + decay_integral) / max(attack_contrib + hold + release_full_integral, 1e-5);
    }

    return vec2<f32>(inst, integral_with_modulation);
}

fn eval_wave(t: f32, w: Wave) -> vec2<f32> {
    if (w.freq == 0.0 && w.amp == 0.0) {
        return vec2<f32>(0.0, 0.0);
    }

    let env_amp = eval_envelope_integral(t, w.amp_envelope);
    let modulated_amp = w.amp * env_amp.x;
    let env_freq = eval_envelope_integral(t, w.freq_envelope);
    let integral_t = env_freq.y;
    let theta_base = 2.0 * PI * w.freq * integral_t + w.phase;

    var clamped_val: f32 = 0.0;
    var raw_val: f32 = 0.0;

    switch (w.kind) {
        case 0u: {
            let raw = sin(theta_base) * modulated_amp + w.bias;
            clamped_val = clamp(raw, 0.0, 1.0);
            raw_val = raw;
        }
        case 1u: {
            let frac_theta = fract(theta_base / (2.0 * PI));
            let tri = 1.0 - 4.0 * abs(frac_theta - 0.5);
            let raw = tri * modulated_amp + w.bias;
            clamped_val = clamp(raw, 0.0, 1.0);
            raw_val = raw;
        }
        case 2u: {
            let frac_theta = fract(theta_base / (2.0 * PI));
            let sq = select(-1.0, 1.0, frac_theta < 0.5);
            let raw = sq * modulated_amp + w.bias;
            clamped_val = clamp(raw, 0.0, 1.0);
            raw_val = raw;
        }
        case 3u: {
            let frac_theta = fract(theta_base / (2.0 * PI));
            let saw = frac_theta * 2.0 - 1.0;
            let raw = saw * modulated_amp + w.bias;
            clamped_val = clamp(raw, 0.0, 1.0);
            raw_val = raw;
        }
        case 4u: {
            clamped_val = clamp(w.bias, 0.0, 1.0);
            raw_val = w.bias;
        }
        case 5u: {
            let raw = t * modulated_amp + w.bias;
            clamped_val = clamp(raw, 0.0, 1.0);
            raw_val = raw;
        }
        case 6u: {
            let raw = (t * t) * modulated_amp + w.bias;
            clamped_val = clamp(raw, 0.0, 1.0);
            raw_val = raw;
        }
        case 7u: {
            let raw = exp(t * 4.0 - 2.0) * modulated_amp + w.bias;
            clamped_val = clamp(raw, 0.0, 1.0);
            raw_val = raw;
        }
        case 8u: {
            raw_val = theta_base;
            clamped_val = clamp(raw_val, 0.0, 1.0);
        }
        default: {
            clamped_val = 0.0;
            raw_val = 0.0;
        }
    }

    return vec2<f32>(clamped_val, raw_val);
}

fn rgb_to_hsv(rgb: vec3<f32>) -> vec3<f32> {
    let cmax = max(rgb.r, max(rgb.g, rgb.b));
    let cmin = min(rgb.r, min(rgb.g, rgb.b));
    let delta = cmax - cmin;

    var h: f32 = 0.0;
    if (delta > 1e-6) {
        if (cmax == rgb.r) {
            h = ((rgb.g - rgb.b) / delta);
            if (rgb.g < rgb.b) { h += 6.0; }
        } else if (cmax == rgb.g) {
            h = ((rgb.b - rgb.r) / delta) + 2.0;
        } else {
            h = ((rgb.r - rgb.g) / delta) + 4.0;
        }
        h /= 6.0;
    }

    let s = select(0.0, delta / cmax, cmax > 1e-6);
    let v = cmax;
    return vec3<f32>(h, s, v);
}

fn hsv_to_rgb(hsv: vec3<f32>) -> vec3<f32> {
    let h = hsv.x;
    let s = hsv.y;
    let v = hsv.z;
    let c = v * s;
    let h6 = h * 6.0;
    let x = c * (1.0 - abs((h6 % 2.0) - 1.0));
    let m = v - c;

    var rgb: vec3<f32> = vec3<f32>(0.0);

    if (h6 < 1.0) {
        rgb = vec3<f32>(c, x, 0.0);
    } else if (h6 < 2.0) {
        rgb = vec3<f32>(x, c, 0.0);
    } else if (h6 < 3.0) {
        rgb = vec3<f32>(0.0, c, x);
    } else if (h6 < 4.0) {
        rgb = vec3<f32>(0.0, x, c);
    } else if (h6 < 5.0) {
        rgb = vec3<f32>(x, 0.0, c);
    } else {
        rgb = vec3<f32>(c, 0.0, x);
    }

    return rgb + m;
}

fn apply_spatial_broadcast(t: f32, pos: vec3<f32>) -> vec3<f32> {
    var p = pos.xy;

    for (var i: u32 = 0u; i < MAX_FX; i = i + 1u) {
        let eff = effect_stack.effects[i];
        if (eff.lifetime.enabled == 0u) { continue; }

        let mt = master_lifetime(t, eff.lifetime);
        if (mt == 0.0 && eff.lifetime.looping == 0u) { continue; }

        for (var j: u32 = 0u; j < MAX_SPATIAL_FX; j = j + 1u) {
            let s = eff.spatial_effects[j];
            if (s.intensity == 0.0) { continue; }

            let pt = phase_lifetime(mt, s.phase);
            if (pt == 0.0) { continue; }

            let wave = eval_wave(pt, s.wave);
            let val = wave.y * s.intensity;
            let offset = (s.anchor - vec2<f32>(0.5, 0.5)) * atlas_dims.sprite_size;
            p = p - offset;

            switch (s.manipulation) {
                case 0u: { p.x = p.x + val; }
                case 1u: { p.y = p.y + val; }
                case 2u: { p.x = p.x * (1.0 + val); }
                case 3u: { p.y = p.y * (1.0 + val); }
                case 4u: { 
                    let c = cos(val); 
                    let si = sin(val); 
                    p = vec2<f32>(p.x * c - p.y * si, p.x * si + p.y * c);
                }
                case 5u: { p.x = p.x + p.y * val; }
                case 6u: { p.y = p.y + p.x * val; }
                default: { }
            }
            p = p + offset;
        }
    }
    return vec3<f32>(p, pos.z);
}

// Broadcast vertex shader - no mesh tag, no instance indexing into storage
@vertex
fn vertex(
    @location(0) position: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @builtin(instance_index) instance_index: u32,
) -> Varyings {
    let t = globals.time;
    let spatial_pos = apply_spatial_broadcast(t, position);

    let atlas_uv_offset = get_atlas_uv_offset(effect_stack.sprite_index);
    let sprite_uv_scale = get_sprite_uv_scale();

    // Color effect processing
    var tint_acc = vec3<f32>(0.0);
    var tint_total_weight = 0.0;
    var tint_max_weight = 0.0;
    var tint_is_summed = 0.0;

    var add_acc = vec3<f32>(0.0);
    var add_total_weight = 0.0;
    var add_max_weight = 0.0;
    var add_is_summed = 0.0;

    var mult_acc = vec3<f32>(0.0);
    var mult_total_weight = 0.0;
    var mult_max_weight = 0.0;
    var mult_is_summed = 0.0;

    for (var i: u32 = 0u; i < MAX_FX; i = i + 1u) {
        let eff = effect_stack.effects[i];
        if (eff.lifetime.enabled == 0u) { continue; }

        let mt = master_lifetime(t, eff.lifetime);
        if (mt == 0.0 && eff.lifetime.looping == 0u) { continue; }

        for (var c: u32 = 0u; c < MAX_COLOR_FX; c = c + 1u) {
            let color_effect = eff.color_effects[c];
            let composite_mode = color_effect.color.w;
            if (composite_mode < 0.9) { continue; }

            let rgb_pt = phase_lifetime(mt, color_effect.phase);
            if (rgb_pt <= 0.0) { continue; }

            let rgb_wave = eval_wave(rgb_pt, color_effect.wave);
            let weight = rgb_wave.x;
            if (weight == 0.0) { continue; }

            let is_summed = step(1.9, composite_mode);

            if (color_effect.blend_mode == 0u) {
                tint_acc += color_effect.color.rgb * weight;
                tint_total_weight += weight;
                tint_max_weight = max(tint_max_weight, weight);
                tint_is_summed = max(tint_is_summed, is_summed);
            }
            else if (color_effect.blend_mode == 1u) {
                add_acc += color_effect.color.rgb * weight;
                add_total_weight += weight;
                add_max_weight = max(add_max_weight, weight);
                add_is_summed = max(add_is_summed, is_summed);
            }
            else if (color_effect.blend_mode == 2u) {
                mult_acc += color_effect.color.rgb * weight;
                mult_total_weight += weight;
                mult_max_weight = max(mult_max_weight, weight);
                mult_is_summed = max(mult_is_summed, is_summed);
            }
        }
    }

    var acc_mul_rgb = vec3<f32>(1.0);
    var acc_add_rgb = vec3<f32>(0.0);

    if (tint_total_weight > 0.001) {
        let avg_tint = tint_acc / tint_total_weight;
        let strength = saturate(mix(tint_max_weight, tint_total_weight, tint_is_summed));
        acc_mul_rgb *= (1.0 - strength);
        acc_add_rgb = acc_add_rgb * (1.0 - strength) + avg_tint * strength;
    }

    if (mult_total_weight > 0.001) {
        let avg_mult = mult_acc / mult_total_weight;
        let strength = saturate(mix(mult_max_weight, mult_total_weight, mult_is_summed));
        let k = vec3<f32>(1.0 - strength) + avg_mult * strength;
        acc_mul_rgb *= k;
        acc_add_rgb *= k;
    }

    if (add_total_weight > 0.001) {
        let avg_add = add_acc / add_total_weight;
        let strength = mix(add_max_weight, add_total_weight, add_is_summed);
        acc_add_rgb += avg_add * strength;
    }

    var seq_mul_rgb = vec3<f32>(1.0);
    var seq_add_rgb = vec3<f32>(0.0);

    var hsv_enabled = 0.0;
    var hsv_hue_delta = 0.0;
    var hsv_sat_mul = 1.0;
    var hsv_val_mul = 1.0;

    var alpha_mul = 1.0;
    var alpha_add = 0.0;

    for (var i: u32 = 0u; i < MAX_FX; i = i + 1u) {
        let eff = effect_stack.effects[i];
        if (eff.lifetime.enabled == 0u) { continue; }

        let mt = master_lifetime(t, eff.lifetime);
        if (mt == 0.0 && eff.lifetime.looping == 0u) { continue; }

        for (var c: u32 = 0u; c < MAX_COLOR_FX; c = c + 1u) {
            let color_effect = eff.color_effects[c];
            let composite_mode = color_effect.color.w;
            if (composite_mode >= 0.9) { continue; }

            let rgb_pt = phase_lifetime(mt, color_effect.phase);
            if (rgb_pt <= 0.0) { continue; }

            let rgb_wave = eval_wave(rgb_pt, color_effect.wave);
            let a_clamped = rgb_wave.x;
            let a_raw = rgb_wave.y;
            if (a_clamped == 0.0 && abs(a_raw) < 1e-6) { continue; }

            if (color_effect.blend_mode == 0u) {
                let k = (1.0 - a_clamped);
                seq_mul_rgb *= k;
                seq_add_rgb = seq_add_rgb * k + color_effect.color.rgb * a_clamped;
            }
            else if (color_effect.blend_mode == 1u) {
                seq_add_rgb += color_effect.color.rgb * a_clamped;
            }
            else if (color_effect.blend_mode == 2u) {
                let k = vec3<f32>(1.0 - a_clamped) + color_effect.color.rgb * a_clamped;
                seq_mul_rgb *= k;
                seq_add_rgb *= k;
            }
            else if (color_effect.blend_mode == 3u) {
                let b = color_effect.color.rgb * a_clamped;
                let k = vec3<f32>(1.0) - b;
                seq_mul_rgb *= k;
                seq_add_rgb = seq_add_rgb * k + b;
            }
            else if (color_effect.blend_mode == 4u) {
                hsv_enabled = 1.0;
                hsv_hue_delta += color_effect.color.r * a_raw;
                hsv_sat_mul *= (1.0 + color_effect.color.g * a_raw);
                hsv_val_mul *= (1.0 + color_effect.color.b * a_raw);
            }
        }

        let alpha_pt = phase_lifetime(mt, eff.alpha_effect.phase);
        if (alpha_pt > 0.0) {
            let alpha_wave = eval_wave(alpha_pt, eff.alpha_effect.wave);
            let a = alpha_wave.x;
            if (a > 0.0) {
                let to = saturate(eff.alpha_effect.target_alpha);
                let k = (1.0 - a);
                alpha_mul *= k;
                alpha_add = alpha_add * k + to * a;
            }
        }
    }

    let model = mesh2d_functions::get_world_from_local(instance_index);
    let clip_pos = mesh2d_functions::mesh2d_position_local_to_clip(model, vec4<f32>(spatial_pos, 1.0));

    var out: Varyings;
    out.position = clip_pos;
    out.uv = uv;
    out.acc_mul = vec4<f32>(acc_mul_rgb, alpha_mul);
    out.acc_add = vec4<f32>(acc_add_rgb, alpha_add);
    out.seq_mul = vec4<f32>(seq_mul_rgb, 0.0);
    out.seq_add = vec4<f32>(seq_add_rgb, hsv_enabled);
    out.hsv = vec4<f32>(hsv_hue_delta, hsv_sat_mul, hsv_val_mul, 0.0);
    out.atlas_uv_offset = atlas_uv_offset;
    out.uv_scale = sprite_uv_scale;

    return out;
}

@fragment
fn fragment(in: Varyings) -> @location(0) vec4<f32> {
    let atlas_uv = in.atlas_uv_offset + (in.uv * in.uv_scale);
    let sampled = textureSample(texture, texture_sampler, atlas_uv);
    let base_a = saturate(sampled.a);

    var rgb = sampled.rgb * in.acc_mul.rgb + in.acc_add.rgb;
    rgb = clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0));
    rgb = rgb * in.seq_mul.rgb + in.seq_add.rgb;

    if (in.seq_add.w > 0.5) {
        var hsv = rgb_to_hsv(clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0)));
        hsv.x = fract(hsv.x + in.hsv.x);
        hsv.y = clamp(hsv.y * in.hsv.y, 0.0, 1.0);
        hsv.z = clamp(hsv.z * in.hsv.z, 0.0, 1.0);
        rgb = hsv_to_rgb(hsv);
    }

    let cov = step(1e-4, base_a);
    let baked_alpha = saturate(base_a * in.acc_mul.w + in.acc_add.w);
    let alpha = mix(base_a, baked_alpha, cov);

    return vec4<f32>(rgb, alpha);
}
