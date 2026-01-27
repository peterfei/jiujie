//! Sprite 角色渲染与物理系统
//!
//! 实现 2.5D 纸片人渲染、物理冲击反馈、呼吸动画及残影特效

use bevy::prelude::*;
use crate::states::GameState;
use crate::components::sprite::{
    CharacterSprite, AnimationState, CharacterType,
    CharacterAnimationEvent, SpriteMarker, PlayerSpriteMarker, EnemySpriteMarker,
    Combatant3d, BreathAnimation, PhysicalImpact, CharacterAssets, Rotating, Ghost, ActionType,
    MagicSealMarker, RelicVisualMarker
};
use crate::plugins::CombatUiRoot;

pub struct SpritePlugin;

impl Plugin for SpritePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CharacterAnimationEvent>();
        
        // 使用 .chain() 确保逻辑顺序，彻底解决动画冲突
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
                update_rotations,
                update_magic_seal_pulse,
                update_relic_floating,
                spawn_ghosts,
                cleanup_ghosts,
                update_sprite_animations,
            ).run_if(in_state(GameState::Combat))
        );
    }
}

/// 更新法宝悬浮效果
fn update_relic_floating(
    mut query: Query<(&mut Transform, &RelicVisualMarker)>,
    time: Res<Time>,
) {
    for (mut transform, marker) in query.iter_mut() {
        // 浮动公式：base_y + sin(t * freq) * amp
        let float_offset = (time.elapsed_secs() * 2.0).sin() * 0.15;
        transform.translation.y = marker.base_y + float_offset;
    }
}

/// 更新物理冲击效果（让立牌产生倾斜和弹回感）
fn update_physical_impacts(
    mut query: Query<(&mut Transform, &mut PhysicalImpact, &BreathAnimation)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (mut transform, mut impact, breath) in query.iter_mut() {
        // 1. 模拟旋转弹簧力 (Tilt)
        let spring_k = 25.0; 
        let damping = 6.0;
        let force = -spring_k * impact.tilt_amount;
        impact.tilt_velocity += force * dt;
        impact.tilt_velocity *= 1.0 - (damping * dt);
        impact.tilt_amount += impact.tilt_velocity * dt;

        // 2. 模拟位置弹簧力 (将位移拉回 0)
        let mut pos_spring_k = 10.0; 
        let mut pos_damping = 5.0;
        
        // 3. 处理动作计时器逻辑 (特化动作)
        let mut action_tilt_offset = 0.0;
        let mut action_pos_offset = Vec3::ZERO;
        
        if impact.action_timer > 0.0 {
            impact.action_timer -= dt;
            let dir = impact.action_direction; 
            
            // 计算动作进度
            let action_duration = match impact.action_type {
                ActionType::WolfBite => 1.0,
                ActionType::SpiderWeb => 0.8,
                ActionType::DemonCast => 0.6,
                _ => 1.0,
            };
            let progress = (1.0 - (impact.action_timer / action_duration)).clamp(0.0, 1.0);

            match impact.action_type {
                ActionType::WolfBite => {
                    // 贪狼：空中回旋扑杀 (720度) + 2次撕咬
                    let action_phase = impact.action_timer * 12.5;
                    action_tilt_offset = action_phase.sin() * 0.8;
                    
                    // 扑杀弧线
                    action_pos_offset.y = (progress * std::f32::consts::PI).sin() * 1.5;
                    
                    // 距离感知减速
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
        
        let delta_offset = impact.offset_velocity * dt;
        impact.current_offset += delta_offset;

        // 4. 模拟特殊回旋弹簧力 (御剑术自转)
        let rot_spring_k = 45.0;
        let rot_damping = 6.0;
        let rot_force = -rot_spring_k * impact.special_rotation;
        impact.special_rotation_velocity += rot_force * dt;
        impact.special_rotation_velocity *= 1.0 - (rot_damping * dt);
        impact.special_rotation += impact.special_rotation_velocity * dt;

        // 5. 限制倾斜角度
        impact.tilt_amount = impact.tilt_amount.clamp(-1.0, 1.0);

        // 6. 整合呼吸动画 (动态抑制)
        let is_acting = impact.action_timer > 0.0 || impact.current_offset.length() > 0.05 || impact.offset_velocity.length() > 0.5;
        let breath_y = if is_acting { 0.0 } else { (breath.timer * breath.frequency).sin() * 0.02 };

        // 7. 应用变换 (视角优化)
        let tilt_suppression = 1.0 / (1.0 + impact.special_rotation.abs() * 5.0);
        let effective_tilt = impact.tilt_amount * tilt_suppression;

        // 狼的空中旋转补丁
        let wolf_spin = if impact.action_timer > 0.0 && impact.action_type == ActionType::WolfBite {
            let progress = (1.0 - (impact.action_timer / 1.0)).clamp(0.0, 1.0);
            progress * std::f32::consts::PI * 4.0
        } else { 0.0 };

        transform.rotation = Quat::from_rotation_x(-0.2) 
            * Quat::from_rotation_z(effective_tilt)
            * Quat::from_rotation_y(impact.special_rotation + action_tilt_offset + wolf_spin);
        
        transform.translation = impact.home_position + impact.current_offset + action_pos_offset + Vec3::new(0.0, breath_y, 0.0);
    }
}

/// 监听受击，触发物理反馈
fn trigger_hit_feedback(
    mut commands: Commands,
    mut events: EventReader<CharacterAnimationEvent>,
    mut query: Query<(&mut PhysicalImpact, Option<&PlayerSpriteMarker>)>,
) {
    for event in events.read() {
        if let Ok((mut impact, is_player)) = query.get_mut(event.target) {
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
                    
                    // 关键修复：挂载死亡动画组件，触发 update_enemy_death_animation 系统进行销毁
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
                    impact.target_offset_dist = (target_x - impact.home_position.x).abs();
                    impact.tilt_velocity = -10.0 * direction; 
                    impact.offset_velocity = Vec3::new(28.0 * direction, 0.0, 0.0);
                    impact.special_rotation_velocity = 80.0 * direction; 
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
                    impact.action_type = ActionType::WolfBite;
                    impact.tilt_velocity = -25.0 * direction;
                    impact.offset_velocity = Vec3::new(16.0 * direction, 0.0, 0.0);
                    impact.action_timer = 1.0; 
                }
                AnimationState::SpiderAttack => {
                    let target_x = if direction < 0.0 { -3.5 } else { 3.5 };
                    impact.target_offset_dist = (target_x - impact.home_position.x).abs();
                    impact.action_type = ActionType::SpiderWeb;
                    impact.tilt_velocity = -8.0 * direction;
                    impact.offset_velocity = Vec3::new(10.0 * direction, 0.0, 0.0);
                    impact.action_timer = 0.8;
                }
                AnimationState::SpiritAttack => {
                    let target_x = if direction < 0.0 { -3.5 } else { 3.5 };
                    impact.target_offset_dist = (target_x - impact.home_position.x).abs();
                    impact.action_type = ActionType::None;
                    impact.tilt_velocity = 50.0; 
                    impact.offset_velocity = Vec3::new(22.0 * direction, 0.0, 0.0);
                    impact.special_rotation_velocity = 120.0; 
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
                _ => {}
            }
        }
    }
}

/// 更新呼吸动画
fn update_breath_animations(
    mut query: Query<(&mut Transform, &mut BreathAnimation, &PhysicalImpact)>,
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
fn sync_2d_to_3d_render(
    mut commands: Commands,
    sprite_query: Query<(Entity, &CharacterSprite, &Transform, Option<&Combatant3d>), (With<SpriteMarker>, Changed<CharacterSprite>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, char_sprite, transform, combatant_3d) in sprite_query.iter() {
        if combatant_3d.is_none() {
            // 移除clamp限制，支持多敌人场景下的正确位置
            let x_3d = transform.translation.x / 100.0;
            let z_3d = transform.translation.y / 100.0;

            let is_boss = char_sprite.size.x > 150.0;

            let mesh = meshes.add(Rectangle::new(char_sprite.size.x / 50.0, char_sprite.size.y / 50.0));
            let material = materials.add(StandardMaterial {
                base_color: Color::WHITE,
                base_color_texture: Some(char_sprite.texture.clone()),
                emissive: LinearRgba::WHITE, 
                emissive_texture: Some(char_sprite.texture.clone()),
                reflectance: 0.0,
                alpha_mode: AlphaMode::Mask(0.5), 
                cull_mode: None,
                double_sided: true,
                ..default()
            });

            let home_pos = Vec3::new(x_3d, 0.05, z_3d + 0.1);
            commands.entity(entity).insert((
                Combatant3d { facing_right: true },
                BreathAnimation::default(),
                PhysicalImpact { home_position: home_pos, ..default() }, 
                Mesh3d(mesh),
                MeshMaterial3d(material),
                bevy::pbr::NotShadowCaster,
                Transform::from_translation(home_pos).with_rotation(Quat::from_rotation_x(-0.2)), 
            )).remove::<Sprite>()
            .with_children(|parent| {
                let base_radius = if is_boss { 1.2 } else { 0.8 };
                parent.spawn((
                    Mesh3d(meshes.add(Cylinder::new(base_radius, 0.02))), 
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgba(0.0, 0.05, 0.0, 0.4),
                        emissive: LinearRgba::new(0.0, 0.2, 0.1, 1.0),
                        alpha_mode: AlphaMode::Blend,
                        ..default()
                    })),
                    Transform::from_xyz(0.0, -0.02, 0.0),
                ));
            });
        }
    }
}

/// 更新精灵动画
fn update_sprite_animations(
    mut commands: Commands,
    mut query: Query<(Entity, &mut CharacterSprite)>,
    time: Res<Time>,
) {
    for (entity, mut sprite) in query.iter_mut() {
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
                            info!("角色实体 {:?} 已彻底消散", entity);
                            commands.entity(entity).despawn_recursive();
                        }
                        AnimationState::Attack | AnimationState::Hit | AnimationState::ImperialSword | 
                        AnimationState::DemonAttack | AnimationState::DemonCast | AnimationState::WolfAttack | 
                        AnimationState::SpiderAttack | AnimationState::SpiritAttack | AnimationState::BossRoar | 
                        AnimationState::BossFrenzy => { sprite.set_idle(); }
                        _ => {}
                    }
                }
            }
        }
    }
}

/// 处理动画事件
fn handle_animation_events(
    mut events: EventReader<CharacterAnimationEvent>,
    mut query: Query<&mut CharacterSprite>,
) {
    for event in events.read() {
        if let Ok(mut sprite) = query.get_mut(event.target) {
            match event.animation {
                AnimationState::Attack => {
                    sprite.set_attack(4, 0.3);
                    info!("角色 {:?} 开始攻击动画", event.target);
                }
                AnimationState::ImperialSword => {
                    sprite.set_attack(8, 0.5);
                    info!("角色 {:?} 开始御剑术回旋斩", event.target);
                }
                AnimationState::DemonAttack => {
                    sprite.set_attack(6, 0.4);
                    info!("角色 {:?} 开始妖术突袭", event.target);
                }
                AnimationState::DemonCast => {
                    sprite.set_attack(4, 0.3);
                    info!("角色 {:?} 开始施展妖术/脉冲", event.target);
                }
                AnimationState::WolfAttack => {
                    sprite.set_attack(10, 1.0);
                    info!("角色 {:?} 开始贪狼撕咬", event.target);
                }
                AnimationState::SpiderAttack => {
                    sprite.set_attack(8, 0.8);
                    info!("角色 {:?} 开始幽蛛吐丝", event.target);
                }
                AnimationState::SpiritAttack => {
                    sprite.set_attack(6, 0.4);
                    info!("角色 {:?} 开始怨灵突袭", event.target);
                }
                AnimationState::BossRoar => {
                    sprite.set_attack(12, 1.2);
                    info!("BOSS {:?} 发起：啸天", event.target);
                }
                AnimationState::BossFrenzy => {
                    sprite.set_attack(10, 0.8);
                    info!("BOSS {:?} 发起：瞬狱杀", event.target);
                }
                AnimationState::Hit => {
                    sprite.set_hit(3, 0.2);
                    info!("角色 {:?} 开始受击动画", event.target);
                }
                AnimationState::Death => {
                    sprite.set_death(6, 0.5);
                    info!("角色 {:?} 开始死亡动画", event.target);
                }
                AnimationState::Idle => {
                    sprite.set_idle();
                    info!("角色 {:?} 恢复待机动画", event.target);
                }
            }
        }
    }
}

/// 创建角色精灵实体
pub fn spawn_character_sprite(
    commands: &mut Commands,
    character_assets: &CharacterAssets, 
    character_type: CharacterType,
    position: Vec3,
    size: Vec2,
    enemy_id: Option<u32>,
) -> Entity {
    let texture = match character_type {
        CharacterType::Player => character_assets.player_idle.clone(),
        CharacterType::DemonicWolf => character_assets.wolf.clone(),
        CharacterType::PoisonSpider => character_assets.spider.clone(),
        CharacterType::CursedSpirit => character_assets.spirit.clone(),
        CharacterType::GreatDemon => character_assets.boss.clone(),
    };

    let sprite = Sprite { custom_size: Some(size), anchor: bevy::sprite::Anchor::BottomCenter, ..default() };

    let mut entity_cmd = commands.spawn((
        sprite,
        Transform::from_translation(position),
        CharacterSprite::new(texture, size), 
        SpriteMarker,
    ));

    match character_type {
        CharacterType::Player => { entity_cmd.insert(PlayerSpriteMarker); }
        CharacterType::GreatDemon => { 
            if let Some(id) = enemy_id { entity_cmd.insert(EnemySpriteMarker { id }); }
            else { entity_cmd.insert(EnemySpriteMarker { id: 99 }); }
        }
        _ => { if let Some(id) = enemy_id { entity_cmd.insert(EnemySpriteMarker { id }); } }
    };

    entity_cmd.id()
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
        let is_moving_fast = impact.offset_velocity.length() > 5.0 || impact.special_rotation_velocity.abs() > 30.0;
        if is_moving_fast {
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
                Ghost { ttl: 0.3 },
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
            let alpha = (ghost.ttl / 0.3).powi(2) * 0.4;
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
