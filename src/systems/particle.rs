//! 粒子特效系统

use bevy::prelude::*;
use crate::components::particle::{
    Particle, ParticleEmitter, EmitterConfig, EffectType,
    SpawnEffectEvent, ParticleMarker, EmitterMarker, LightningBolt
};
use crate::states::GameState;

use std::collections::HashMap;

#[derive(Resource, Clone)]
pub struct ParticleAssets {
    pub textures: HashMap<EffectType, Handle<Image>>,
    pub default_texture: Handle<Image>,
}

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnEffectEvent>();
        app.add_systems(Startup, setup_particle_texture);
        app.add_systems(
            Update,
            (
                handle_effect_events,
                update_emitters,
                update_particles,
                update_lightning_bolts,
            ).run_if(in_state(GameState::Combat)
                .or(in_state(GameState::Reward))
                .or(in_state(GameState::Tribulation))
            )
        );
    }
}

fn setup_particle_texture(mut commands: Commands, asset_server: Res<AssetServer>, mut images: ResMut<Assets<Image>>) {
    let mut textures = HashMap::new();
    textures.insert(EffectType::WanJian, asset_server.load("textures/cards/sword.png"));
    textures.insert(EffectType::WebShot, asset_server.load("textures/web_effect.png"));
    textures.insert(EffectType::SwordEnergy, asset_server.load("textures/cards/sword.png"));
    let default_texture = images.add(Image::default());
    commands.insert_resource(ParticleAssets { textures, default_texture });
}

pub fn handle_effect_events(
    mut commands: Commands, 
    assets: Res<ParticleAssets>, 
    mut events: EventReader<SpawnEffectEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in events.read() {
        if event.effect_type == EffectType::Lightning {
            spawn_real_lightning(&mut commands, &mut meshes, &mut materials, event.position);
            continue;
        }

        let config = event.effect_type.config();
        if event.burst {
            for _ in 0..event.count {
                let mut particle = config.spawn_particle(event.position, event.effect_type);
                if let Some(target) = event.target { particle.target = Some(target); }
                if let Some(target_entity) = event.target_entity { particle.target_entity = Some(target_entity); }
                if let Some(ref target_group) = event.target_group { particle.target_group = Some(target_group.clone()); }
                if let Some(target_index) = event.target_index { particle.target_index = Some(target_index); }
                spawn_particle_entity(&mut commands, &assets, particle);
            }
        } else {
            commands.spawn((
                ParticleEmitter::new(30.0, config).with_type(event.effect_type),
                Transform::from_translation(event.position),
                GlobalTransform::default(),
                EmitterMarker,
            ));
        }
    }
}

fn spawn_particle_entity(commands: &mut Commands, assets: &ParticleAssets, particle: Particle) {
    let size = particle.start_size;
    let ui_x = 640.0 + particle.position.x;
    let ui_y = 360.0 - particle.position.y;
    let handle = assets.textures.get(&particle.effect_type).unwrap_or(&assets.default_texture).clone();
    let (w, h) = if particle.effect_type == EffectType::WanJian { (size, size * 4.0) } else { (size, size) };
    let initial_rotation = if particle.effect_type == EffectType::WanJian || particle.effect_type == EffectType::SwordEnergy {
        particle.rotation - std::f32::consts::PI / 2.0
    } else {
        particle.rotation
    };

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(ui_x - w/2.0), top: Val::Px(ui_y - h/2.0),
            width: Val::Px(w), height: Val::Px(h), ..default()
        },
        ImageNode::new(handle).with_color(particle.start_color),
        ZIndex(5),
        particle,
        ParticleMarker,
        Transform::from_translation(Vec3::new(0.0, 0.0, rand::random::<f32>() * 0.01)).with_rotation(Quat::from_rotation_z(initial_rotation)),
        GlobalTransform::default(),
    ));
}

fn update_emitters(mut commands: Commands, assets: Res<ParticleAssets>, mut emitters: Query<(Entity, &mut ParticleEmitter, &GlobalTransform)>, time: Res<Time>) {
    for (entity, mut emitter, transform) in emitters.iter_mut() {
        emitter.elapsed += time.delta_secs();
        if emitter.duration > 0.0 && emitter.elapsed >= emitter.duration { commands.entity(entity).despawn_recursive(); continue; }
        emitter.timer += time.delta_secs();
        let interval = 1.0 / emitter.rate;
        while emitter.timer >= interval {
            emitter.timer -= interval;
            if emitter.emitted_count >= emitter.max_particles {
                if !emitter.looping { commands.entity(entity).despawn_recursive(); }
                break;
            }
            let particle = emitter.config.spawn_particle(transform.translation(), emitter.effect_type);
            spawn_particle_entity(&mut commands, &assets, particle);
            emitter.emitted_count += 1;
        }
    }
}

use crate::components::screen_effect::ScreenEffectEvent;
use crate::components::sprite::EnemySpriteMarker;
use bevy::core_pipeline::core_3d::Camera3d;

pub fn update_particles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Particle, &mut Node, &mut ImageNode, &mut Visibility, &mut Transform), Without<EnemySpriteMarker>>,
    time: Res<Time>,
    mut events: EventWriter<SpawnEffectEvent>,
    mut screen_events: EventWriter<ScreenEffectEvent>,
    enemy_query: Query<(Entity, &Transform), With<EnemySpriteMarker>>,
    enemy_impact_query: Query<(Entity, &crate::components::sprite::EnemySpriteMarker, &crate::components::sprite::PhysicalImpact), With<EnemySpriteMarker>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
) {
    let delta = time.delta_secs();
    for (entity, mut p, mut node, mut image, mut visibility, mut transform) in query.iter_mut() {
        p.elapsed += delta;
        let global_prog = (p.elapsed / p.lifetime).min(1.0);

        if p.effect_type == EffectType::WanJian {
            let local_prog = (global_prog * 1.6 - p.seed * 0.6).clamp(0.0, 1.0);
            if local_prog <= 0.0 { *visibility = Visibility::Hidden; continue; } else { *visibility = Visibility::Visible; }
            let hub_pos = Vec2::new(0.0, 250.0);
            if !p.position.x.is_finite() || !p.position.y.is_finite() { p.position = Vec2::new(-350.0, -80.0); }
            if !p.start_pos.x.is_finite() || !p.start_pos.y.is_finite() { p.start_pos = Vec2::new(-350.0, -80.0); }

            if local_prog < 0.2 { phase_one_the_call(&mut p, local_prog, hub_pos, &mut screen_events); }
            else if local_prog < 0.45 { phase_two_celestial_mandala(&mut p, local_prog, hub_pos); }
            else if local_prog < 0.55 { phase_three_ominous_pause(&mut p, local_prog, &mut screen_events, &enemy_query); }
            else { phase_four_mach_piercing(&mut p, local_prog, &mut events, &mut screen_events, &enemy_query, &enemy_impact_query, &camera_query); }
        } else {
            // 修复借用冲突
            let move_delta = p.velocity * delta;
            p.position += move_delta;
            let grav_delta = p.gravity * delta;
            p.velocity += grav_delta;
            
            if p.velocity.length() > 10.0 { p.rotation = (-p.velocity.y).atan2(p.velocity.x); }
            else { let rs = p.rotation_speed; p.rotation += rs * delta; }
        }

        let size = p.current_size();
        let (w, h) = if p.effect_type == EffectType::WanJian { (size * 1.83, size) } else { (size, size) };
        let ui_x = 640.0 + p.position.x;
        let ui_y = 360.0 - p.position.y;
        node.left = Val::Px(ui_x - w/2.0); node.top = Val::Px(ui_y - h/2.0);
        node.width = Val::Px(w); node.height = Val::Px(h);
        image.color = p.current_color();
        transform.rotation = Quat::from_rotation_z(p.rotation);

        if p.is_dead() { commands.entity(entity).despawn_recursive(); continue; }
    }
}

fn phase_one_the_call(p: &mut Particle, local_prog: f32, _hub_pos: Vec2, screen_events: &mut EventWriter<ScreenEffectEvent>) {
    let t = local_prog / 0.2;
    let recoil = if t < 0.2 { -0.3 * (1.0 - t * 5.0) } else { ((t - 0.2) * 5.0).exp().min(1.0) };
    let start_pos = p.start_pos + Vec2::new(0.0, -50.0);
    let target_pos = Vec2::new(-100.0 + (p.seed - 0.5) * 100.0, 300.0);
    p.position = start_pos.lerp(target_pos, t) + Vec2::new(0.0, recoil * 80.0);
    let move_dir = (target_pos - start_pos).normalize();
    p.rotation = (-move_dir.y).atan2(move_dir.x);
    if t > 0.0 && t < 0.05 && p.seed < 0.1 { screen_events.send(ScreenEffectEvent::Shake { trauma: 0.2, decay: 1.5 }); }
}

fn phase_two_celestial_mandala(p: &mut Particle, local_prog: f32, hub_pos: Vec2) {
    let t = (local_prog - 0.2) / 0.25;
    let layer = if p.seed < 0.33 { 0 } else if p.seed < 0.66 { 1 } else { 2 };
    let layer_factor = layer as f32 + 1.0;
    let angle = t * 12.0 * std::f32::consts::PI + layer_factor * 0.3 * std::f32::consts::PI + p.seed * 6.28;
    let breath = (t * 8.0 * std::f32::consts::PI).sin() * 15.0;
    let current_radius = 100.0 * layer_factor * 0.5 + breath;
    let y_offset = 150.0 * (1.0 - (t * 0.5)) * layer_factor * 0.3;
    p.position = hub_pos + Vec2::new(angle.cos() * current_radius, (angle.sin() * current_radius * 0.3) + y_offset);
    p.rotation = angle + std::f32::consts::PI / 2.0 + (t * 20.0).cos() * 0.02;
}

fn phase_three_ominous_pause(p: &mut Particle, local_prog: f32, screen_events: &mut EventWriter<ScreenEffectEvent>, enemy_query: &Query<(Entity, &Transform), With<EnemySpriteMarker>>) {
    let t = (local_prog - 0.45) / 0.1;
    if t > 0.0 && t < 0.05 && p.seed < 0.05 { screen_events.send(ScreenEffectEvent::Shake { trauma: 0.1, decay: 0.5 }); }
    if let Some(target_entity) = p.target_entity {
        if let Ok((_, transform)) = enemy_query.get(target_entity) { p.target = Some(transform.translation.truncate()); }
    }
    if t >= 0.5 && p.target.is_some() {
        let lock_progress = ((t - 0.5) * 2.0).min(1.0);
        let dir = (p.target.unwrap() - p.position).normalize();
        p.rotation = p.rotation.lerp((-dir.y).atan2(dir.x), lock_progress * 3.0);
        let glow = (lock_progress * 0.5).min(1.0);
        p.start_color = Color::srgba(1.0 + glow, 0.9 + glow * 0.5, 0.3, 1.0);
    }
}

fn phase_four_mach_piercing(p: &mut Particle, local_prog: f32, events: &mut EventWriter<SpawnEffectEvent>, _screen_events: &mut EventWriter<ScreenEffectEvent>, _enemy_query: &Query<(Entity, &Transform), With<EnemySpriteMarker>>, enemy_impact_query: &Query<(Entity, &crate::components::sprite::EnemySpriteMarker, &crate::components::sprite::PhysicalImpact), With<EnemySpriteMarker>>, camera_query: &Query<(&Camera, &GlobalTransform), With<Camera3d>>) {
    let strike_t = (local_prog - 0.55) / 0.45;
    if let Some(target_entity) = p.target_entity {
        let (camera, camera_transform) = match camera_query.get_single() { Ok(c) => c, Err(_) => return };
        if let Ok((_, _, impact)) = enemy_impact_query.get(target_entity) {
            if let Ok(screen_pos) = camera.world_to_viewport(camera_transform, impact.home_position) {
                let window_size = camera.viewport.as_ref().map(|v| v.physical_size).unwrap_or(UVec2::new(1280, 720));
                p.target = Some(Vec2::new(screen_pos.x - window_size.x as f32 / 2.0, window_size.y as f32 / 2.0 - screen_pos.y));
            }
        }
    }
    if let Some(target) = p.target {
        if p.lock_pos.is_none() { p.lock_pos = Some(p.position); }
        let lock_pos = p.lock_pos.unwrap();
        let base_dir = (target - lock_pos).normalize_or(Vec2::ZERO);
        let side_dir = Vec2::new(-base_dir.y, base_dir.x);
        let inv_t = 1.0 - strike_t;
        p.position = lock_pos * inv_t.powi(3) + (lock_pos + side_dir * (p.seed - 0.5) * 150.0) * 3.0 * inv_t.powi(2) * strike_t + (target - base_dir * 50.0 + side_dir * (p.seed - 0.5) * 30.0) * 3.0 * inv_t * strike_t.powi(2) + target * strike_t.powi(3);
        let move_dir = (target - p.position).normalize();
        p.rotation = (-move_dir.y).atan2(move_dir.x);
        if strike_t > 0.95 && p.seed < 0.2 { events.send(SpawnEffectEvent::new(EffectType::ImpactSpark, target.extend(0.0)).burst(5)); }
    }
}

// === 真实闪电系统 ===
use rand::Rng;

fn spawn_real_lightning(commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>>, target_pos: Vec3) {
    let mut rng = rand::thread_rng();
    let start_pos = Vec3::new(target_pos.x + rng.gen_range(-1.0..1.0), 10.0, target_pos.z);
    let segments = 8;
    let mut points = vec![start_pos];
    for i in 1..segments {
        let t = i as f32 / segments as f32;
        let noise = (1.0 - t) * 1.5;
        points.push(start_pos.lerp(target_pos, t) + Vec3::new(rng.gen_range(-noise..noise), 0.0, rng.gen_range(-noise..noise)));
    }
    points.push(target_pos);
    for i in 0..points.len() - 1 {
        let p1 = points[i]; let p2 = points[i+1];
        let dir = p2 - p1; let length = dir.length();
        if length < 0.01 { continue; }
        commands.spawn((Mesh3d(meshes.add(Cylinder::new(1.0, 1.0))), MeshMaterial3d(materials.add(StandardMaterial { base_color: Color::srgba(1.5, 1.5, 2.5, 1.0), emissive: LinearRgba::new(10.0, 10.0, 20.0, 1.0), ..default() })), Transform::from_translation(p1 + dir * 0.5).looking_at(p2, Vec3::Y).with_scale(Vec3::new(0.05, 0.05, length)), LightningBolt::new(points.clone(), 0.2, true), crate::components::CombatUiRoot));
    }
}

fn update_lightning_bolts(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut LightningBolt, &MeshMaterial3d<StandardMaterial>, &mut Transform)>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let mut rng = rand::thread_rng();
    for (entity, mut bolt, mat_handle, mut transform) in query.iter_mut() {
        bolt.ttl -= time.delta_secs();
        if bolt.ttl <= 0.0 { commands.entity(entity).despawn_recursive(); continue; }
        if rng.gen_bool(0.3) { bolt.alpha = (bolt.ttl / bolt.max_ttl) * 0.5; } else { bolt.alpha = bolt.ttl / bolt.max_ttl; }
        transform.scale.x *= 0.85; transform.scale.y *= 0.85;
        if let Some(mat) = materials.get_mut(mat_handle) {
            mat.base_color.set_alpha(bolt.alpha);
            let e = 10.0 * (bolt.ttl / bolt.max_ttl);
            mat.emissive = LinearRgba::new(e, e, e * 2.0, 1.0);
        }
    }
}
