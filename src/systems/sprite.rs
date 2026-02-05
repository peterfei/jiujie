//! Sprite 角色渲染与物理系统
//!
//! 实现 2.5D 纸片人渲染、物理冲击反馈、呼吸动画及残影特效


use bevy::prelude::*;
use bevy::scene::SceneRoot; 
use bevy::pbr::{DistanceFog, FogFalloff}; // 显式导入雾气组件
use crate::states::GameState;
use crate::components::sprite::{
    CharacterSprite, AnimationState, CharacterType,
    CharacterAnimationEvent, SpriteMarker, PlayerSpriteMarker, EnemySpriteMarker,
    Combatant3d, BreathAnimation, PhysicalImpact, CharacterAssets, Rotating, Ghost, ActionType,
    MagicSealMarker, RelicVisualMarker, SpiritClone, CombatCamera
};
use crate::components::CombatUiRoot;

pub struct SpritePlugin;

impl Plugin for SpritePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CharacterAnimationEvent>();
        app.add_event::<crate::components::particle::SpawnEffectEvent>(); 
        
        app.add_systems(
            Update,
            (
                handle_animation_events,
                trigger_hit_feedback,
                (
                    sync_2d_to_3d_render,
                    update_breath_animations,
                    update_physical_impacts,
                ).chain(),
                update_combat_camera, // [新增] 交互摄像机
                update_rotations,
                update_magic_seal_pulse,
                update_relic_floating,
                spawn_ghosts,
                cleanup_ghosts,
                update_spirit_clones,
                update_clouds, 
                update_mist, 
                update_wind_sway, 
                update_water, 
                update_sprite_animations,
            ).run_if(in_state(GameState::Combat))
        );
    }
}

use bevy::input::mouse::{MouseWheel, MouseMotion};

/// 更新战斗交互摄像机
pub fn update_combat_camera(
    mut scroll_evr: EventReader<MouseWheel>,
    mut motion_evr: EventReader<MouseMotion>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&mut Transform, &mut CombatCamera)>,
) {
    for (mut transform, mut config) in query.iter_mut() {
        // 1. 滚轮缩放
        for ev in scroll_evr.read() {
            config.distance = (config.distance - ev.y * 0.5).clamp(4.0, 25.0);
        }

        // 2. 左键旋转
        if mouse_button.pressed(MouseButton::Left) {
            for ev in motion_evr.read() {
                config.rotation_y -= ev.delta.x * 0.005;
                config.rotation_x = (config.rotation_x - ev.delta.y * 0.005).clamp(-1.2, 0.1);
            }
        } else {
            motion_evr.clear(); // 防止累积非点击时的位移
        }

        // 3. 计算最终位置：Orbit 算法
        let rotation = Quat::from_rotation_y(config.rotation_y) * Quat::from_rotation_x(config.rotation_x);
        let direction = rotation * Vec3::Z;
        transform.translation = config.target + direction * config.distance;
        transform.look_at(config.target, Vec3::Y);
    }
}

use crate::components::particle::SpawnEffectEvent;

/// 更新怨灵分身系统
pub fn update_spirit_clones(
    mut commands: Commands,
    mut query: Query<(Entity, &mut SpiritClone, &mut Transform)>,
    time: Res<Time>,
    mut effect_events: EventWriter<SpawnEffectEvent>,
) {
    let dt = time.delta_secs();
    for (entity, mut clone, mut transform) in query.iter_mut() {
        clone.lifetime -= dt;
        
        // 处理静止等待延迟
        if clone.delay > 0.0 {
            clone.delay -= dt;
            // 静止期间可以加一点轻微的抖动或半透明闪烁（可选）
            continue; 
        }

        // 延迟结束后，执行位移（向中心合拢）
        if clone.lifetime <= 0.0 {
            // [最终阶段] 分身合拢至中心并消散，产生剧烈爆炸
            use crate::components::particle::EffectType;
            
            // [修正] 仅对 X 和 Y 进行 UI 坐标转换，Z 轴保持在特效层级 (0.5 左右)
            let explosion_pos = Vec3::new(
                transform.translation.x * 100.0,
                transform.translation.y * 100.0,
                0.5 // 固定 Z 轴深度
            );
            
            // 生成式爆炸：规模根据随机种子微调
            let burst_count = if clone.seed > 0.5 { 30 } else { 20 };
            
            // 产生复合爆炸效果
            effect_events.send(SpawnEffectEvent::new(
                EffectType::ImpactSpark,
                explosion_pos + Vec3::new(0.0, -50.0, 0.0)
            ).burst(burst_count)); 
            
            effect_events.send(SpawnEffectEvent::new(
                EffectType::SwordEnergy,
                explosion_pos
            ).burst(burst_count / 2)); 

            commands.entity(entity).despawn_recursive();
            continue;
        }

        // --- 有机运动逻辑 ---
        // 基础直线位移
        transform.translation += clone.velocity * dt;
        
        // 垂直于运动方向的正弦扰动 (飘忽感)
        let sway_speed = 8.0 + clone.seed * 4.0;
        let sway_amount = 0.05 + clone.seed * 0.05;
        let time_val = time.elapsed_secs() + clone.seed * 100.0;
        
        // 计算垂向向量 (假设 velocity 只有 X/Z 位移)
        let sway = (time_val * sway_speed).sin() * sway_amount;
        transform.translation.y += sway; // 垂直方向飘动
        
        // 缩放演化：合拢时稍微拉长，模拟高速感
        let stretch = 1.0 + (1.0 - clone.lifetime.max(0.0) / 1.5) * 0.2;
        transform.scale = Vec3::new(1.0, stretch, 1.0);
    }
}

/// 更新法宝悬浮效果
fn update_relic_floating(
    mut query: Query<(&mut Transform, &RelicVisualMarker)>,
    time: Res<Time>,
) {
    for (mut transform, marker) in query.iter_mut() {
        let float_offset = (time.elapsed_secs() * 2.0).sin() * 0.15;
        transform.translation.y = marker.base_y + float_offset;
    }
}

/// 更新物理冲击效果
pub fn update_physical_impacts(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut PhysicalImpact, &BreathAnimation, &Combatant3d)>,
    mut effect_events: EventWriter<crate::components::particle::SpawnEffectEvent>,
    mut screen_events: EventWriter<crate::components::ScreenEffectEvent>,
) {
    let dt = time.delta_secs().min(0.033);
    for (mut transform, mut impact, breath, combatant) in query.iter_mut() {
        // 1. 模拟旋转弹簧力
        let spring_k = 25.0; 
        let damping = 6.0;
        let force = -spring_k * impact.tilt_amount;
        impact.tilt_velocity += force * dt;
        impact.tilt_velocity *= 1.0 - (damping * dt);
        impact.tilt_amount += impact.tilt_velocity * dt;

        // 2. 模拟位置弹簧力
        let pos_spring_k = 10.0; 
        let mut pos_damping = 5.0;
        
        // 3. 处理动作计时器逻辑
        let mut action_tilt_offset = 0.0;
        let mut action_pos_offset = Vec3::ZERO;
        
        if impact.action_timer > 0.0 {
            impact.action_timer -= dt;
            let dir = impact.action_direction; 
            
            let action_duration = match impact.action_type {
                ActionType::WolfBite => 1.0,
                ActionType::SpiderWeb => 0.8,
                ActionType::DemonCast => 0.6,
                ActionType::Ascend => 3.5, // 延长到 3.5s，涵盖所有落雷阶段
                _ => 1.0,
            };
            let progress = (1.0 - (impact.action_timer / action_duration)).clamp(0.0, 1.0);

            match impact.action_type {
                // ... (其他动作逻辑保持)
                ActionType::Ascend => {
                    // 祈祷升腾曲线优化：上升 -> 悬停 -> 降落
                    let height = 0.8;
                    if progress < 0.2 {
                        // 阶段 1: 上升 (前 20% 时间)
                        let p = progress / 0.2;
                        action_pos_offset.y = p * height;
                    } else if progress < 0.8 {
                        // 阶段 2: 悬停暂停不动 (中间 60% 时间)
                        action_pos_offset.y = height;
                    } else {
                        // 阶段 3: 降落 (最后 20% 时间)
                        let p = (1.0 - progress) / 0.2;
                        action_pos_offset.y = p * height;
                    }
                    
                    // 身体倾斜：悬停时保持轻微后仰
                    action_tilt_offset = if progress < 0.8 { -0.1 } else { -0.1 * (1.0 - progress) / 0.2 };
                    impact.offset_velocity = Vec3::ZERO;
                },
                ActionType::WolfBite => {
                    let action_phase = impact.action_timer * 12.5;
                    action_tilt_offset = action_phase.sin() * 0.8;
                    action_pos_offset.y = (progress * std::f32::consts::PI).sin() * 1.5;
                    
                    let stage_thresholds = [0.3, 0.6, 0.8];
                    let current_stage = impact.action_stage as usize;
                    if current_stage < stage_thresholds.len() && progress >= stage_thresholds[current_stage] {
                        use crate::components::particle::EffectType;
                        let y_offset = (current_stage as f32 - 1.0) * 0.3;
                        effect_events.send(crate::components::particle::SpawnEffectEvent::new(
                            EffectType::Slash, 
                            Vec3::new(-350.0, y_offset * 100.0, 0.5)
                        ).burst(12));
                        impact.action_stage += 1;
                    }

                    let target_dist = impact.target_offset_dist;
                    let current_dist = impact.current_offset.x.abs();
                    let dist_left = (target_dist - current_dist).max(0.0);
                    let speed_scalar = if dist_left < 1.0 { dist_left } else { 1.0 };
                    
                    pos_damping = 10.0;
                    impact.offset_velocity = Vec3::new(8.5 * dir * speed_scalar, 0.0, 0.0); 
                },
                ActionType::SpiderWeb => {
                    pos_damping = 5.0;
                    impact.offset_velocity = Vec3::new(4.5 * dir, 0.0, 0.0);
                },
                ActionType::WolfPounce => {
                    // 1. 狼扑动作终极版：蓄力 + 动态抛物线
                    let total_time = 0.8;
                    let elapsed = (total_time - impact.action_timer).max(0.0);
                    let normalized = (elapsed / total_time).clamp(0.0, 1.0);
                    
                    if normalized < 0.15 {
                        // --- 蓄力阶段：向后微缩，蓄势待发 ---
                        let back_t = normalized / 0.15;
                        action_pos_offset.x = -0.5 * dir * back_t;
                        action_tilt_offset = -0.1 * dir * back_t; // 压低身体
                        impact.offset_velocity = Vec3::ZERO;
                    } else {
                        // --- 扑杀阶段 ---
                        let jump_t = (normalized - 0.15) / 0.85;
                        let max_h = (impact.target_offset_dist * 0.4).min(2.5); // 动态高度
                        action_pos_offset.y = 4.0 * max_h * jump_t * (1.0 - jump_t);
                        action_tilt_offset = (0.5 - jump_t) * 0.4 * dir;
                        
                        let target_dist = impact.target_offset_dist;
                        let speed = (target_dist / total_time) * 1.3;
                        impact.offset_velocity = Vec3::new(speed * dir, 0.0, 0.0);
                        
                        // 落地瞬间 (jump_t 接近 1.0)
                        if jump_t > 0.95 && impact.action_stage == 0 {
                            use crate::components::particle::EffectType;
                            // [Fix] Convert 3D position back to UI coordinates (x100) for correct particle placement
                            let hit_pos = (impact.home_position + impact.current_offset) * 100.0;
                            effect_events.send(crate::components::particle::SpawnEffectEvent::new(
                                EffectType::ImpactSpark,
                                hit_pos + Vec3::new(0.0, -50.0, 0.0) // Adjusted offset for UI scale
                            ).burst(8));
                            impact.action_stage = 1; // 标记已落地
                        }
                    }
                    pos_damping = 8.0;
                },
                ActionType::SiriusFrenzy => {
                    // 2. 狼大招终极版：高频轨迹采样 + 目标收束
                    let stage_duration = 0.3;
                    let elapsed = (0.9 - impact.action_timer).max(0.0);
                    let stage = (elapsed / stage_duration).floor() as u32;
                    
                    let player_x = dir * impact.target_offset_dist;
                    let current_x = impact.current_offset.x;
                    let to_player_x = (player_x - current_x).signum();
                    
                    let stage_dir = match stage {
                        0 => { action_tilt_offset = -0.3 * dir; Vec3::new(to_player_x, 0.5, 0.0) },
                        1 => { action_tilt_offset = 0.4 * dir; Vec3::new(to_player_x, -0.8, 0.0) },
                        _ => { action_tilt_offset = 0.0; Vec3::new(to_player_x, 0.0, 0.0) },
                    };
                    
                    // --- 持续轨迹生成逻辑 ---
                    impact.trail_timer -= dt;
                    if impact.trail_timer <= 0.0 && stage < 3 {
                        use crate::components::particle::EffectType;
                        let spawn_pos = (impact.home_position + impact.current_offset) * 100.0;
                        
                        // 每帧生成 2 个带动量的轨迹粒子
                        let mut event = crate::components::particle::SpawnEffectEvent::new(
                            EffectType::WolfSlash,
                            spawn_pos + Vec3::new(0.0, 0.0, 0.1)
                        ).burst(2);
                        
                        // 赋予顺着当前冲刺方向的高速动量 (确保超过 10.0 的旋转阈值)
                        event.velocity_override = Some(Vec2::new(stage_dir.x * 45.0, stage_dir.y * 45.0));
                        effect_events.send(event);
                        
                        impact.trail_timer = 0.03; // 30ms 高频采样
                    }

                    if stage != impact.action_stage && stage < 3 {
                        screen_events.send(crate::components::ScreenEffectEvent::Shake { trauma: 0.7, decay: 4.0 });
                        impact.action_stage = stage;
                    }
                    
                    let stage_t = (elapsed % stage_duration) / stage_duration;
                    let dist_to_player = (player_x - current_x).abs();
                    let braking = if dist_to_player < 1.0 { dist_to_player } else { 1.0 };
                    let dash_speed = 32.0 * (1.0 - stage_t) * braking; 
                    
                    impact.offset_velocity = stage_dir * dash_speed;
                    pos_damping = 4.5; 
                },
                ActionType::SkitterApproach => {
                    // 1. 真实多足爬行位移逻辑 (类大作实现)
                    let jerky_phase = impact.action_timer * 30.0; // 提高频率
                    
                    // 垂直方向 (Y) 极微小跳动，模拟步足蹬地
                    action_pos_offset.y = jerky_phase.sin().abs() * -0.05; 
                    // [新增] 纵深方向 (Z) 抖动：这是蜘蛛“横行”感的灵魂，模拟左右腿交替发力
                    action_pos_offset.z = jerky_phase.cos() * 0.12;
                    
                    // [关键修正] 消除 45 度倾斜 (Z轴 Roll)，转为高频偏航 (Y轴 Yaw)
                    // 蜘蛛身体不应该左右歪斜，而是头部左右微调
                    action_tilt_offset = jerky_phase.sin() * 0.12; 
                    impact.tilt_amount *= 0.1; // 强制衰减原本的倾斜力
                    
                    // 距离限制逻辑 (保持原有的精准停顿)
                    let target_dist = impact.target_offset_dist;
                    let current_dist = impact.current_offset.x.abs();
                    let dist_left = (target_dist - current_dist).max(0.0);
                    let speed_scalar = if dist_left < 1.0 { (dist_left / 1.0).max(0.0) } else { 1.0 };
                    
                    let speed_pulse = (jerky_phase * 0.4).sin().abs() + 0.6;
                    impact.offset_velocity = Vec3::new(11.0 * dir * speed_pulse * speed_scalar, 0.0, 0.0);
                    pos_damping = 15.0; // 极高的阻尼，消除滑行感，增加步进感

                    // 2. 丝迹生成 (位置跟随 Z 轴抖动)
                    if speed_scalar > 0.1 {
                        impact.trail_timer -= dt;
                        if impact.trail_timer <= 0.0 {
                            use crate::components::particle::EffectType;
                            let trail_pos = impact.home_position + impact.current_offset + action_pos_offset;
                            effect_events.send(crate::components::particle::SpawnEffectEvent::new(
                                EffectType::SilkTrail,
                                trail_pos + Vec3::new(0.0, -0.6, 0.0)
                            ).burst(2));
                            impact.trail_timer = 0.05; 
                        }
                    }
                },
                ActionType::DemonCast => {
                    pos_damping = 30.0;
                    impact.offset_velocity = Vec3::ZERO;
                },
                _ => {}
            }
        }

        let pos_force = -pos_spring_k * impact.current_offset;
        impact.offset_velocity += pos_force * dt;
        impact.offset_velocity *= 1.0 - (pos_damping * dt);
        
        let move_delta = impact.offset_velocity * dt;
        impact.current_offset += move_delta;

        // 4. 模拟特殊回旋弹簧力
        let rot_spring_k = 45.0;
        let rot_damping = 6.0;
        let rot_force = -rot_spring_k * impact.special_rotation;
        impact.special_rotation_velocity += rot_force * dt;
        impact.special_rotation_velocity *= 1.0 - (rot_damping * dt);
        impact.special_rotation += impact.special_rotation_velocity * dt;

        impact.tilt_amount = impact.tilt_amount.clamp(-1.0, 1.0);

        let is_acting = impact.action_timer > 0.0 || impact.current_offset.length() > 0.05 || impact.offset_velocity.length() > 0.5;
        let breath_y = if is_acting { 0.0 } else { (breath.timer * breath.frequency).sin() * 0.02 };

        let tilt_suppression = 1.0 / (1.0 + impact.special_rotation.abs() * 5.0);
        let effective_tilt = impact.tilt_amount * tilt_suppression;

        let wolf_spin = if impact.action_timer > 0.0 && impact.action_type == ActionType::WolfBite {
            let progress = (1.0 - (impact.action_timer / 1.0)).clamp(0.0, 1.0);
            progress * std::f32::consts::PI * 4.0
        } else { 0.0 };

        // [关键修复] 根据 Combatant3d 的基础旋转字段计算最终偏转
        // 解决了不同模型（玩家、狼、蜘蛛）初始 Forward 轴不统一导致的朝向混乱
        let base_y_rot = combatant.base_rotation;

        transform.rotation = Quat::from_rotation_x(-0.2) 
            * Quat::from_rotation_y(base_y_rot + impact.special_rotation + action_tilt_offset + wolf_spin)
            * Quat::from_rotation_z(effective_tilt);
        
        transform.translation = impact.home_position + impact.current_offset + action_pos_offset + Vec3::new(0.0, breath_y, 0.0);
    }
}

/// 监听受击，触发物理反馈
pub fn trigger_hit_feedback(
    mut commands: Commands,
    mut events: EventReader<CharacterAnimationEvent>,
    mut query: Query<(&mut PhysicalImpact, &CharacterSprite, Option<&PlayerSpriteMarker>)>,
) {
    for event in events.read() {
        if let Ok((mut impact, sprite, is_player)) = query.get_mut(event.target) {
            let direction = if is_player.is_some() { 1.0 } else { -1.0 };
            impact.action_direction = direction; 
            
            if impact.action_timer > 0.0 { continue; }

            match event.animation {
                AnimationState::Hit => {
                    impact.action_type = ActionType::None;
                    impact.tilt_velocity = 15.0 * direction; 
                    impact.offset_velocity = Vec3::new(-2.0 * direction, 0.0, 0.0);
                }
                AnimationState::Death => {
                    impact.action_type = ActionType::None;
                    impact.tilt_velocity = 45.0 * direction; 
                    impact.offset_velocity = Vec3::new(-5.0 * direction, 2.0, 0.0);
                    commands.entity(event.target).insert(crate::components::particle::EnemyDeathAnimation::new(1.2));
                }
                AnimationState::Attack => {
                    impact.action_type = ActionType::None;
                    let target_x = if direction < 0.0 { -3.5 } else { 3.5 };
                    impact.target_offset_dist = (target_x - impact.home_position.x).abs();
                    impact.tilt_velocity = -40.0 * direction;
                    impact.offset_velocity = Vec3::new(20.0 * direction, 0.0, 0.0);
                }
                AnimationState::ImperialSword => {
                    impact.action_type = ActionType::None;
                    let target_x = if direction < 0.0 { -3.5 } else { 3.5 };
                    impact.target_offset_dist = (target_x - impact.home_position.x).abs() * 0.5;
                    impact.tilt_velocity = -25.0 * direction; 
                    impact.offset_velocity = Vec3::new(15.0 * direction, 0.0, 0.0);
                    impact.special_rotation_velocity = 150.0 * direction; 
                }
                                AnimationState::HeavenCast => {
                                    impact.action_type = ActionType::Ascend;
                                    impact.action_timer = 3.5; 
                                    impact.tilt_velocity = 0.0;
                 
                    impact.special_rotation = 0.0;
                    impact.special_rotation_velocity = 0.0; 
                    impact.offset_velocity = Vec3::ZERO;
                }
                AnimationState::Defense => {
                    impact.action_type = ActionType::None;
                    // 防御功法：彻底静止，仅微调倾斜
                    impact.tilt_velocity = -5.0 * direction; 
                    impact.special_rotation = 0.0;
                    impact.special_rotation_velocity = 0.0;
                    impact.offset_velocity = Vec3::ZERO;
                }
                AnimationState::DemonAttack => {
                    impact.action_type = ActionType::None;
                    let target_x = if direction < 0.0 { -3.5 } else { 3.5 };
                    impact.target_offset_dist = (target_x - impact.home_position.x).abs();
                    impact.tilt_velocity = -20.0 * direction;
                    impact.offset_velocity = Vec3::new(12.0 * direction, 0.0, 0.0);
                }
                AnimationState::DemonCast => {
                    impact.action_type = ActionType::DemonCast;
                    impact.tilt_velocity = 0.0; 
                    impact.special_rotation = 0.0;
                    impact.special_rotation_velocity = 0.0; 
                    impact.action_timer = 0.6; 
                }
                AnimationState::WolfAttack => {
                    let target_x = if direction < 0.0 { -3.5 } else { 3.5 };
                    impact.target_offset_dist = (target_x - impact.home_position.x).abs();
                    impact.action_type = ActionType::WolfPounce;
                    impact.tilt_velocity = -12.0 * direction;
                    impact.offset_velocity = Vec3::new(15.0 * direction, 0.0, 0.0);
                    impact.action_timer = 0.8;
                }
                AnimationState::WolfHowl => {
                    let target_x = if direction < 0.0 { -2.0 } else { 2.0 };
                    impact.target_offset_dist = (target_x - impact.home_position.x).abs();
                    impact.action_type = ActionType::SiriusFrenzy;
                    impact.action_timer = 0.9;
                    impact.action_stage = 0;
                    impact.trail_timer = 0.0; // 立即开始生成
                    impact.offset_velocity = Vec3::ZERO;
                    impact.tilt_velocity = 0.0;
                }
                AnimationState::SpiderAttack => {
                    let target_x = if direction < 0.0 { -3.5 } else { 3.5 };
                    impact.target_offset_dist = (target_x - impact.home_position.x).abs();
                    impact.action_type = ActionType::SkitterApproach;
                    impact.tilt_velocity = -1.0 * direction; // 近乎为零的微弱倾斜，模拟起步惯性
                    impact.offset_velocity = Vec3::new(14.0 * direction, 0.0, 0.0);
                    impact.action_timer = 1.2;
                    impact.trail_timer = 0.0;
                }
                AnimationState::SpiritAttack => {
                    let target_x = if direction < 0.0 { -3.5 } else { 3.5 };
                    impact.target_offset_dist = (target_x - impact.home_position.x).abs();
                    impact.action_type = ActionType::SpiritMultiShadow;
                    impact.tilt_velocity = 50.0; 
                    impact.offset_velocity = Vec3::new(22.0 * direction, 0.0, 0.0);
                    impact.special_rotation_velocity = 120.0; 

                    // --- [生成式进阶] 动态幻影分身阵 ---
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    
                    let target_ui_x = -350.0; 
                    let target_ui_y = 0.0; 
                    let clone_count = rng.gen_range(4..=6); // 随机生成 4 到 6 个分身

                    for i in 0..clone_count {
                        // 1. 程序化位置：在圆周上均匀分布并加入随机抖动 (Jitter)
                        let base_angle = (i as f32 / clone_count as f32) * std::f32::consts::TAU;
                        let jitter_angle = rng.gen_range(-0.2..0.2);
                        let final_angle = base_angle + jitter_angle;
                        
                        let radius = rng.gen_range(180.0..250.0);
                        let offset_x = final_angle.cos() * radius;
                        let offset_y = final_angle.sin() * radius;
                        let spawn_pos_ui = Vec3::new(target_ui_x + offset_x, target_ui_y + offset_y, 10.0);
                        
                        // 2. 生成式视觉：随机缩放、随机色调插值
                        let scale = rng.gen_range(0.8..1.2);
                        let mut clone_sprite = CharacterSprite::new(sprite.texture.clone(), sprite.size * scale);
                        
                        // 从冷色调光谱中随机插值
                        let hue = rng.gen_range(180.0..280.0); // 蓝-紫区间
                        clone_sprite.tint = Color::hsla(hue, 0.8, 0.6, 0.6); 

                        // 3. 动态运动参数
                        let speed = rng.gen_range(450.0..650.0);
                        let velocity_ui = Vec3::new(-offset_x, -offset_y, 0.0).normalize() * speed;

                        commands.spawn((
                            Transform::from_translation(spawn_pos_ui),
                            clone_sprite,
                            SpriteMarker,
                            Visibility::default(),
                            InheritedVisibility::default(),
                            ViewVisibility::default(),
                            SpiritClone {
                                lifetime: rng.gen_range(1.3..1.7), // 随机生命周期
                                delay: rng.gen_range(0.6..1.0),    // 随机静止时间，产生参差感
                                velocity: velocity_ui / 100.0,
                                seed: rng.gen(),                   // 独特种子驱动有机运动
                            },
                        ));
                    }
                }
                AnimationState::BossRoar => {
                    impact.action_type = ActionType::DemonCast;
                    impact.tilt_velocity = 0.0;
                    impact.special_rotation_velocity = 100.0; 
                    impact.action_timer = 1.2; 
                }
                AnimationState::BossFrenzy => {
                    let target_x = if direction < 0.0 { -3.5 } else { 3.5 };
                    impact.target_offset_dist = (target_x - impact.home_position.x).abs();
                    impact.action_type = ActionType::None;
                    impact.offset_velocity = Vec3::new(35.0 * direction, 0.0, 0.0);
                    impact.action_timer = 0.8;
                }
                AnimationState::Idle => {
                    impact.action_type = ActionType::None;
                    impact.action_timer = 0.0;
                    impact.special_rotation = 0.0;
                    impact.special_rotation_velocity = 0.0;
                }
            }
        }
    }
}

/// 更新精灵动画
fn update_sprite_animations(
    mut commands: Commands,
    mut query: Query<(Entity, &mut CharacterSprite, Option<&PlayerSpriteMarker>)>,
    character_assets: Res<CharacterAssets>,
    time: Res<Time>,
) {
    for (entity, mut sprite, is_player) in query.iter_mut() {
        if sprite.total_frames <= 1 { continue; }
        sprite.elapsed += time.delta_secs();
        if sprite.elapsed >= sprite.frame_duration {
            sprite.elapsed -= sprite.frame_duration;
            sprite.current_frame += 1;
            if sprite.current_frame >= sprite.total_frames {
                if sprite.looping { sprite.current_frame = 0; }
                else {
                    sprite.current_frame = sprite.total_frames - 1;
                    match sprite.state {
                        AnimationState::Death => {
                            commands.entity(entity).despawn_recursive();
                        }
                        AnimationState::Attack | AnimationState::Hit | AnimationState::ImperialSword | 
                        AnimationState::HeavenCast | AnimationState::Defense | AnimationState::DemonAttack | 
                        AnimationState::DemonCast | AnimationState::WolfAttack | AnimationState::WolfHowl | AnimationState::SpiderAttack | 
                        AnimationState::SpiritAttack | AnimationState::BossRoar | AnimationState::BossFrenzy => { 
                            // 自动恢复待机
                            if is_player.is_some() {
                                sprite.texture = character_assets.player_idle.clone();
                            }
                            sprite.set_idle(); 
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

/// 处理动画事件
pub fn handle_animation_events(
    mut events: EventReader<CharacterAnimationEvent>,
    mut query: Query<(&mut CharacterSprite, Option<&PlayerSpriteMarker>)>,
    character_assets: Res<CharacterAssets>,
) {
    for event in events.read() {
        if let Ok((mut sprite, is_player)) = query.get_mut(event.target) {
            match event.animation {
                AnimationState::Attack | AnimationState::ImperialSword => {
                    // 如果是玩家，切换到攻击贴图
                    if is_player.is_some() {
                        sprite.texture = character_assets.player_attack.clone();
                    }
                    if event.animation == AnimationState::Attack {
                        sprite.set_attack(4, 0.3);
                    } else {
                        sprite.set_attack(8, 0.5);
                    }
                }
                AnimationState::HeavenCast => {
                    // 天象施法：如果是玩家，切换到祈祷贴图
                    if is_player.is_some() {
                        sprite.texture = character_assets.player_prise.clone();
                    }
                    sprite.set_attack(6, 3.5);
                }
                AnimationState::Defense => {
                    // 防御功法：如果是玩家，切换到防御贴图
                    if is_player.is_some() {
                        sprite.texture = character_assets.player_defense.clone();
                    }
                    sprite.set_attack(4, 0.3);
                }
                AnimationState::DemonAttack => {
                    sprite.set_attack(6, 0.4);
                }
                AnimationState::DemonCast => {
                    sprite.set_attack(4, 0.3);
                }
                AnimationState::WolfAttack => {
                    sprite.set_attack(10, 1.0);
                }
                AnimationState::WolfHowl => {
                    sprite.set_attack(12, 0.9); // 大招节奏更紧凑
                }
                AnimationState::SpiderAttack => {
                    sprite.set_attack(8, 0.8);
                }
                AnimationState::SpiritAttack => {
                    sprite.set_attack(6, 0.4);
                }
                AnimationState::BossRoar => {
                    sprite.set_attack(12, 1.2);
                }
                AnimationState::BossFrenzy => {
                    sprite.set_attack(10, 0.8);
                }
                AnimationState::Hit => {
                    sprite.set_hit(3, 0.2);
                }
                AnimationState::Death => {
                    sprite.set_death(6, 0.5);
                }
                AnimationState::Idle => {
                    // 恢复待机：如果是玩家，切回默认贴图
                    if is_player.is_some() {
                        sprite.texture = character_assets.player_idle.clone();
                    }
                    sprite.set_idle();
                }
            }
        }
    }
}

/// 更新呼吸动画
fn update_breath_animations(
    mut query: Query<(&mut Transform, &mut BreathAnimation, &PhysicalImpact), Without<RelicVisualMarker>>,
    time: Res<Time>,
) {
    for (mut transform, mut breath, impact) in query.iter_mut() {
        breath.timer += time.delta_secs();
        let is_acting = impact.action_timer > 0.0 || impact.current_offset.length() > 0.05 || impact.offset_velocity.length() > 0.5;
        
        if is_acting {
            transform.scale = Vec3::ONE;
            if impact.action_type == ActionType::DemonCast {
                let pulse = 1.0 + (impact.action_timer * 35.0).sin().abs() * 0.15;
                transform.scale = Vec3::splat(pulse);
            }
        } else {
            let breath_cycle = (breath.timer * breath.frequency).sin();
            let stretch_y = 1.0 + breath_cycle * 0.015; 
            let squash_x = 1.0 - breath_cycle * 0.01;  
            transform.scale = Vec3::new(squash_x, stretch_y, 1.0);
        }
    }
}

/// 同步系统：将 2D 贴图同步到 3D 立牌材质
pub fn sync_2d_to_3d_render(
    mut commands: Commands,
    sprite_query: Query<(Entity, &CharacterSprite, &Transform, Option<&Combatant3d>, Option<&MeshMaterial3d<StandardMaterial>>, Has<RelicVisualMarker>), (With<SpriteMarker>, Changed<CharacterSprite>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, char_sprite, transform, combatant_3d, mat_handle_opt, is_relic) in sprite_query.iter() {
        if combatant_3d.is_none() {
            // --- 初始化流程 (仅首次) ---
            let x_3d = transform.translation.x / 100.0;
            let z_3d = transform.translation.y / 100.0;
            let is_boss = char_sprite.size.x > 150.0;

            let mesh = meshes.add(Rectangle::new(char_sprite.size.x / 50.0, char_sprite.size.y / 50.0));
            
            // 使用 PBR 材质替代 Unlit，通过环境光和点光源增强画质
            let material = materials.add(StandardMaterial {
                base_color: char_sprite.tint,
                base_color_texture: Some(char_sprite.texture.clone()),
                emissive: LinearRgba::from(char_sprite.tint).with_alpha(0.5), 
                emissive_texture: Some(char_sprite.texture.clone()),
                alpha_mode: AlphaMode::Blend, 
                cull_mode: None,
                double_sided: true,
                perceptual_roughness: 0.1, // 降低粗糙度，让光影更锐利
                ..default()
            });

            let home_pos = Vec3::new(x_3d, 0.8, z_3d + 0.1);
            let mut entity_cmd = commands.entity(entity);
            entity_cmd.insert((
                Combatant3d { facing_right: true, base_rotation: 0.0 },
                BreathAnimation::default(),
                PhysicalImpact { home_position: home_pos, ..default() }, 
                Mesh3d(mesh),
                MeshMaterial3d(material),
                bevy::pbr::NotShadowCaster,
                Transform::from_translation(home_pos).with_rotation(Quat::from_rotation_x(-0.2)), 
            )).remove::<Sprite>();

            // 非法宝（人物/怪）生成底座和补光灯
            if !is_relic {
                entity_cmd.with_children(|parent| {
                    let base_radius = if is_boss { 2.5 } else { 1.8 };
                    // 灵气法阵 (同步大作效果)
                    parent.spawn((
                        Mesh3d(meshes.add(Plane3d::default().mesh().size(base_radius * 2.0, base_radius * 2.0))), 
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgba(0.0, 0.8, 1.0, 0.6),
                            emissive: LinearRgba::new(0.0, 2.0, 5.0, 1.0),
                            alpha_mode: AlphaMode::Blend,
                            unlit: true,
                            ..default()
                        })),
                        Transform::from_xyz(0.0, 0.01, 0.0),
                        Rotating { speed: 0.6 },
                    ));
                    
                    // [大作优化] 为每个角色增加局部点光源
                    parent.spawn((
                        PointLight {
                            intensity: 12000.0,
                            radius: 8.0,
                            color: Color::srgb(1.0, 1.0, 1.0),
                            shadows_enabled: false,
                            ..default()
                        },
                        Transform::from_xyz(0.0, 2.5, 1.5),
                    ));
                });
            }
        } else {
            // --- 更新流程 (贴图切换时触发) ---
            if let Some(mat_handle) = mat_handle_opt {
                if let Some(material) = materials.get_mut(mat_handle) {
                    material.base_color_texture = Some(char_sprite.texture.clone());
                    material.emissive_texture = Some(char_sprite.texture.clone());
                    info!("【3D同步】已更新实体贴图");
                }
            }
        }
    }
}

/// 更新旋转系统
fn update_rotations(mut query: Query<(&mut Transform, &Rotating)>, time: Res<Time>) {
    for (mut transform, rotating) in query.iter_mut() {
        transform.rotate_y(rotating.speed * time.delta_secs());
    }
}

/// 生成残影系统
fn spawn_ghosts(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &PhysicalImpact, &Mesh3d, &MeshMaterial3d<StandardMaterial>, Option<&EnemySpriteMarker>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (_entity, transform, impact, mesh_handle, material_handle, enemy_marker) in query.iter() {
        // [新增逻辑] 仅在非跳跃动作且高速移动时生成残影
        let is_jumping = impact.action_type == ActionType::WolfPounce;
        let is_moving_fast = impact.offset_velocity.length() > 15.0 || impact.special_rotation_velocity.abs() > 60.0;
        
        if is_moving_fast && !is_jumping {
            let is_boss = enemy_marker.map_or(false, |m| m.id == 99);
            let ghost_material = if let Some(base_mat) = materials.get(material_handle) {
                let mut m = base_mat.clone();
                if is_boss { m.base_color = Color::srgba(1.5, 0.1, 0.1, 0.6); }
                else { m.base_color.set_alpha(0.4); }
                m.cull_mode = None; m.double_sided = true;
                materials.add(m)
            } else {
                materials.add(StandardMaterial { base_color: Color::srgba(1.0, 1.0, 1.0, 0.4), ..default() })
            };

            commands.spawn((
                Ghost { ttl: 0.15 }, // 从 0.3 减短到 0.15，残影更干脆
                Mesh3d(mesh_handle.0.clone()),
                MeshMaterial3d(ghost_material),
                Transform {
                    translation: transform.translation + Vec3::new(0.0, 0.0, -0.05),
                    rotation: transform.rotation,
                    scale: if is_boss { transform.scale * 1.05 } else { transform.scale },
                },
                CombatUiRoot,
            ));
        }
    }
}

/// 清理残影系统
fn cleanup_ghosts(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Ghost, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    for (entity, mut ghost, material_handle) in query.iter_mut() {
        ghost.ttl -= time.delta_secs();
        if ghost.ttl <= 0.0 { commands.entity(entity).despawn_recursive(); }
        else if let Some(material) = materials.get_mut(material_handle) {
            let alpha = (ghost.ttl / 0.15).powi(2) * 0.4; // 同步缩短 alpha 衰减周期
            material.base_color.set_alpha(alpha);
        }
    }
}

/// 更新法阵脉动效果
fn update_magic_seal_pulse(
    enemy_query: Query<&crate::components::Enemy>,
    mut seal_query: Query<(&MeshMaterial3d<StandardMaterial>, &mut Rotating), With<MagicSealMarker>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    let has_boss = enemy_query.iter().any(|e| e.enemy_type == crate::components::EnemyType::GreatDemon);
    let (base_emissive, pulse_speed, spin_speed) = if has_boss {
        (LinearRgba::new(1.5, 1.0, 0.2, 1.0), 3.0, 0.15) 
    } else {
        (LinearRgba::new(0.0, 0.8, 0.3, 1.0), 0.5, 0.05) 
    };

    for (material_handle, mut rotating) in seal_query.iter_mut() {
        if let Some(material) = materials.get_mut(material_handle) {
            let pulse = 1.0 + (time.elapsed_secs() * pulse_speed).sin() * 0.2;
            material.emissive = base_emissive * pulse;
            rotating.speed = spin_speed;
        }
    }
}

use crate::resources::{LandscapeGenerator, EnvironmentConfig};
use rand::SeedableRng;

/// 更新植被风场摇曳
pub fn update_wind_sway(
    time: Res<Time>,
    env: Res<EnvironmentConfig>,
    mut query: Query<(&mut Transform, &crate::components::sprite::WindSway)>,
) {
    let t = time.elapsed_secs();
    let wind_power = env.wind_strength; 

    for (mut transform, sway) in query.iter_mut() {
        let phase = transform.translation.x * 0.5 + transform.translation.z * 0.2 + sway.seed;
        let angle = (t * sway.speed + phase).sin() * sway.strength * wind_power + 
                    (t * sway.speed * 2.5 + phase).cos() * (sway.strength * 0.3);
        
        let rot = Quat::from_euler(EulerRot::XYZ, angle * 0.7, 0.0, angle);
        let (y_rot, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
        transform.rotation = Quat::from_rotation_y(y_rot) * rot;
    }
}

/// 更新水面动态
pub fn update_water(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &crate::components::sprite::Water)>,
) {
    let t = time.elapsed_secs();
    for (mut transform, water) in query.iter_mut() {
        let wave = (t * 0.8).sin() * water.wave_strength;
        transform.translation.y = -2.2 + wave; 
    }
}

/// 更新云海动画
pub fn update_clouds(
    time: Res<Time>,
    env: Res<EnvironmentConfig>,
    mut query: Query<(&mut Transform, &crate::components::sprite::Cloud)>,
) {
    let t = time.elapsed_secs();
    for (mut transform, cloud) in query.iter_mut() {
        let speed_factor = env.wind_strength;
        transform.translation.x += cloud.scroll_speed.x * speed_factor * time.delta_secs();
        transform.translation.z += cloud.scroll_speed.y * speed_factor * time.delta_secs();

        let dist_sq = transform.translation.x.powi(2) + transform.translation.z.powi(2);
        if dist_sq < 144.0 {
            let dir = Vec2::new(transform.translation.x, transform.translation.z).normalize();
            transform.translation.x = dir.x * 12.5;
            transform.translation.z = dir.y * 12.5;
        }

        let vertical_wave = (t * cloud.frequency + cloud.seed * 10.0).sin() * cloud.amplitude;
        transform.translation.y += vertical_wave * time.delta_secs();

        if transform.translation.x > 50.0 { transform.translation.x = -50.0; }
        if transform.translation.x < -50.0 { transform.translation.x = 50.0; }
        if transform.translation.z > 50.0 { transform.translation.z = -50.0; }
        if transform.translation.z < -50.0 { transform.translation.z = 50.0; }
    }
}

/// 更新动态流雾系统
pub fn update_mist(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &crate::components::sprite::Mist)>,
) {
    let dt = time.delta_secs();
    for (mut transform, mist) in query.iter_mut() {
        transform.translation.x += mist.scroll_speed.x * dt;
        transform.translation.z += mist.scroll_speed.y * dt;
        transform.translation.y += (time.elapsed_secs() * 0.5 + mist.seed).sin() * 0.02 * dt;

        if transform.translation.x > 60.0 { transform.translation.x = -60.0; }
        if transform.translation.x < -60.0 { transform.translation.x = 60.0; }
        if transform.translation.z > 60.0 { transform.translation.z = -60.0; }
        if transform.translation.z < -60.0 { transform.translation.z = 60.0; }
    }
}

fn generate_noise_texture(
    images: &mut ResMut<Assets<Image>>,
    width: u32,
    height: u32,
    mode: NoiseType,
) -> Handle<Image> {
    use rand::{Rng, SeedableRng};
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let mut data = vec![0; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let u = (x as f32 / width as f32) - 0.5;
            let v = (y as f32 / height as f32) - 0.5;
            let idx = ((y * width + x) * 4) as usize;
            
            let nx = u * 15.0f32;
            let ny = v * 15.0f32;
            let fbm = (nx.sin() * ny.cos() * 0.6 + (nx * 2.0).cos() * (ny * 2.0).sin() * 0.3 + (nx * 4.0).sin() * 0.1).abs();
            
            match mode {
                NoiseType::Rock => {
                    let factor = (0.4 + fbm * 0.6).clamp(0.0, 1.0);
                    data[idx] = (110.0 * factor) as u8;
                    data[idx+1] = (112.0 * factor) as u8;
                    data[idx+2] = (115.0 * factor) as u8;
                    data[idx+3] = 255;
                }
                NoiseType::Cloud => {
                    let d = (u*u + v*v).sqrt();
                    let alpha = ((1.0 - (d * 2.2).clamp(0.0, 1.0)).powi(3) * fbm).clamp(0.0, 1.0) * 0.4;
                    data[idx] = 255; data[idx+1] = 255; data[idx+2] = 255;
                    data[idx+3] = (alpha * 255.0) as u8;
                }
                NoiseType::Grass => {
                    data[idx] = 100; data[idx+1] = 150; data[idx+2] = 100; data[idx+3] = 255;
                }
            }
        }
    }
    use bevy::render::render_asset::RenderAssetUsages;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    images.add(Image::new(Extent3d { width, height, depth_or_array_layers: 1 }, TextureDimension::D2, data, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default()))
}

#[derive(Clone, Copy)]
enum NoiseType { Rock, Cloud, Grass }

pub fn spawn_procedural_landscape(
    mut commands: Commands,
    generator: Option<Res<LandscapeGenerator>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    _asset_server: Res<AssetServer>,
    env_assets: Option<Res<crate::resources::EnvironmentAssets>>,
) {
    let seed = generator.map(|g| g.seed).unwrap_or(12345);
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    use rand::Rng;

    let rock_tex = generate_noise_texture(&mut images, 512, 512, NoiseType::Rock);
    
    // [视觉优化] 石台使用 Mask 模式，配合 Y 轴微偏移，彻底解决闪烁消失问题
    let rock_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(rock_tex.clone()),
        perceptual_roughness: 0.95,
        alpha_mode: AlphaMode::Mask(0.1),
        ..default()
    });

    let main_island_root = commands.spawn((
        Transform::from_xyz(0.0, -2.2, 0.0),
        Visibility::default(), InheritedVisibility::default(), ViewVisibility::default(),
        Name::new("DynamicEnvIsland"),
    )).id();

    // [新增] 动态水面
    let water_mesh = meshes.add(Circle::new(45.0));
    commands.spawn((
        Mesh3d(water_mesh),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.1, 0.3, 0.5, 0.8),
            perceptual_roughness: 0.08,
            metallic: 0.1,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            double_sided: true,
            ..default()
        })),
        Transform::from_xyz(0.0, -2.2, 0.0).with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        crate::components::sprite::Water {
            flow_speed: Vec2::new(0.1, 0.1),
            wave_strength: 0.15,
        },
        CombatUiRoot,
    ));

    // --- [万法归一] 巨型全场聚灵阵地板 (解决太空感) ---
    let floor_radius = 15.0;
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(floor_radius * 2.0, floor_radius * 2.0))), 
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.0, 0.8, 1.0, 0.4),
            base_color_texture: Some(_asset_server.load("textures/magic_circle.png")),
            emissive: LinearRgba::new(0.0, 1.5, 3.0, 1.0),
            emissive_texture: Some(_asset_server.load("textures/magic_circle.png")),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.05, 0.0),
        Rotating { speed: 0.15 }, // 全场大阵缓缓转动，庄严厚重
        MagicSealMarker, // 标记以便后期实现呼吸脉动光效
        CombatUiRoot,
    ));

    if let Some(assets) = &env_assets {
        // [Reactive PG] 极简巨石阵，填补远景虚空
        for _ in 0..15 { 
            let radius = rng.gen_range(25.0..65.0); 
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let pos_z = angle.sin() * radius;
            let y = ((radius - 20.0) / 10.0f32).powf(1.3) * 5.0 - 5.0;
            let pos = Vec3::new(angle.cos() * radius, y, pos_z);
            
            let scale_xz = rng.gen_range(15.0..35.0);
            let scale_y = rng.gen_range(10.0..45.0); // 真正的高耸入云

            commands.entity(main_island_root).with_children(|parent| {
                parent.spawn((
                    SceneRoot(assets.rock.clone()),
                    Transform::from_translation(pos)
                        .with_scale(Vec3::new(scale_xz, scale_y, scale_xz))
                        .with_rotation(Quat::from_rotation_y(rng.gen_range(0.0..6.28))),
                ));
            });
        }
    }

    // --- 动态流雾与光照微调 ---
    let mist_tex = generate_noise_texture(&mut images, 256, 256, NoiseType::Cloud);
    let unit_quad = meshes.add(Plane3d::default().mesh().size(1.0, 1.0));
    
    for _ in 0..25 {
        let radius = rng.gen_range(30.0..80.0) as f32;
        let angle = rng.gen_range(0.0..6.28) as f32;
        let pos = Vec3::new(angle.cos() * radius, rng.gen_range(2.0..6.5), angle.sin() * radius);
        
        commands.spawn((
            Mesh3d(unit_quad.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.8, 0.9, 1.0, rng.gen_range(0.05..0.15)), 
                base_color_texture: Some(mist_tex.clone()),
                alpha_mode: AlphaMode::Blend,
                emissive: LinearRgba::new(0.05, 0.1, 0.2, 0.2), 
                double_sided: true, cull_mode: None, ..default()
            })),
            Transform::from_translation(pos)
                .with_scale(Vec3::new(rng.gen_range(35.0..75.0), 1.0, rng.gen_range(25.0..55.0)))
                .with_rotation(Quat::from_rotation_y(rng.gen_range(-1.0..1.0))),
            crate::components::sprite::Mist { scroll_speed: Vec2::new(rng.gen_range(-0.8..0.8), 0.0), seed: rng.gen() },
            CombatUiRoot,
        ));
    }

    commands.spawn((
        DirectionalLight { 
            shadows_enabled: false, 
            illuminance: 120000.0, // 极强主光源勾勒 NPR 轮廓
            color: Color::srgb(1.0, 0.98, 0.92), 
            ..default() 
        },
        Transform::from_xyz(30.0, 60.0, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    spawn_cloud_sea(&mut commands, &mut materials, &mut images, &mut meshes, seed, env_assets);
}

fn spawn_cloud_sea(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    images: &mut ResMut<Assets<Image>>,
    meshes: &mut ResMut<Assets<Mesh>>,
    seed: u64,
    env_assets: Option<Res<crate::resources::EnvironmentAssets>>,
) {
    let cloud_tex = generate_noise_texture(images, 256, 256, NoiseType::Cloud);
    let unit_quad = meshes.add(Plane3d::default().mesh().size(1.0, 1.0));
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed + 777);
    use rand::Rng;
    
    if let Some(_assets) = &env_assets {
        for _ in 0..12 {
            let radius = rng.gen_range(25.0..85.0);
            let angle: f32 = rng.gen_range(0.0..6.28);
            let center = Vec3::new(angle.cos() * radius, rng.gen_range(-15.0..-5.0), angle.sin() * radius);
            for layer in 0..3 {
                let layer_opacity = 0.15 - (layer as f32 * 0.04);
                let base_scale = rng.gen_range(18.0..30.0);
                commands.spawn((
                    Mesh3d(unit_quad.clone()),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgba(1.0, 1.0, 1.0, layer_opacity),
                        base_color_texture: Some(cloud_tex.clone()),
                        alpha_mode: AlphaMode::Blend,
                        unlit: true,
                        cull_mode: None, double_sided: true,
                        ..default()
                    })),
                    Transform::from_translation(center + Vec3::Y * (layer as f32 * 0.8))
                        .with_scale(Vec3::new(base_scale * (1.0 + layer as f32 * 0.3), 1.0, base_scale))
                        .with_rotation(Quat::from_rotation_y(rng.gen_range(0.0..6.28))),
                    // [视觉优化] 将滚动速度和波幅设为 0，使云海石台保持静止
                    crate::components::sprite::Cloud {
                        scroll_speed: Vec2::ZERO,
                        amplitude: 0.0, frequency: 0.0, seed: rng.gen(),
                    },
                    CombatUiRoot,
                ));
            }
        }
    }
}

pub fn spawn_character_sprite(
    commands: &mut Commands,
    character_assets: &CharacterAssets, 
    character_type: CharacterType,
    position: Vec3,
    size: Vec2,
    enemy_id: Option<u32>,
    tint: Option<Color>, 
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Entity {
    // 1. 资源准备
    let model_3d = match character_type {
        CharacterType::Player => &character_assets.player_3d,
        CharacterType::DemonicWolf => &character_assets.wolf_3d,
        CharacterType::PoisonSpider => &character_assets.spider_3d,
        CharacterType::CursedSpirit => &character_assets.spirit_3d,
        CharacterType::GreatDemon => &character_assets.boss_3d,
    };

    let texture_2d = match character_type {
        CharacterType::Player => character_assets.player_idle.clone(),
        CharacterType::DemonicWolf => character_assets.wolf.clone(),
        CharacterType::PoisonSpider => character_assets.spider.clone(),
        CharacterType::CursedSpirit => character_assets.spirit.clone(),
        CharacterType::GreatDemon => character_assets.boss.clone(),
    };

    let is_boss = size.x > 150.0;
    let mut sprite = CharacterSprite::new(texture_2d.clone(), size);
    if let Some(c) = tint { sprite = sprite.with_tint(c); }

    // [关键修复] 将 UI 坐标转换为 3D 世界坐标
    let x_3d = position.x / 100.0;
    let z_3d = position.y / 100.0;
    let world_pos = Vec3::new(x_3d, 0.8, z_3d + 0.1);

    // [最终修正] 解决不同模型的 Forward 轴差异，确保对峙感：
    // 1. 玩家(Warrior): 初始朝向正确，旋转 0 度 (保持默认) 面向右侧 (+X)
    // 2. 狼(Wolf): 初始朝向正确，旋转 -PI/2 (-90度) 面向左侧 (-X)
    // 3. 蜘蛛(Spider): 初始朝向右侧，旋转 PI (180度) 面向左侧 (-X)
    let base_rotation = match character_type {
        CharacterType::Player => 0.0,
        CharacterType::DemonicWolf => -std::f32::consts::FRAC_PI_2,
        CharacterType::PoisonSpider => std::f32::consts::PI,
        _ => -std::f32::consts::FRAC_PI_2, 
    };

    let is_player = character_type == CharacterType::Player;
    let facing_right = is_player;

    // 2. 生成实体 (3D 优先)
    let mut entity_cmd = commands.spawn((
        Transform::from_translation(world_pos).with_rotation(Quat::from_rotation_x(-0.2) * Quat::from_rotation_y(base_rotation)),
        Visibility::default(), InheritedVisibility::default(), ViewVisibility::default(),
        SpriteMarker,
        sprite, // 必须挂载，用于保存战斗状态 (AnimationState)
        PhysicalImpact { home_position: world_pos, ..default() }, // 必须挂载，否则无法处理位移
        BreathAnimation::default(), // 必须挂载，提供生命力
        Combatant3d { facing_right, base_rotation },
    ));

    if let Some(model_handle) = model_3d {
        // [AAA] 3D 分支：挂载模型
        entity_cmd.insert(bevy::scene::SceneRoot(model_handle.clone()));
        // 修正缩放：3D 模型通常需要放大
        entity_cmd.insert(Transform::from_translation(world_pos)
            .with_rotation(Quat::from_rotation_x(-0.2) * Quat::from_rotation_y(base_rotation))
            .with_scale(Vec3::splat(if is_boss { 3.0 } else { 2.0 })));
            
                // [NPR 优化] 补光灯大幅减弱，仅用于提供边缘微光
            
                entity_cmd.with_children(|parent| {
            
                    parent.spawn((
            
                        PointLight {
            
                            intensity: 3000.0, 
            
                            radius: 10.0,
            
                            color: Color::srgb(0.8, 0.9, 1.0),
            
                            shadows_enabled: false,
            
                            ..default()
            
                        },
            
                        Transform::from_xyz(0.0, 3.0, 2.0), 
            
                    ));
            
                });
            
            } else {
            
                // [NPR 降级] 2D 分支材质优化：消除高光
            
                let mesh = meshes.add(Rectangle::new(size.x / 50.0, size.y / 50.0));
            
                let material = materials.add(StandardMaterial {
            
                    base_color: tint.unwrap_or(Color::WHITE),
            
                    base_color_texture: Some(texture_2d),
            
                    perceptual_roughness: 0.95,
            
                    reflectance: 0.05, // 极低反射，消除泛白感
            
                    unlit: false, 
            
                    alpha_mode: AlphaMode::Blend, cull_mode: None, double_sided: true,
            
                    ..default()
            
                });
            
                entity_cmd.insert((Mesh3d(mesh), MeshMaterial3d(material)));
            
            }
            
        
            
            let entity = entity_cmd.id();
            
        
            
            // 4. 挂载身份标记
            
            match character_type {
            
                CharacterType::Player => { commands.entity(entity).insert(PlayerSpriteMarker); }
            
                _ => { if let Some(id) = enemy_id { commands.entity(entity).insert(EnemySpriteMarker { id }); } }
            
            };
            
        
            
            entity
            
        }
