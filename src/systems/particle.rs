//! 粒子特效系统

use bevy::prelude::*;
use crate::components::particle::{
    Particle, ParticleEmitter, EmitterConfig, EffectType,
    SpawnEffectEvent, ParticleMarker, EmitterMarker
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
            ).run_if(in_state(GameState::Combat)
                .or(in_state(GameState::Reward))
                .or(in_state(GameState::Tribulation))
            )
        );
    }
}

fn setup_particle_texture(mut commands: Commands, asset_server: Res<AssetServer>, mut images: ResMut<Assets<Image>>) {
    let mut textures = HashMap::new();
    
    // 加载专属贴图
    textures.insert(EffectType::WanJian, asset_server.load("textures/cards/sword.png"));
    textures.insert(EffectType::WebShot, asset_server.load("textures/web_effect.png"));
    textures.insert(EffectType::SwordEnergy, asset_server.load("textures/cards/sword.png"));
    
    // 默认贴图（1x1 白色）
    let default_texture = images.add(Image::default());
    
    commands.insert_resource(ParticleAssets { 
        textures,
        default_texture,
    });
}

fn handle_effect_events(mut commands: Commands, assets: Res<ParticleAssets>, mut events: EventReader<SpawnEffectEvent>) {
    for event in events.read() {
        let config = event.effect_type.config();
        if event.burst {
            for _ in 0..event.count {
                let mut particle = config.spawn_particle(event.position, event.effect_type);
                if let Some(target) = event.target {
                    particle.target = Some(target);
                }
                if let Some(target_entity) = event.target_entity {
                    particle.target_entity = Some(target_entity);
                }
                if let Some(ref target_group) = event.target_group {
                    particle.target_group = Some(target_group.clone());
                }
                if let Some(target_index) = event.target_index {
                    particle.target_index = Some(target_index);
                }
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

    let (w, h) = if particle.effect_type == EffectType::WanJian {
        (size, size * 4.0)
    } else {
        (size, size)
    };

    // 计算初始旋转
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
        // 添加 Transform 和 GlobalTransform 以支持 2D 旋转
        Transform::from_rotation(Quat::from_rotation_z(initial_rotation)),
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
            // --- 大作级：万剑归宗 · 诛仙剑阵：四相位终极视觉方案 ---
            // local_prog 决定了飞剑启动和结束的先后顺序，形成龙身
            let local_prog = (global_prog * 1.6 - p.seed * 0.6).clamp(0.0, 1.0);

            if local_prog <= 0.0 {
                *visibility = Visibility::Hidden;
                continue;
            } else {
                *visibility = Visibility::Visible;
            }

            // 定义半空集结点 (Hub)
            let hub_pos = Vec2::new(0.0, 250.0);

            // 防护：确保 position 和 start_pos 有效（防止 NaN）
            if !p.position.x.is_finite() || !p.position.y.is_finite() {
                p.position = Vec2::new(-350.0, -80.0); // 玩家默认位置
            }
            if !p.start_pos.x.is_finite() || !p.start_pos.y.is_finite() {
                p.start_pos = Vec2::new(-350.0, -80.0);
            }

            // 四相位划分
            if local_prog < 0.2 {
                // 第一相位：万剑齐鸣 (The Call) - 0% ~ 20%
                phase_one_the_call(&mut p, local_prog, hub_pos, &mut screen_events);
            } else if local_prog < 0.45 {
                // 第二相位：八卦剑轮 (Celestial Mandala) - 20% ~ 45%
                phase_two_celestial_mandala(&mut p, local_prog, hub_pos);
            } else if local_prog < 0.55 {
                // 第三相位：瞬狱锁定 (Ominous Pause) - 45% ~ 55%
                phase_three_ominous_pause(&mut p, local_prog, &mut screen_events, &enemy_query);
            } else {
                // 第四相位：极速穿心 (Mach Piercing) - 55% ~ 100%
                phase_four_mach_piercing(&mut p, local_prog, &mut events, &mut screen_events, &enemy_query, &enemy_impact_query, &camera_query);
            }
        } else {
            // --- 通用粒子逻辑 ---
            let current_v = p.velocity;
            let grav = p.gravity;
            p.position += current_v * delta;
            p.velocity += grav * delta;
            
            let v_len = p.velocity.length();
            if v_len > 10.0 {
                p.rotation = (-p.velocity.y).atan2(p.velocity.x);
            } else {
                let rs = p.rotation_speed;
                p.rotation += rs * delta;
            }
        }

        let size = p.current_size();
        let (w, h) = if p.effect_type == EffectType::WanJian {
            // sword.png 原始尺寸 1408×768，宽高比 ~1.83
            (size * 1.83, size)
        } else {
            (size, size)
        };
        let ui_x = 640.0 + p.position.x;
        let ui_y = 360.0 - p.position.y;
        node.left = Val::Px(ui_x - w/2.0); node.top = Val::Px(ui_y - h/2.0);
        node.width = Val::Px(w); node.height = Val::Px(h);

        // 6. 应用视觉状态
        image.color = p.current_color();

        // 更新 2D 旋转（Transform 组件）
        // sword.png 是横向贴图 (1408×768)，直接使用计算出的旋转角度
        let final_rotation = p.rotation;
        transform.rotation = Quat::from_rotation_z(final_rotation);

        // 7. 死亡检查与销毁 (放最后且确保逻辑闭环)
        if p.is_dead() {
            commands.entity(entity).despawn_recursive();
            continue; // 彻底跳过当前实体后续逻辑，防止 Panic
        }
    }
}

// =============================================================================
// 万剑归宗 · 诛仙剑阵：四相位终极视觉方案
// =============================================================================

/// 第一相位：万剑齐鸣 (The Call) - 0% ~ 20%
/// 视觉：飞剑从虚空中"撕裂"而出，斜插向天际
/// 动感：强烈后坐力（先沉一下再极速弹射）
fn phase_one_the_call(
    p: &mut Particle,
    local_prog: f32,
    hub_pos: Vec2,
    screen_events: &mut EventWriter<ScreenEffectEvent>,
) {
    let t = local_prog / 0.2;

    // 后坐力函数：先下沉再极速弹射
    let recoil = if t < 0.2 {
        // 下沉阶段
        -0.3 * (1.0 - t * 5.0)
    } else {
        // 弹射阶段
        ((t - 0.2) * 5.0).exp().min(1.0)
    };

    // 从脚下深位置斜向上飞向天际
    let start_pos = p.start_pos + Vec2::new(0.0, -50.0);
    let target_pos = Vec2::new(-100.0 + (p.seed - 0.5) * 100.0, 300.0);

    let base_pos = start_pos.lerp(target_pos, t);
    p.position = base_pos + Vec2::new(0.0, recoil * 80.0);

    // 旋转朝向运动方向
    let move_dir = (target_pos - start_pos).normalize();
    p.rotation = (-move_dir.y).atan2(move_dir.x);

    // 触发轻微震动（前 5% 的时间）
    if t > 0.0 && t < 0.05 && p.seed < 0.1 {
        screen_events.send(ScreenEffectEvent::Shake { trauma: 0.2, decay: 1.5 });
    }
}

/// 第二相位：八卦剑轮 (Celestial Mandala) - 20% ~ 45%
/// 视觉：立体多层圆锥形剑阵
/// 动感：像鱼群"呼吸"颤动，剑身嗡鸣
fn phase_two_celestial_mandala(
    p: &mut Particle,
    local_prog: f32,
    hub_pos: Vec2,
) {
    let t = (local_prog - 0.2) / 0.25;

    // 三层圆锥结构（根据种子分配层级）
    let layer = if p.seed < 0.33 { 0 } else if p.seed < 0.66 { 1 } else { 2 };
    let layer_factor = layer as f32 + 1.0;

    // 螺旋上升轨迹
    let angle_base = t * 12.0 * std::f32::consts::PI;
    let layer_offset = layer_factor * 0.3 * std::f32::consts::PI;
    let angle = angle_base + layer_offset + p.seed * 6.28;

    // 呼吸颤动
    let breath = (t * 8.0 * std::f32::consts::PI).sin() * 15.0;
    let base_radius = 100.0 * layer_factor * 0.5;
    let current_radius = base_radius + breath;

    // 圆锥高度分布
    let cone_height = 150.0;
    let h_factor = 1.0 - (t * 0.5);
    let y_offset = cone_height * h_factor * layer_factor * 0.3;

    p.position = hub_pos + Vec2::new(
        angle.cos() * current_radius,
        (angle.sin() * current_radius * 0.3) + y_offset
    );

    // 旋转跟随切向
    let tangent_angle = angle + std::f32::consts::PI / 2.0;
    p.rotation = tangent_angle;

    // 嗡鸣震动
    p.rotation += (t * 20.0).cos() * 0.02;
}

/// 第三相位：瞬狱锁定 (Ominous Pause) - 45% ~ 55%
/// 视觉：全屏突然一静，飞剑调头指向敌人，背景变暗
/// 动感：瞬间静止后的压迫感
fn phase_three_ominous_pause(
    p: &mut Particle,
    local_prog: f32,
    screen_events: &mut EventWriter<ScreenEffectEvent>,
    enemy_query: &Query<(Entity, &Transform), With<EnemySpriteMarker>>,
) {
    let t = (local_prog - 0.45) / 0.1;

    // 触发背景变暗（仅一次）
    if t > 0.0 && t < 0.05 && p.seed < 0.05 {
        screen_events.send(ScreenEffectEvent::Shake { trauma: 0.1, decay: 0.5 });
    }

    // 动态同步目标位置
    if let Some(target_entity) = p.target_entity {
        if let Ok((_, transform)) = enemy_query.get(target_entity) {
            p.target = Some(transform.translation.truncate());
        } else {
            // 目标实体已消失，寻找新目标进行重定向
            if let Some((new_entity, new_transform)) = enemy_query.iter().next() {
                p.target_entity = Some(new_entity);
                p.target = Some(new_transform.translation.truncate());
            }
        }
    }

    // 减速到静止 - 保持当前位置不变
    if t < 0.5 {
        let freeze_progress = (t * 2.0).min(1.0); // 限制在 [0, 1]
        let _damping = 1.0 - freeze_progress.powi(3);
        // 保持位置不变，确保位置始终有效
        // 不更新 p.position，保持上一次的位置
    } else {
        // 调头指向敌人
        if let Some(target) = p.target {
            let lock_progress = ((t - 0.5) * 2.0).min(1.0); // 限制在 [0, 1]

            // 确保 position 有效（防止 NaN）
            if !p.position.x.is_finite() || !p.position.y.is_finite() {
                // 如果位置无效，使用 hub_pos 作为默认位置
                p.position = Vec2::new(0.0, 250.0);
            }

            let dir_to_enemy = (target - p.position).normalize();
            let target_angle = (-dir_to_enemy.y).atan2(dir_to_enemy.x);

            // 平滑旋转
            p.rotation = p.rotation.lerp(target_angle, lock_progress * 3.0);

            // 剑身发光增强（通过颜色）
            let glow = (lock_progress * 0.5).min(1.0); // 限制 glow 范围
            p.start_color = Color::srgba(1.0 + glow, 0.9 + glow * 0.5, 0.3, 1.0);
        }
        // 如果没有 target，保持当前位置和旋转不变
    }
}

/// 第四相位：极速穿心 (Mach Piercing) - 55% ~ 100%
/// 视觉：极长残影流光，切向突刺
/// 动感：每把剑击中时触发高亮火花
fn phase_four_mach_piercing(
    p: &mut Particle,
    local_prog: f32,
    events: &mut EventWriter<SpawnEffectEvent>,
    screen_events: &mut EventWriter<ScreenEffectEvent>,
    enemy_query: &Query<(Entity, &Transform), With<EnemySpriteMarker>>,
    enemy_impact_query: &Query<(Entity, &crate::components::sprite::EnemySpriteMarker, &crate::components::sprite::PhysicalImpact), With<EnemySpriteMarker>>,
    camera_query: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
) {
    let strike_t = (local_prog - 0.55) / 0.45;

    // 【磁力吸引 + 摄像机投影修正】
    // 使用 world_to_viewport 进行正确的3D到2D投影，解决窗口大小变化时的位置偏移问题
    if let Some(target_entity) = p.target_entity {
        // 获取摄像机
        let (camera, camera_transform) = match camera_query.get_single() {
            Ok((cam, trans)) => (cam, trans),
            Err(_) => {
                // 摄像机未就绪，使用简单乘法作为后备方案
                if let Ok((_, _, impact)) = enemy_impact_query.get(target_entity) {
                    let x_world = impact.home_position.x * 100.0;
                    let y_world = (impact.home_position.z - 0.1) * 100.0;
                    p.target = Some(Vec2::new(x_world, y_world));
                }
                return;
            }
        };

        // 检查当前目标是否存活
        if let Ok((_, _, impact)) = enemy_impact_query.get(target_entity) {
            // 敌人存活，使用 world_to_viewport 进行正确的3D到2D投影
            if let Ok(screen_pos) = camera.world_to_viewport(camera_transform, impact.home_position) {
                // screen_pos 是相对于窗口左上角的像素坐标
                // 转换为相对于屏幕中心的粒子坐标
                let window_size = camera.viewport.as_ref()
                    .map(|v| v.physical_size)
                    .unwrap_or(bevy::render::camera::Viewport::default().physical_size);

                let center_x = window_size.x as f32 / 2.0;
                let center_y = window_size.y as f32 / 2.0;

                // 粒子坐标系：中心为原点，x向右为正，y向上为正
                let dynamic_target = Vec2::new(
                    screen_pos.x - center_x,
                    center_y - screen_pos.y
                );

                p.target = Some(dynamic_target);
            }
        } else {
            // 目标实体已消失，使用 target_group 进行循环重定向
            if let Some(ref group) = p.target_group {
                if let Some(idx) = p.target_index {
                    // 从当前索引开始，循环查找下一个存活目标
                    for offset in 1..=group.len() {
                        let new_idx = (idx + offset) % group.len();
                        let (entity, _) = &group[new_idx];

                        // 检查该实体是否仍然存活，并获取其位置
                        if let Ok((_, _, impact)) = enemy_impact_query.get(*entity) {
                            p.target_entity = Some(*entity);
                            if let Ok(screen_pos) = camera.world_to_viewport(camera_transform, impact.home_position) {
                                let window_size = camera.viewport.as_ref()
                                    .map(|v| v.physical_size)
                                    .unwrap_or(bevy::render::camera::Viewport::default().physical_size);

                                let center_x = window_size.x as f32 / 2.0;
                                let center_y = window_size.y as f32 / 2.0;

                                p.target = Some(Vec2::new(
                                    screen_pos.x - center_x,
                                    center_y - screen_pos.y
                                ));
                            }
                            break;
                        }
                    }
                }
            }
        }
    }

    if let Some(target) = p.target {
        // 确保 position 有效（防止 NaN）
        if !p.position.x.is_finite() || !p.position.y.is_finite() {
            p.position = Vec2::new(0.0, 250.0); // 默认位置
        }

        // 触发初始震动（优化：减少触发频率）
        // 只让第一个粒子触发震动，避免 80 个粒子都触发
        if strike_t > 0.0 && strike_t < 0.02 && p.seed < 0.01 {
            screen_events.send(ScreenEffectEvent::Shake { trauma: 0.3, decay: 0.5 });
        }

        // 修复问题 1: 记录 Phase 4 启动瞬间的位置作为固定起点
        if p.lock_pos.is_none() {
            p.lock_pos = Some(p.position);
        }
        let lock_pos = p.lock_pos.unwrap();

        // 三次贝塞尔曲线：B(t) = (1-t)³P0 + 3(1-t)²tP1 + 3(1-t)t²P2 + t³P3
        let base_dir = (target - lock_pos).normalize_or(Vec2::ZERO);
        let side_dir = Vec2::new(-base_dir.y, base_dir.x);

        let control1 = lock_pos + side_dir * (p.seed - 0.5) * 150.0;
        let control2 = target - base_dir * 50.0 + side_dir * (p.seed - 0.5) * 30.0;

        let inv_t = (1.0 - strike_t).max(0.0); // 确保 inv_t 非负
        let curve_pos = lock_pos * inv_t * inv_t * inv_t
            + control1 * 3.0 * inv_t * inv_t * strike_t
            + control2 * 3.0 * inv_t * strike_t * strike_t
            + target * strike_t * strike_t * strike_t;

        // 验证计算结果有效
        if curve_pos.x.is_finite() && curve_pos.y.is_finite() {
            p.position = curve_pos;
        }

        // 旋转朝向
        let move_dir = (target - p.position).normalize();
        p.rotation = (-move_dir.y).atan2(move_dir.x);

        // 极长流光（优化：降低频率，减少闪屏）
        // 只在特定时间间隔生成残影，而不是每帧都生成
        if strike_t > 0.0 && strike_t < 0.95 {
            // 每 0.05 秒生成一次残影（约每 3 帧一次，假设 60fps）
            let trail_interval = 0.05;
            let time_in_phase = strike_t * 0.45; // phase 4 的实际时间
            let should_spawn = (time_in_phase % trail_interval) < 0.01; // 允许一些误差

            if should_spawn && p.seed < 0.3 { // 只让 30% 的粒子生成残影
                // 只生成 1 个残影，而不是多个
                events.send(SpawnEffectEvent::new(
                    EffectType::SwordEnergy,
                    p.position.extend(0.0)
                ).burst(1));
            }
        }

        // 撞击火花（优化：减少粒子数量）
        if strike_t > 0.95 {
            // 只让部分粒子触发撞击效果，避免过度渲染
            if p.seed < 0.2 { // 只有 20% 的粒子触发撞击
                let impact_intensity = ((strike_t - 0.95) / 0.05).min(1.0);

                events.send(SpawnEffectEvent::new(
                    EffectType::ImpactSpark,
                    target.extend(0.0)
                ).burst((5.0 * impact_intensity) as usize)); // 从 12 减少到 5
            }

            // 撞击闪光（只触发一次）
            if strike_t > 0.98 && p.seed < 0.05 { // 只在最后时刻，只有少数粒子触发
                screen_events.send(ScreenEffectEvent::Shake { trauma: 0.2, decay: 0.3 });
            }
        }
    }
}
