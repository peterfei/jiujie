//! 粒子特效系统

use bevy::prelude::*;
use crate::components::particle::{
    Particle, ParticleEmitter, EmitterConfig, EffectType,
    SpawnEffectEvent, ParticleMarker, EmitterMarker, LightningBolt, Decal
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
                update_decals,
            ).run_if(in_state(GameState::Combat).or(in_state(GameState::MainMenu))),
        );
    }
}

fn setup_particle_texture(mut commands: Commands, asset_server: Res<AssetServer>, mut images: ResMut<Assets<Image>>) {
    let mut textures = HashMap::new();
    textures.insert(EffectType::WanJian, asset_server.load("textures/cards/sword.png"));
    textures.insert(EffectType::WebShot, asset_server.load("textures/web_effect.png"));
    textures.insert(EffectType::SwordEnergy, asset_server.load("textures/cards/sword.png"));
    
    // --- [终极优化] 程序化生成“水墨写意”晕染贴图 (消除硬心，增加蓬松感) ---
    let width = 128;
    let height = 128;
    let mut data = vec![255; width * height * 4];
    
    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - 63.5;
            let dy = y as f32 - 63.5;
            let dist = (dx*dx + dy*dy).sqrt() / 64.0;
            
            // 引入更自然的边缘扰动
            let angle = dy.atan2(dx);
            let noise = (angle * 4.0).sin() * 0.12 + (angle * 7.0).cos() * 0.08;
            let threshold = 0.95 + noise;
            
            // [核心修正] 恢复平缓衰减曲线，确保单体墨迹够大、够显眼
            let normalized_dist = (dist / threshold).clamp(0.0, 1.0);
            let alpha = (1.0 - normalized_dist * normalized_dist).powi(2); 
            
            let idx = (y * width + x) * 4;
            // 保持冷墨色调
            data[idx + 0] = 15; // 更深一点
            data[idx + 1] = 20; 
            data[idx + 2] = 25; 
            data[idx + 3] = (alpha * 255.0) as u8;
        }
    }
    use bevy::render::render_asset::RenderAssetUsages;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    let cloud_image = Image::new(
        Extent3d { width: width as u32, height: height as u32, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    textures.insert(EffectType::CloudMist, images.add(cloud_image));
    
    // 复用云雾贴图给元素特效，利用粒子颜色进行区分
    let cloud_handle = textures.get(&EffectType::CloudMist).cloned();
    if let Some(handle) = cloud_handle {
        textures.insert(EffectType::Fire, handle.clone());
        textures.insert(EffectType::Ice, handle.clone());
        textures.insert(EffectType::Poison, handle.clone());
        textures.insert(EffectType::SilkTrail, handle);
    }

    // --- [3.0 终极进化] 程序化生成“灵气烧灼”贴图 ---
    let center = width as f32 / 2.0;
    let mut scorch_data = vec![0; width * height * 4];
    for y in 0..height {
        for x in 0..width {
            let dx = (x as f32 - center) / center;
            let dy = (y as f32 - center) / center;
            let dist = (dx * dx + dy * dy).sqrt();
            
            let angle = dy.atan2(dx);
            // 叠加多层频率的噪声，模拟不规则的雷击分叉和炭化边缘
            let noise = (angle * 5.0).sin() * 0.15 
                      + (angle * 13.0).cos() * 0.07 
                      + (angle * 27.0).sin() * 0.03;
            let distorted_dist = dist + noise;
            
            let i = (y * width + x) * 4;
            // 提高对比度：中心更实，边缘更碎
            let intensity = (1.0 - distorted_dist).powf(1.2).max(0.0);
            
            if intensity > 0.0 {
                // 中心核心呈深紫黑色，向外扩散为烟熏感
                scorch_data[i] = (intensity.powf(2.5) * 40.0) as u8; 
                scorch_data[i+1] = (intensity.powf(3.0) * 15.0) as u8; 
                scorch_data[i+2] = (intensity.powf(1.8) * 70.0) as u8; 
                scorch_data[i+3] = (intensity.powf(0.6) * 230.0) as u8; 
            } else {
                scorch_data[i+3] = 0;
            }
        }
    }
    let scorch_image = Image::new(
        Extent3d { 
            width: width as u32, 
            height: height as u32, 
            depth_or_array_layers: 1 
        },
        TextureDimension::D2,
        scorch_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    textures.insert(EffectType::Lightning, images.add(scorch_image)); // 将雷击类型的贴图关联为焦痕

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
            spawn_real_lightning(&mut commands, &mut meshes, &mut materials, event.position, &assets);
            // 关键：雷击现在完全由程序化 3D 闪电和残痕负责，不再生成通用粒子点
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

fn spawn_particle_entity(commands: &mut Commands, assets: &ParticleAssets, particle: Particle) -> Entity {
    let size = particle.start_size;
    let ui_x = 640.0 + particle.position.x;
    let ui_y = 360.0 - particle.position.y;
    let handle = assets.textures.get(&particle.effect_type).unwrap_or(&assets.default_texture).clone();
    
    // [优化] 万剑归宗的长宽比在 Transform 中处理
    let initial_rotation = if particle.effect_type == EffectType::WanJian || particle.effect_type == EffectType::SwordEnergy {
        particle.rotation - std::f32::consts::PI / 2.0
    } else {
        particle.rotation
    };

    if particle.effect_type == EffectType::WanJian {
        let (w, h) = (size * 1.83, size);
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(ui_x - w/2.0), top: Val::Px(ui_y - h/2.0),
                width: Val::Px(w), height: Val::Px(h), ..default()
            },
            ImageNode::new(handle).with_color(particle.start_color),
            PickingBehavior::IGNORE,
            ZIndex(5),
            particle,
            ParticleMarker,
            Transform::from_translation(Vec3::new(0.0, 0.0, rand::random::<f32>() * 0.01)).with_rotation(Quat::from_rotation_z(initial_rotation)),
            GlobalTransform::default(),
        )).id()
    } else {
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(ui_x), top: Val::Px(ui_y),
                width: Val::Px(1.0), height: Val::Px(1.0), ..default()
            },
            ImageNode::new(handle).with_color(particle.start_color),
            PickingBehavior::IGNORE,
            ZIndex(5),
            particle,
            ParticleMarker,
            Transform::from_translation(Vec3::new(0.0, 0.0, rand::random::<f32>() * 0.01))
                .with_scale(Vec3::splat(size))
                .with_rotation(Quat::from_rotation_z(initial_rotation)),
            GlobalTransform::default(),
        )).id()
    }
}

pub fn update_emitters(

    mut commands: Commands, 

    assets: Res<ParticleAssets>, 

    mut emitters: Query<(Entity, &mut ParticleEmitter, &GlobalTransform, Option<&crate::plugins::MainMenuRoot>)>, 

    time: Res<Time>

) {

    for (entity, mut emitter, transform, in_main_menu) in emitters.iter_mut() {

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

            

                        let mut pos = transform.translation();

            

                        // [史诗级改进] 主界面云雾全屏横向播种

            

                        if emitter.effect_type == EffectType::CloudMist {

            

                            use rand::Rng;

            

                            let mut rng = rand::thread_rng();

            

                            // 横向铺满，纵向给予一定的初始抖动，使升腾更有层次感

            

                            pos.x += rng.gen_range(-800.0..800.0);

            

                            pos.y += rng.gen_range(-50.0..150.0);

            

                        }

            

                        

            

                        let particle = emitter.config.spawn_particle(pos, emitter.effect_type);

            

            

            let p_entity = spawn_particle_entity(&mut commands, &assets, particle);

            

            // 如果是在主菜单中，给粒子也打上标记以便销毁

            if in_main_menu.is_some() {

                commands.entity(p_entity).insert(crate::plugins::MainMenuRoot);

            }

            

            emitter.emitted_count += 1;

        }

    }
}

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
    for (entity, mut p, mut _node, mut image, mut visibility, mut transform) in query.iter_mut() {
        p.elapsed += delta;
        let global_prog = (p.elapsed / p.lifetime).min(1.0);

        if p.effect_type == EffectType::WanJian {
            let local_prog = (global_prog * 1.6 - p.seed * 0.6).clamp(0.0, 1.0);
            if local_prog <= 0.0 { *visibility = Visibility::Hidden; continue; } else { *visibility = Visibility::Visible; }
            let hub_pos = Vec2::new(0.0, 250.0);
            if !p.position.x.is_finite() || !p.position.y.is_finite() { p.position = Vec2::new(-350.0, -80.0); }
            if !p.start_pos.x.is_finite() || !p.start_pos.y.is_finite() { p.start_pos = Vec2::new(-350.0, -80.0); }

            if local_prog < 0.2 { 
                phase_one_the_call(&mut p, local_prog, hub_pos, &mut screen_events); 
            } else if local_prog < 0.45 { 
                phase_two_celestial_mandala(&mut p, local_prog, hub_pos); 
            } else if local_prog < 0.55 { 
                phase_three_ominous_pause(&mut p, local_prog, &mut screen_events, &enemy_query); 
            } else {
                phase_four_mach_piercing(&mut p, local_prog, &mut events, &mut screen_events, &enemy_query, &enemy_impact_query, &camera_query, &mut commands);
            }
        } else {
            // 修复借用冲突
            let move_delta = p.velocity * delta;
            p.position += move_delta;
            let grav_delta = p.gravity * delta;
            p.velocity += grav_delta;
            
            if p.velocity.length() > 10.0 { p.rotation = (-p.velocity.y).atan2(p.velocity.x); }
            else { let rs = p.rotation_speed; p.rotation += rs * delta; }
        }

        // [性能分流] 万剑归宗涉及复杂 UI 轨迹，需使用 Node 布局；普通粒子使用 Transform 优化
        let current_size = p.current_size();
        if p.effect_type == EffectType::WanJian {
            let (w, h) = (current_size * 1.83, current_size); // 还原经典长宽比
            let ui_x = 640.0 + p.position.x;
            let ui_y = 360.0 - p.position.y;
            _node.left = Val::Px(ui_x - w/2.0); 
            _node.top = Val::Px(ui_y - h/2.0);
            _node.width = Val::Px(w); 
            _node.height = Val::Px(h);
            
            transform.scale = Vec3::ONE; // 缩放已由 Node 处理
            // 关键修复：不要放在 ZERO，防止在 3D 空间中心产生亮点残留
            transform.translation = Vec3::new(-5000.0, -5000.0, -10.0); 
        } else {
            let ui_x = 640.0 + p.position.x;
            let ui_y = 360.0 - p.position.y;
            transform.translation.x = ui_x;
            transform.translation.y = ui_y;
            transform.scale = Vec3::splat(current_size);
        }
        
        transform.rotation = Quat::from_rotation_z(p.rotation);
        
        // 针对 CloudMist 优化平滑淡入淡出
        if p.effect_type == EffectType::CloudMist {
            let mut color = p.current_color();
            let fade = (global_prog * (1.0 - global_prog) * 4.0).clamp(0.0, 1.0);
            // [黄金平衡] 提升透明度，找回水墨厚度
            color.set_alpha(0.28 * fade); 
            image.color = color;
            
            // 增加更显著的旋转扰动 (水墨流变感)
            p.rotation += delta * (p.seed - 0.5) * 0.25;
        } else if p.effect_type == EffectType::SilkTrail {
            let mut color = p.current_color();
            // 丝迹随着生命值降低快速变透明
            color.set_alpha(0.5 * (1.0 - global_prog));
            image.color = color;
            
            // 特殊拉伸效果：模拟丝线的长条感
            transform.scale.x = current_size * 2.5; 
            transform.scale.y = current_size * 0.4;
        } else {
            image.color = p.current_color();
        }

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

fn phase_four_mach_piercing(
    p: &mut Particle, 
    local_prog: f32, 
    events: &mut EventWriter<SpawnEffectEvent>, 
    _screen_events: &mut EventWriter<ScreenEffectEvent>, 
    _enemy_query: &Query<(Entity, &Transform), With<EnemySpriteMarker>>, 
    enemy_impact_query: &Query<(Entity, &crate::components::sprite::EnemySpriteMarker, &crate::components::sprite::PhysicalImpact), With<EnemySpriteMarker>>, 
    camera_query: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    commands: &mut Commands,
) {
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
        if strike_t > 0.95 && p.seed < 0.2 { 
            events.send(SpawnEffectEvent::new(EffectType::ImpactSpark, target.extend(0.0)).burst(5)); 
            // 产生微弱的屏幕冲量
            let impulse_dir = (target - p.position).normalize_or(Vec2::ZERO);
            commands.trigger(ScreenEffectEvent::impact(impulse_dir * 1.5, 0.1));
        }
    }
}

// === 真实闪电系统 ===
use rand::Rng;

fn spawn_real_lightning(
    commands: &mut Commands, 
    meshes: &mut ResMut<Assets<Mesh>>, 
    materials: &mut ResMut<Assets<StandardMaterial>>, 
    target_pos: Vec3,
    assets: &ParticleAssets,
) {
    // 发送物理冲击事件：雷击向下冲击
    commands.trigger(ScreenEffectEvent::impact(Vec2::new(0.0, -10.0), 0.3));

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
        
        // 为每段闪电添加一个点光源（或者在中心点添加一个大光源）
        // 这里选择在闪电的中段位置注入光源
        if i == segments / 2 {
            commands.spawn((
                PointLight {
                    color: Color::srgba(0.8, 0.8, 1.0, 1.0),
                    intensity: 500_000.0, // 高亮度瞬时闪光
                    range: 10.0,
                    shadows_enabled: false, // 性能优化：动态闪电不开启阴影
                    ..default()
                },
                Transform::from_translation(p1 + dir * 0.5),
                CombatUiRoot, // 确保随战斗结束清理
                LightningBolt::new(points.clone(), 0.1, false), // 借用 LightningBolt 逻辑进行自动销毁
            ));

            // 在击地点生成复合式残痕 (底层晕染 + 顶层焦核)
            let scorch_handle = assets.textures.get(&EffectType::Lightning).cloned().unwrap_or(assets.default_texture.clone());
            let random_rotation = rand::random::<f32>() * std::f32::consts::TAU;
            let random_scale = 1.5 + rand::random::<f32>() * 1.5;

            // 1. 底层能量场 (大而淡，使用纹理遮罩)
            commands.spawn((
                Mesh3d(meshes.add(Plane3d::default().mesh().size(1.0, 1.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgba(0.3, 0.1, 0.6, 0.25), // 更暗、更透明的紫色
                    base_color_texture: Some(scorch_handle.clone()), // 关键：使用同样的炸裂形状
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    ..default()
                })),
                Transform::from_translation(target_pos + Vec3::new(0.0, 0.005, 0.0))
                    .with_rotation(Quat::from_rotation_y(random_rotation + 0.5)) // 稍微错开角度增加层次
                    .with_scale(Vec3::splat(random_scale * 1.6)),
                crate::components::particle::Decal::new(3.5),
                crate::components::particle::ParticleMarker,
                CombatUiRoot,
            ));

            // 2. 顶层核心焦痕 (小而深)
            commands.spawn((
                Mesh3d(meshes.add(Plane3d::default().mesh().size(1.0, 1.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color_texture: Some(scorch_handle),
                    emissive: LinearRgba::new(0.5, 0.2, 1.0, 1.0) * 0.1,
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                })),
                Transform::from_translation(target_pos + Vec3::new(0.0, 0.01, 0.0)) // 稍微高一点
                    .with_rotation(Quat::from_rotation_y(random_rotation))
                    .with_scale(Vec3::splat(random_scale)),
                crate::components::particle::Decal::new(4.5), 
                crate::components::particle::ParticleMarker,
                CombatUiRoot,
            ));
        }

        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(1.0, 1.0))), 
            MeshMaterial3d(materials.add(StandardMaterial { 
                base_color: Color::srgba(1.5, 1.5, 2.5, 1.0), 
                emissive: LinearRgba::new(10.0, 10.0, 20.0, 1.0), 
                ..default() 
            })), 
            Transform::from_translation(p1 + dir * 0.5).looking_at(p2, Vec3::Y).with_scale(Vec3::new(0.05, 0.05, length)), 
            Visibility::Visible,
            InheritedVisibility::VISIBLE,
            LightningBolt::new(points.clone(), 0.2, true), 
            crate::components::particle::ParticleMarker,
            CombatUiRoot
        ));
    }
}

fn update_lightning_bolts(
    mut commands: Commands, 
    time: Res<Time>, 
    mut query: Query<(Entity, &mut LightningBolt, Option<&MeshMaterial3d<StandardMaterial>>, &mut Transform, Option<&mut PointLight>)>, 
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    let mut rng = rand::thread_rng();
    for (entity, mut bolt, mat_handle, mut transform, mut light) in query.iter_mut() {
        bolt.ttl -= time.delta_secs();
        if bolt.ttl <= 0.0 { commands.entity(entity).despawn_recursive(); continue; }
        
        if rng.gen_bool(0.3) { bolt.alpha = (bolt.ttl / bolt.max_ttl) * 0.5; } else { bolt.alpha = bolt.ttl / bolt.max_ttl; }
        transform.scale.x *= 0.85; transform.scale.y *= 0.85;
        
        // 同步材质透明度和发光
        if let Some(handle) = mat_handle {
            if let Some(mat) = materials.get_mut(handle) {
                mat.base_color.set_alpha(bolt.alpha);
                let e = 10.0 * (bolt.ttl / bolt.max_ttl);
                mat.emissive = LinearRgba::new(e, e, e * 2.0, 1.0);
            }
        }

        // 同步光源强度
        if let Some(mut pl) = light {
            pl.intensity = 500_000.0 * (bolt.ttl / bolt.max_ttl);
        }
    }
}

pub fn update_decals(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut crate::components::particle::Decal, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut decal, mat_handle) in query.iter_mut() {
        decal.ttl -= time.delta_secs();
        if decal.ttl <= 0.0 {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        // 随时间淡出
        if let Some(mat) = materials.get_mut(mat_handle) {
            let alpha = (decal.ttl / decal.max_ttl).min(1.0);
            mat.base_color.set_alpha(alpha * 0.8);
        }
    }
}
