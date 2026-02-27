//! VFX Orchestrator (特效编排器)
//! 
//! 负责具有复杂逻辑（如万剑归宗）或非粒子形态（如 3D 闪电网格）的特效。

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
    
    // --- [程序化贴图生成] ---
    let width = 128; let height = 128;
    let mut ink_data = vec![255; width * height * 4];
    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - 63.5; let dy = y as f32 - 63.5;
            let dist = (dx*dx + dy*dy).sqrt() / 64.0;
            let angle = dy.atan2(dx);
            let noise = (angle * 4.0).sin() * 0.12 + (angle * 7.0).cos() * 0.08;
            let alpha = (1.0 - (dist / (0.95 + noise)).clamp(0.0, 1.0).powi(2)).powi(2); 
            let idx = (y * width + x) * 4;
            ink_data[idx+0]=15; ink_data[idx+1]=20; ink_data[idx+2]=25; ink_data[idx+3]=(alpha * 255.0) as u8;
        }
    }
    use bevy::render::render_asset::RenderAssetUsages;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    textures.insert(EffectType::CloudMist, images.add(Image::new(Extent3d { width: width as u32, height: height as u32, depth_or_array_layers: 1 }, TextureDimension::D2, ink_data, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default())));

    let mut scorch_data = vec![0; width * height * 4];
    for y in 0..height {
        for x in 0..width {
            let dx = (x as f32 - 64.0) / 64.0; let dy = (y as f32 - 64.0) / 64.0;
            let dist = (dx * dx + dy * dy).sqrt();
            let intensity = (1.0 - (dist + (dy.atan2(dx) * 5.0).sin() * 0.15)).powf(1.2).max(0.0);
            let i = (y * width + x) * 4;
            if intensity > 0.0 { scorch_data[i]=(intensity.powf(2.5)*40.0) as u8; scorch_data[i+1]=(intensity.powf(3.0)*15.0) as u8; scorch_data[i+2]=(intensity.powf(1.8)*70.0) as u8; scorch_data[i+3]=(intensity.powf(0.6)*230.0) as u8; }
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
            EffectType::Lightning => { spawn_real_lightning(&mut commands, &mut meshes, &mut materials, event.position, &assets); }
            EffectType::WanJian => {
                let config = event.effect_type.config();
                for _ in 0..event.count {
                    let mut p = config.spawn_particle(event.position, event.effect_type);
                    p.target = event.target_pos; p.target_entity = event.target_entity; p.target_index = Some(event.target_index);
                    p.target_group = if event.target_group.is_empty() { None } else { Some(event.target_group.clone()) };
                    p.start_pos = event.position.truncate();
                    let model = event.model_override.as_ref().or(player_assets_opt.as_ref().map(|pa| &pa.weapon));
                    if let Some(m) = model { spawn_3d_sword_particle(&mut commands, &assets, p, m); } else { spawn_vfx_entity(&mut commands, &assets, p); }
                }
            }
            _ => continue,
        }
    }
}

// === 电影级闪电生成系统 (中点位移算法) ===

fn spawn_real_lightning(commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>>, target_pos: Vec3, assets: &ParticleAssets) {
    commands.trigger(ScreenEffectEvent::impact(Vec2::new(0.0, -10.0), 0.3));
    let mut rng = rand::thread_rng();
    
    // 生成主路径
    let start_pos = Vec3::new(target_pos.x + rng.gen_range(-2.0..2.0), 15.0, target_pos.z + rng.gen_range(-1.5..1.5));
    let mut main_path = vec![start_pos, target_pos];
    
    // 递归细分主路径 (工业级 5 级细分 = 32 段)
    for _ in 0..5 {
        let mut new_path = Vec::new();
        for i in 0..main_path.len() - 1 {
            let p1 = main_path[i]; let p2 = main_path[i+1];
            let mid = p1.lerp(p2, 0.5);
            let dist = p1.distance(p2);
            
            // 核心修复：路径纠偏逻辑
            // 计算当前点到“起点-终点”中心线的垂向偏差，并强制拉回
            let ideal_line_pos = start_pos.lerp(target_pos, 1.0 - (mid.y / 15.0));
            let pull_back = (ideal_line_pos - mid) * 0.6; // 50% 纠偏力
            
            // 随机位移：主要在水平面上，且随分段缩短而减小幅度
            let offset = Vec3::new(rng.gen_range(-1.0..1.0), rng.gen_range(-0.1..0.1), rng.gen_range(-1.0..1.0)).normalize_or_zero() * dist * 0.3;
            
            new_path.push(p1);
            new_path.push(mid + offset + pull_back);
        }
        new_path.push(*main_path.last().unwrap());
        main_path = new_path;
    }

    // 处理分叉 (极其克制：只在主干上半部分生成 2 条主要侧枝)
    let mut branch_segments = Vec::new();
    for i in 2..main_path.len() / 2 {
        if rng.gen_bool(0.08) {
            let start = main_path[i];
            let dir = Vec3::new(rng.gen_range(-5.0..5.0), -2.0, rng.gen_range(-5.0..5.0)).normalize();
            let mut b_p1 = start;
            for _ in 0..4 {
                let b_p2 = b_p1 + dir * rng.gen_range(0.8..1.5) + Vec3::new(rng.gen_range(-0.5..0.5), 0.0, rng.gen_range(-0.5..0.5));
                branch_segments.push((b_p1, b_p2, 1));
                b_p1 = b_p2;
            }
        }
    }

    // 核心发光材质 (AAA 级：极亮蓝白核心)
    let trunk_mat = materials.add(StandardMaterial { 
        base_color: Color::srgba(1.0, 1.0, 1.0, 1.0),
        emissive: LinearRgba::new(1000.0, 1500.0, 5000.0, 1.0), // 极端发光
        unlit: true,
        ..default() 
    });
    let branch_mat = materials.add(StandardMaterial { 
        base_color: Color::srgba(0.4, 0.2, 0.8, 1.0),
        emissive: LinearRgba::new(50.0, 20.0, 200.0, 1.0),
        unlit: true, ..default() 
    });
    
    // 粗细升级：主干 0.035，确保在 3D 空间中像素凝聚力
    let trunk_mesh = meshes.add(Cylinder::new(0.035, 1.0));
    let branch_mesh = meshes.add(Cylinder::new(0.015, 1.0));

    // 渲染主干
    for i in 0..main_path.len() - 1 {
        let p1 = main_path[i]; let p2 = main_path[i+1];
        let dir = p2 - p1; let len = dir.length(); if len < 0.001 { continue; }
        commands.spawn((Mesh3d(trunk_mesh.clone()), MeshMaterial3d(trunk_mat.clone()), Transform { translation: p1 + dir * 0.5, rotation: Quat::from_rotation_arc(Vec3::Y, dir.normalize()), scale: Vec3::new(1.0, len, 1.0) }, LightningBolt::new(vec![], 0.3, false).with_branch_level(0), ParticleMarker, CombatUiRoot));
    }
    
    // 渲染侧枝
    for (p1, p2, _) in branch_segments {
        let dir = p2 - p1; let len = dir.length();
        commands.spawn((Mesh3d(branch_mesh.clone()), MeshMaterial3d(branch_mat.clone()), Transform { translation: p1 + dir * 0.5, rotation: Quat::from_rotation_arc(Vec3::Y, dir.normalize()), scale: Vec3::new(1.0, len, 1.0) }, LightningBolt::new(vec![], 0.15, false).with_branch_level(1), ParticleMarker, CombatUiRoot));
    }

    // 单点光源增强
    commands.spawn((PointLight { color: Color::srgba(0.6, 0.7, 1.0, 1.0), intensity: 1_200_000.0, range: 20.0, ..default() }, Transform::from_translation(target_pos + Vec3::Y * 5.0), CombatUiRoot, LightningBolt::new(vec![], 0.1, true)));
    
    // 焦痕
    let scorch = assets.textures.get(&EffectType::Lightning).cloned().unwrap_or(assets.default_texture.clone());
    commands.spawn((Mesh3d(meshes.add(Plane3d::default().mesh().size(1.0, 1.0))), MeshMaterial3d(materials.add(StandardMaterial { base_color: Color::srgba(0.1, 0.05, 0.3, 0.4), base_color_texture: Some(scorch.clone()), alpha_mode: AlphaMode::Blend, unlit: true, ..default() })), Transform::from_translation(target_pos + Vec3::new(0.0, 0.01, 0.0)).with_rotation(Quat::from_rotation_y(rand::random::<f32>()*6.28)).with_scale(Vec3::splat(3.0)), Decal::new(4.0), ParticleMarker, CombatUiRoot));
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
    for (entity, mut p, mut _node, mut visibility, mut transform) in query.iter_mut() {
        p.elapsed += delta;
        if p.effect_type == EffectType::WanJian {
            let prog = (p.elapsed / p.lifetime).min(1.0);
            let local_prog = (prog * 1.6 - p.seed * 0.6).clamp(0.0, 1.0);
            if local_prog <= 0.0 { *visibility = Visibility::Hidden; continue; } else { *visibility = Visibility::Visible; }
            if local_prog < 0.2 { phase_one_the_call(&mut p, local_prog, Vec2::new(0.0, 250.0), &mut screen_events); }
            else if local_prog < 0.45 { phase_two_celestial_mandala(&mut p, local_prog, Vec2::new(0.0, 250.0)); }
            else if local_prog < 0.55 { phase_three_ominous_pause(&mut p, local_prog, &mut screen_events, &enemy_query); }
            else { phase_four_mach_piercing(&mut p, local_prog, &mut events, &mut screen_events, &enemy_query, &enemy_impact_query, &camera_query, &mut commands); }
            sync_3d_sword_transform(&mut p, prog, &mut transform, &mut visibility);
        }
        if p.is_dead() { commands.entity(entity).despawn_recursive(); }
    }
}

fn sync_3d_sword_transform(p: &mut Particle, prog: f32, transform: &mut Transform, visibility: &mut Visibility) {
    let local_prog = (prog * 1.6 - p.seed * 0.6).clamp(0.0, 1.0);
    let prev = transform.translation;
    let next_y = if local_prog < 0.15 { 0.8 + (local_prog/0.15)*4.2 } else if local_prog < 0.6 { 5.0 + (p.elapsed*4.0).sin()*0.1 } else { 5.0 * (1.0 - ((local_prog-0.6)/0.4).powi(3)) + 0.2 };
    transform.translation = Vec3::new(p.position.x/100.0, next_y, -p.position.y/100.0);
    transform.rotation = Quat::from_rotation_arc(Vec3::Y, (transform.translation - prev).normalize_or(Vec3::Y));
    transform.scale = Vec3::splat(0.32); *visibility = if local_prog < 0.05 { Visibility::Hidden } else { Visibility::Visible };
}

fn phase_one_the_call(p: &mut Particle, lp: f32, _h: Vec2, se: &mut EventWriter<ScreenEffectEvent>) {
    let t = lp / 0.2; p.position = (p.start_pos + Vec2::new(0.0, -50.0)).lerp(Vec2::new(-100.0+(p.seed-0.5)*100.0, 300.0), t);
    if t > 0.0 && t < 0.05 && p.seed < 0.1 { se.send(ScreenEffectEvent::Shake { trauma: 0.2, decay: 1.5 }); }
}

fn phase_two_celestial_mandala(p: &mut Particle, lp: f32, hub: Vec2) {
    let t = (lp - 0.2) / 0.25; let angle = t * 37.0 + p.seed * 6.28; let r = 50.0 + (t * 25.0).sin() * 15.0;
    p.position = hub + Vec2::new(angle.cos() * r, angle.sin() * r * 0.3 + 45.0);
}

fn phase_three_ominous_pause(p: &mut Particle, lp: f32, se: &mut EventWriter<ScreenEffectEvent>, eq: &Query<(Entity, &Transform), With<EnemySpriteMarker>>) {
    if let Some(te) = p.target_entity { if let Ok((_, tr)) = eq.get(te) { p.target = Some(tr.translation.truncate()); } }
}

fn phase_four_mach_piercing(p: &mut Particle, lp: f32, ev: &mut EventWriter<SpawnEffectEvent>, _se: &mut EventWriter<ScreenEffectEvent>, _eq: &Query<(Entity, &Transform), With<EnemySpriteMarker>>, eiq: &Query<(Entity, &crate::components::sprite::EnemySpriteMarker, &crate::components::sprite::PhysicalImpact), With<EnemySpriteMarker>>, cq: &Query<(&Camera, &GlobalTransform), With<Camera3d>>, commands: &mut Commands) {
    let t = (lp - 0.55) / 0.45;
    if let Some(target) = p.target {
        if p.lock_pos.is_none() { p.lock_pos = Some(p.position); }
        let lp = p.lock_pos.unwrap(); p.position = lp.lerp(target, t.powi(2));
        if t > 0.95 && p.seed < 0.2 { ev.send(SpawnEffectEvent::new(EffectType::ImpactSpark, target.extend(0.0)).burst(5)); }
    }
}

pub fn update_lightning_bolts(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut LightningBolt, Option<&MeshMaterial3d<StandardMaterial>>, &mut Transform, Option<&mut PointLight>)>, mut materials: ResMut<Assets<StandardMaterial>>) {
    for (entity, mut bolt, mat, mut transform, mut light) in query.iter_mut() {
        bolt.ttl -= time.delta_secs(); if bolt.ttl <= 0.0 { if let Some(mut e) = commands.get_entity(entity) { e.despawn_recursive(); } continue; }
        let prog = bolt.ttl / bolt.max_ttl;
        if let Some(h) = mat { if let Some(m) = materials.get_mut(h) { m.base_color.set_alpha(prog); } }
        if let Some(mut pl) = light { pl.intensity *= 0.8; }
    }
}

pub fn update_decals(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut Decal, &MeshMaterial3d<StandardMaterial>)>, mut materials: ResMut<Assets<StandardMaterial>>) {
    for (entity, mut decal, mat) in query.iter_mut() {
        decal.ttl -= time.delta_secs(); if decal.ttl <= 0.0 { if let Some(mut e) = commands.get_entity(entity) { e.despawn_recursive(); } continue; }
        if let Some(m) = materials.get_mut(mat) { m.base_color.set_alpha((decal.ttl/decal.max_ttl).min(1.0)*0.8); }
    }
}

fn spawn_3d_sword_particle(commands: &mut Commands, _assets: &ParticleAssets, particle: Particle, model: &Handle<Scene>) -> Entity {
    commands.spawn((SceneRoot(model.clone()), Transform::from_xyz(0.0, 0.0, 0.0), particle, ParticleMarker)).id()
}

fn spawn_vfx_entity(commands: &mut Commands, assets: &ParticleAssets, particle: Particle) -> Entity {
    let handle = assets.textures.get(&particle.effect_type).unwrap_or(&assets.default_texture).clone();
    commands.spawn((Node { position_type: PositionType::Absolute, ..default() }, ImageNode::new(handle), particle, ParticleMarker)).id()
}
