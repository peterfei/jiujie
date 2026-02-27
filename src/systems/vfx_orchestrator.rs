//! VFX Orchestrator (特效编排器)
//! 
//! 负责具有复杂逻辑（如万剑归宗）或非粒子形态（如 3D 闪电网格）的特效。
//! 简单的粒子效果已迁移至基于 GPU 的 GpuParticlePlugin (bevy_hanabi)。

use bevy::prelude::*;
use rand::Rng;
use crate::components::particle::{
    Particle, EffectType, SpawnEffectEvent, ParticleMarker, 
    LightningBolt, Decal
};
use crate::components::screen_effect::ScreenEffectEvent;
use crate::components::combat::CombatUiRoot;
use crate::components::sprite::EnemySpriteMarker;
use crate::states::GameState;

use std::collections::HashMap;

#[derive(Resource, Clone)]
pub struct ParticleAssets {
    pub textures: HashMap<EffectType, Handle<Image>>,
    pub default_texture: Handle<Image>,
}

pub struct VfxOrchestratorPlugin;

impl Plugin for VfxOrchestratorPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnEffectEvent>();
        app.add_systems(Startup, setup_vfx_assets);
        app.add_systems(
            Update,
            (
                handle_vfx_events,
                update_vfx_logic,
                update_lightning_bolts,
                update_decals,
            ).run_if(in_state(GameState::Combat).or(in_state(GameState::MainMenu))),
        );
    }
}

fn setup_vfx_assets(mut commands: Commands, asset_server: Res<AssetServer>, mut images: ResMut<Assets<Image>>) {
    let mut textures = HashMap::new();
    textures.insert(EffectType::WanJian, asset_server.load("textures/cards/sword.png"));
    textures.insert(EffectType::WebShot, asset_server.load("textures/web_effect.png"));
    textures.insert(EffectType::SwordEnergy, asset_server.load("textures/cards/sword.png"));
    
    // --- [程序化贴图生成] 保留原有优秀算法 ---
    let width = 128;
    let height = 128;
    
    // 1. 水墨贴图
    let mut ink_data = vec![255; width * height * 4];
    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - 63.5;
            let dy = y as f32 - 63.5;
            let dist = (dx*dx + dy*dy).sqrt() / 64.0;
            let angle = dy.atan2(dx);
            let noise = (angle * 4.0).sin() * 0.12 + (angle * 7.0).cos() * 0.08;
            let threshold = 0.95 + noise;
            let normalized_dist = (dist / threshold).clamp(0.0, 1.0);
            let alpha = (1.0 - normalized_dist * normalized_dist).powi(2); 
            let idx = (y * width + x) * 4;
            ink_data[idx + 0] = 15; ink_data[idx + 1] = 20; ink_data[idx + 2] = 25; ink_data[idx + 3] = (alpha * 255.0) as u8;
        }
    }
    use bevy::render::render_asset::RenderAssetUsages;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    let cloud_image = Image::new(Extent3d { width: width as u32, height: height as u32, depth_or_array_layers: 1 }, TextureDimension::D2, ink_data, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default());
    textures.insert(EffectType::CloudMist, images.add(cloud_image));

    // 2. 雷击焦痕贴图
    let mut scorch_data = vec![0; width * height * 4];
    for y in 0..height {
        for x in 0..width {
            let dx = (x as f32 - 64.0) / 64.0; let dy = (y as f32 - 64.0) / 64.0;
            let dist = (dx * dx + dy * dy).sqrt();
            let angle = dy.atan2(dx);
            let noise = (angle * 5.0).sin() * 0.15 + (angle * 13.0).cos() * 0.07 + (angle * 27.0).sin() * 0.03;
            let intensity = (1.0 - (dist + noise)).powf(1.2).max(0.0);
            let i = (y * width + x) * 4;
            if intensity > 0.0 {
                scorch_data[i] = (intensity.powf(2.5) * 40.0) as u8; 
                scorch_data[i+1] = (intensity.powf(3.0) * 15.0) as u8; 
                scorch_data[i+2] = (intensity.powf(1.8) * 70.0) as u8; 
                scorch_data[i+3] = (intensity.powf(0.6) * 230.0) as u8; 
            }
        }
    }
    textures.insert(EffectType::Lightning, images.add(Image::new(Extent3d { width: width as u32, height: height as u32, depth_or_array_layers: 1 }, TextureDimension::D2, scorch_data, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default())));

    let default_texture = images.add(Image::default());
    commands.insert_resource(ParticleAssets { textures, default_texture });
}

pub fn handle_vfx_events(
    mut commands: Commands, 
    assets: Res<ParticleAssets>, 
    mut events: EventReader<SpawnEffectEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_assets_opt: Option<Res<crate::resources::PlayerAssets>>,
) {
    for event in events.read() {
        match event.effect_type {
            // 编排器处理类型 1: 3D 真实闪电 (涉及程序化网格和光源)
            EffectType::Lightning => {
                spawn_real_lightning(&mut commands, &mut meshes, &mut materials, event.position, &assets);
            }
            // 编排器处理类型 2: 万剑归宗 (涉及复杂四阶段状态机和多实体编排)
            EffectType::WanJian => {
                let config = event.effect_type.config();
                for _ in 0..event.count {
                    let mut p = config.spawn_particle(event.position, event.effect_type);
                    p.target = event.target_pos;
                    p.target_entity = event.target_entity;
                    p.target_group = if event.target_group.is_empty() { None } else { Some(event.target_group.clone()) };
                    p.target_index = Some(event.target_index);
                    p.start_pos = event.position.truncate();
                    
                    let model_to_use = event.model_override.as_ref().or(player_assets_opt.as_ref().map(|pa| &pa.weapon));
                    if let Some(model) = model_to_use {
                        spawn_3d_sword_particle(&mut commands, &assets, p, model);
                    } else {
                        spawn_vfx_entity(&mut commands, &assets, p);
                    }
                }
            }
            // 其余所有类型均由 GpuParticlePlugin (bevy_hanabi) 处理
            _ => continue,
        }
    }
}

fn spawn_3d_sword_particle(commands: &mut Commands, _assets: &ParticleAssets, particle: Particle, sword_model: &Handle<Scene>) -> Entity {
    let world_pos = Vec3::new(particle.position.x / 100.0, 1.0 + (particle.seed - 0.5) * 2.0, -particle.position.y / 100.0);
    commands.spawn((
        SceneRoot(sword_model.clone()),
        Transform::from_translation(world_pos).with_rotation(Quat::from_rotation_z(particle.rotation)).with_scale(Vec3::splat(1.2)),
        particle,
        ParticleMarker,
    )).id()
}

fn spawn_vfx_entity(commands: &mut Commands, assets: &ParticleAssets, particle: Particle) -> Entity {
    let ui_x = 640.0 + particle.position.x;
    let ui_y = 360.0 - particle.position.y;
    let handle = assets.textures.get(&particle.effect_type).unwrap_or(&assets.default_texture).clone();
    let initial_rotation = if particle.effect_type == EffectType::WanJian { particle.rotation - std::f32::consts::PI / 2.0 } else { particle.rotation };

    commands.spawn((
        Node { position_type: PositionType::Absolute, left: Val::Px(ui_x), top: Val::Px(ui_y), width: Val::Px(particle.start_size), height: Val::Px(particle.start_size), ..default() },
        ImageNode::new(handle).with_color(particle.start_color),
        ZIndex(5),
        particle,
        ParticleMarker,
        Transform::from_rotation(Quat::from_rotation_z(initial_rotation)),
    )).id()
}

pub fn update_vfx_logic(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Particle, Option<&mut Node>, &mut Visibility, &mut Transform), Without<EnemySpriteMarker>>,
    time: Res<Time>,
    mut events: EventWriter<SpawnEffectEvent>,
    mut screen_events: EventWriter<ScreenEffectEvent>,
    enemy_query: Query<(Entity, &Transform), With<EnemySpriteMarker>>,
    enemy_impact_query: Query<(Entity, &crate::components::sprite::EnemySpriteMarker, &crate::components::sprite::PhysicalImpact), With<EnemySpriteMarker>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
) {
    let delta = time.delta_secs();
    for (entity, mut p, mut node_opt, mut visibility, mut transform) in query.iter_mut() {
        p.elapsed += delta;
        let global_prog = (p.elapsed / p.lifetime).min(1.0);

        if p.effect_type == EffectType::WanJian {
            let local_prog = (global_prog * 1.6 - p.seed * 0.6).clamp(0.0, 1.0);
            if local_prog <= 0.0 { *visibility = Visibility::Hidden; continue; } else { *visibility = Visibility::Visible; }
            let hub_pos = Vec2::new(0.0, 250.0);

            if local_prog < 0.2 { phase_one_the_call(&mut p, local_prog, hub_pos, &mut screen_events); }
            else if local_prog < 0.45 { phase_two_celestial_mandala(&mut p, local_prog, hub_pos); }
            else if local_prog < 0.55 { phase_three_ominous_pause(&mut p, local_prog, &mut screen_events, &enemy_query); }
            else { phase_four_mach_piercing(&mut p, local_prog, &mut events, &mut screen_events, &enemy_query, &enemy_impact_query, &camera_query, &mut commands); }

            // 同步渲染 (UI 或 3D)
            let current_size = p.current_size();
            if let Some(mut node) = node_opt {
                let (w, h) = (current_size * 1.83, current_size);
                node.left = Val::Px(640.0 + p.position.x - w/2.0); node.top = Val::Px(360.0 - p.position.y - h/2.0);
                node.width = Val::Px(w); node.height = Val::Px(h);
                transform.rotation = Quat::from_rotation_z(p.rotation);
            } else {
                // 3D 路径已在相位逻辑或此处同步
                sync_3d_sword_transform(&mut p, global_prog, &mut transform, &mut visibility);
            }
        }

        if p.is_dead() { commands.entity(entity).despawn_recursive(); }
    }
}

fn sync_3d_sword_transform(p: &mut Particle, global_prog: f32, transform: &mut Transform, visibility: &mut Visibility) {
    let local_prog = (global_prog * 1.6 - p.seed * 0.6).clamp(0.0, 1.0);
    let dive_start = 0.5 + p.seed * 0.2; 
    let is_diving = local_prog > dive_start + 0.1;

    let prev_pos_3d = transform.translation;
    let mut next_pos_3d = Vec3::new(p.position.x / 100.0, 0.0, -p.position.y / 100.0);
    let sky_height = 5.0 + (p.seed - 0.5) * 1.5;
    
    next_pos_3d.y = if local_prog < 0.15 { 0.8 + (local_prog / 0.15) * (sky_height - 0.8) }
                    else if !is_diving { sky_height + (p.elapsed * 4.0 + p.seed * 30.0).sin() * 0.1 }
                    else { sky_height * (1.0 - ((local_prog - (dive_start + 0.1)) / (1.0 - (dive_start + 0.1))).powi(3)) + 0.2 };
    
    transform.translation = next_pos_3d;
    transform.scale = Vec3::splat(0.32); 
    let velocity_vec = (next_pos_3d - prev_pos_3d).normalize_or(Vec3::Y);
    transform.rotation = Quat::from_rotation_arc(Vec3::Y, velocity_vec);
    if is_diving { transform.scale.y *= 1.4; }
    *visibility = if local_prog < 0.05 { Visibility::Hidden } else { Visibility::Visible };
}

fn phase_one_the_call(p: &mut Particle, local_prog: f32, _hub_pos: Vec2, screen_events: &mut EventWriter<ScreenEffectEvent>) {
    let t = local_prog / 0.2;
    let recoil = if t < 0.2 { -0.3 * (1.0 - t * 5.0) } else { ((t - 0.2) * 5.0).exp().min(1.0) };
    let start_pos = p.start_pos + Vec2::new(0.0, -50.0);
    let target_pos = Vec2::new(-100.0 + (p.seed - 0.5) * 100.0, 300.0);
    p.position = start_pos.lerp(target_pos, t) + Vec2::new(0.0, recoil * 80.0);
    if t > 0.0 && t < 0.05 && p.seed < 0.1 { screen_events.send(ScreenEffectEvent::Shake { trauma: 0.2, decay: 1.5 }); }
}

fn phase_two_celestial_mandala(p: &mut Particle, local_prog: f32, hub_pos: Vec2) {
    let t = (local_prog - 0.2) / 0.25;
    let layer_factor = (if p.seed < 0.33 { 0.0 } else if p.seed < 0.66 { 1.0 } else { 2.0 }) + 1.0;
    let angle = t * 12.0 * std::f32::consts::PI + layer_factor * 0.3 * std::f32::consts::PI + p.seed * 6.28;
    let radius = 100.0 * layer_factor * 0.5 + (t * 8.0 * std::f32::consts::PI).sin() * 15.0;
    p.position = hub_pos + Vec2::new(angle.cos() * radius, (angle.sin() * radius * 0.3) + 150.0 * (1.0 - t * 0.5) * layer_factor * 0.3);
    p.rotation = angle + std::f32::consts::PI / 2.0;
}

fn phase_three_ominous_pause(p: &mut Particle, local_prog: f32, screen_events: &mut EventWriter<ScreenEffectEvent>, enemy_query: &Query<(Entity, &Transform), With<EnemySpriteMarker>>) {
    let t = (local_prog - 0.45) / 0.1;
    if t > 0.0 && t < 0.05 && p.seed < 0.05 { screen_events.send(ScreenEffectEvent::Shake { trauma: 0.1, decay: 0.5 }); }
    if let Some(target_entity) = p.target_entity { if let Ok((_, transform)) = enemy_query.get(target_entity) { p.target = Some(transform.translation.truncate()); } }
    if t >= 0.5 && p.target.is_some() {
        let lock_progress = ((t - 0.5) * 2.0).min(1.0);
        let dir = (p.target.unwrap() - p.position).normalize();
        p.rotation = p.rotation.lerp((-dir.y).atan2(dir.x), lock_progress * 3.0);
        p.start_color = Color::srgba(1.0 + (lock_progress * 0.5).min(1.0), 0.9, 0.3, 1.0);
    }
}

fn phase_four_mach_piercing(p: &mut Particle, local_prog: f32, events: &mut EventWriter<SpawnEffectEvent>, _screen_events: &mut EventWriter<ScreenEffectEvent>, _enemy_query: &Query<(Entity, &Transform), With<EnemySpriteMarker>>, enemy_impact_query: &Query<(Entity, &crate::components::sprite::EnemySpriteMarker, &crate::components::sprite::PhysicalImpact), With<EnemySpriteMarker>>, camera_query: &Query<(&Camera, &GlobalTransform), With<Camera3d>>, commands: &mut Commands) {
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
        let inv_t = 1.0 - strike_t;
        let base_dir = (target - lock_pos).normalize_or(Vec2::ZERO);
        let side_dir = Vec2::new(-base_dir.y, base_dir.x);
        p.position = lock_pos * inv_t.powi(3) + (lock_pos + side_dir * (p.seed - 0.5) * 150.0) * 3.0 * inv_t.powi(2) * strike_t + (target - base_dir * 50.0 + side_dir * (p.seed - 0.5) * 30.0) * 3.0 * inv_t * strike_t.powi(2) + target * strike_t.powi(3);
        p.rotation = (-(target - p.position).y).atan2((target - p.position).x);
        if strike_t > 0.95 && p.seed < 0.2 { 
            events.send(SpawnEffectEvent::new(EffectType::ImpactSpark, target.extend(0.0)).burst(5)); 
            commands.trigger(ScreenEffectEvent::impact((target - p.position).normalize_or(Vec2::ZERO) * 1.5, 0.1));
        }
    }
}

fn spawn_real_lightning(commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>>, target_pos: Vec3, assets: &ParticleAssets) {
    // 1. 物理冲击感同步
    commands.trigger(ScreenEffectEvent::impact(Vec2::new(0.0, -10.0), 0.3));
    
    let mut rng = rand::thread_rng();
    let start_pos = Vec3::new(target_pos.x + rng.gen_range(-1.0..1.0), 10.0, target_pos.z);
    let mut points = vec![start_pos];
    for i in 1..8 { 
        let t = i as f32 / 8.0; 
        points.push(start_pos.lerp(target_pos, t) + Vec3::new(rng.gen_range(-(1.0-t)*1.5..(1.0-t)*1.5), 0.0, rng.gen_range(-(1.0-t)*1.5..(1.0-t)*1.5))); 
    }
    points.push(target_pos);

    // 2. 核心性能优化：整条闪电仅生成一个中心点光源
    let center_idx = points.len() / 2;
    let mid_point = points[center_idx];
    commands.spawn((
        PointLight { 
            color: Color::srgba(0.8, 0.8, 1.0, 1.0), 
            intensity: 800_000.0, // 增强中心亮度
            range: 15.0, 
            shadows_enabled: false, 
            ..default() 
        }, 
        Transform::from_translation(mid_point), 
        CombatUiRoot, 
        LightningBolt::new(points.clone(), 0.12, false) // 仅作为光源销毁器
    ));

    // 3. 地面残痕 (Decal)
    let scorch_handle = assets.textures.get(&EffectType::Lightning).cloned().unwrap_or(assets.default_texture.clone());
    let rot = rand::random::<f32>() * 6.28; 
    let sc = 1.5 + rand::random::<f32>() * 1.5;
    
    // 底层紫色晕染
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(1.0, 1.0))), 
        MeshMaterial3d(materials.add(StandardMaterial { base_color: Color::srgba(0.3, 0.1, 0.6, 0.2), base_color_texture: Some(scorch_handle.clone()), alpha_mode: AlphaMode::Blend, unlit: true, ..default() })), 
        Transform::from_translation(target_pos + Vec3::new(0.0, 0.005, 0.0)).with_rotation(Quat::from_rotation_y(rot + 0.5)).with_scale(Vec3::splat(sc * 1.8)), 
        Decal::new(3.0), 
        ParticleMarker, 
        CombatUiRoot
    ));
    
    // 顶层核心焦痕
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(1.0, 1.0))), 
        MeshMaterial3d(materials.add(StandardMaterial { base_color_texture: Some(scorch_handle), emissive: LinearRgba::new(0.5, 0.2, 1.0, 1.0) * 0.1, alpha_mode: AlphaMode::Blend, ..default() })), 
        Transform::from_translation(target_pos + Vec3::new(0.0, 0.01, 0.0)).with_rotation(Quat::from_rotation_y(rot)).with_scale(Vec3::splat(sc)), 
        Decal::new(4.0), 
        ParticleMarker, 
        CombatUiRoot
    ));

    // 4. 生成闪电实体
    let bolt_mesh = meshes.add(Cylinder::new(0.04, 0.04)); // 稍微变细更具写意感
    let bolt_material = materials.add(StandardMaterial { 
        base_color: Color::srgba(2.0, 2.0, 3.5, 1.0), 
        emissive: LinearRgba::new(20.0, 20.0, 40.0, 1.0), 
        ..default() 
    });

    for i in 0..points.len() - 1 {
        let (p1, p2) = (points[i], points[i+1]); 
        let dir = p2 - p1; 
        let length = dir.length();
        if length < 0.01 { continue; }
        
        commands.spawn((
            Mesh3d(bolt_mesh.clone()), 
            MeshMaterial3d(bolt_material.clone()), 
            Transform::from_translation(p1 + dir * 0.5).looking_at(p2, Vec3::Y).with_scale(Vec3::new(1.0, 1.0, length)), 
            LightningBolt::new(points.clone(), 0.15, true), 
            ParticleMarker, 
            CombatUiRoot
        ));
    }
}

fn update_lightning_bolts(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut LightningBolt, Option<&MeshMaterial3d<StandardMaterial>>, &mut Transform, Option<&mut PointLight>)>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let delta = time.delta_secs();
    for (entity, mut bolt, mat, mut transform, mut light) in query.iter_mut() {
        bolt.ttl -= delta; 
        if bolt.ttl <= 0.0 { 
            commands.entity(entity).despawn_recursive(); 
            continue; 
        }
        
        // 优化闪烁算法：使用 TTL 百分比
        let progress = bolt.ttl / bolt.max_ttl;
        bolt.alpha = progress * if rand::random::<f32>() < 0.25 { 0.4 } else { 1.0 };
        
        // 整体缩窄
        transform.scale.x *= 0.88; 
        transform.scale.y *= 0.88;
        
        // 材质更新性能优化：只有作为网格时才更新材质
        if let Some(h) = mat { 
            if let Some(m) = materials.get_mut(h) { 
                m.base_color.set_alpha(bolt.alpha); 
                let e = 15.0 * progress; 
                m.emissive = LinearRgba::new(e, e, e * 2.0, 1.0); 
            } 
        }
        
        // 光源同步
        if let Some(mut pl) = light { 
            pl.intensity = 800_000.0 * progress; 
        }
    }
}

pub fn update_decals(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut Decal, &MeshMaterial3d<StandardMaterial>)>, mut materials: ResMut<Assets<StandardMaterial>>) {
    for (entity, mut decal, mat) in query.iter_mut() {
        decal.ttl -= time.delta_secs(); if decal.ttl <= 0.0 { commands.entity(entity).despawn_recursive(); continue; }
        if let Some(m) = materials.get_mut(mat) { m.base_color.set_alpha((decal.ttl / decal.max_ttl).min(1.0) * 0.8); }
    }
}
