//! Sprite 角色渲染系统
//!
//! 处理战斗中的角色精灵显示和动画

use bevy::prelude::*;
use bevy::sprite::Anchor;
use crate::components::sprite::{
    CharacterSprite, AnimationState, CharacterType,
    CharacterAnimationEvent, SpriteMarker, PlayerSpriteMarker, EnemySpriteMarker,
    Combatant3d, BreathAnimation, PhysicalImpact, CharacterAssets, Rotating, Ghost
};
use crate::states::GameState;

/// Sprite 渲染插件
pub struct SpritePlugin;

impl Plugin for SpritePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CharacterAnimationEvent>();
        app.add_systems(
            Update,
            (
                update_sprite_animations,
                handle_animation_events,
                update_breath_animations,
                sync_2d_to_3d_render,
                update_physical_impacts,
                trigger_hit_feedback,
                update_rotations,
                spawn_ghosts,
                cleanup_ghosts,
            ).run_if(in_state(GameState::Combat))
        );
    }
}

/// 生成残影
fn spawn_ghosts(
    mut commands: Commands,
    query: Query<(&Transform, &PhysicalImpact, &Mesh3d, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
    mut last_spawn: Local<f32>,
) {
    if time.elapsed_secs() - *last_spawn < 0.03 { return; }

    for (transform, impact, mesh, material_handle) in query.iter() {
        if impact.offset_velocity.length() > 3.0 {
            *last_spawn = time.elapsed_secs();
            
            // 关键修复：克隆材质而不是共用句柄
            let ghost_material = if let Some(base_mat) = materials.get(material_handle) {
                let mut m = base_mat.clone();
                // 初始残影亮度降低一点
                m.base_color.set_alpha(0.4);
                materials.add(m)
            } else {
                material_handle.0.clone()
            };

            commands.spawn((
                Mesh3d(mesh.0.clone()),
                MeshMaterial3d(ghost_material),
                Transform::from_translation(transform.translation)
                    .with_rotation(transform.rotation)
                    .with_scale(transform.scale),
                Ghost { ttl: 0.3 }, 
            ));
        }
    }
}

/// 残影淡出并销毁
fn cleanup_ghosts(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Ghost, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    for (entity, mut ghost, mat_handle) in query.iter_mut() {
        ghost.ttl -= time.delta_secs();
        if ghost.ttl <= 0.0 {
            commands.entity(entity).despawn_recursive();
        } else {
            // 让残影逐渐变透明 (如果材质允许)
            if let Some(mat) = materials.get_mut(mat_handle) {
                mat.base_color.set_alpha(ghost.ttl / 0.3 * 0.5);
            }
        }
    }
}

/// 更新持续旋转逻辑
fn update_rotations(
    mut query: Query<(&mut Transform, &Rotating)>,
    time: Res<Time>,
) {
    for (mut transform, rotating) in query.iter_mut() {
        transform.rotate_y(rotating.speed * time.delta_secs());
    }
}

/// 更新物理冲击效果（让立牌产生倾斜和弹回感）
fn update_physical_impacts(
    mut query: Query<(&mut Transform, &mut PhysicalImpact, &BreathAnimation)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (mut transform, mut impact, breath) in query.iter_mut() {
        // 1. 模拟旋转弹簧力
        let spring_k = 25.0; 
        let damping = 6.0;
        
        let force = -spring_k * impact.tilt_amount;
        impact.tilt_velocity += force * dt;
        impact.tilt_velocity *= 1.0 - (damping * dt);
        impact.tilt_amount += impact.tilt_velocity * dt;

        // 2. 模拟位置弹簧力 (将位移拉回 0)
        let pos_spring_k = 6.0; // 大幅降低刚度，允许长距离位移
        let pos_damping = 4.0;  // 降低阻尼，让位移更顺滑
        let pos_force = -pos_spring_k * impact.current_offset;
        impact.offset_velocity += pos_force * dt;
        impact.offset_velocity *= 1.0 - (pos_damping * dt);
        
        let delta_offset = impact.offset_velocity * dt;
        impact.current_offset += delta_offset;

        // 3. 模拟特殊回旋弹簧力
        let rot_spring_k = 40.0;
        let rot_damping = 5.0;
        let rot_force = -rot_spring_k * impact.special_rotation;
        impact.special_rotation_velocity += rot_force * dt;
        impact.special_rotation_velocity *= 1.0 - (rot_damping * dt);
        impact.special_rotation += impact.special_rotation_velocity * dt;

        // 4. 限制倾斜角度，防止“倒转”
        impact.tilt_amount = impact.tilt_amount.clamp(-0.8, 0.8);

        // 5. 整合呼吸动画 Y 轴偏移
        let breath_cycle = (breath.timer * breath.frequency).sin();
        let breath_y = breath_cycle * 0.05;

        // 6. 应用到变换
        // 旋转：俯视角 (-0.2) * 物理倾斜 (Tilt) * 特殊回旋 (Special)
        transform.rotation = Quat::from_rotation_x(-0.2) 
            * Quat::from_rotation_z(impact.tilt_amount)
            * Quat::from_rotation_z(impact.special_rotation);
        
        // 最终位置 = 初始位置 + 物理偏移 + 呼吸偏移
        transform.translation = impact.home_position + impact.current_offset + Vec3::new(0.0, breath_y, 0.0);
    }
}

/// 监听受击，触发物理反馈
fn trigger_hit_feedback(
    mut events: EventReader<CharacterAnimationEvent>,
    mut query: Query<(&mut PhysicalImpact, Option<&PlayerSpriteMarker>)>,
) {
    for event in events.read() {
        if let Ok((mut impact, is_player)) = query.get_mut(event.target) {
            let direction = if is_player.is_some() { 1.0 } else { -1.0 };
            
            match event.animation {
                AnimationState::Hit => {
                    // 受击：立牌向后猛飞
                    impact.tilt_velocity = 35.0 * direction; 
                    impact.offset_velocity = Vec3::new(-3.0 * direction, 0.0, 0.0);
                }
                AnimationState::Attack => {
                    // 攻击：角色瞬移冲向对手！速度提回 25.0
                    impact.tilt_velocity = -50.0 * direction;
                    impact.offset_velocity = Vec3::new(25.0 * direction, 0.0, 0.0);
                }
                AnimationState::ImperialSword => {
                    // 御剑术：极速冲锋 + 270 度暴力回旋
                    impact.tilt_velocity = -30.0 * direction;
                    impact.offset_velocity = Vec3::new(32.0 * direction, 0.0, 0.0);
                    // 给一个强大的回旋初速度 (方向根据角色面向)
                    impact.special_rotation_velocity = -45.0 * direction; 
                }
                AnimationState::DemonAttack => {
                    // 妖物突袭：速度稍慢但倾斜幅度极大，展现力量感
                    impact.tilt_velocity = -25.0 * direction;
                    impact.offset_velocity = Vec3::new(18.0 * direction, 0.0, 0.0);
                }
                AnimationState::DemonCast => {
                    // 施展妖术：原地高频震颤
                    impact.tilt_velocity = 80.0; // 极高角速度产生的震荡效果
                }
                _ => {}
            }
        }
    }
}

/// 更新呼吸动画（2.5D 纸片人：缩放计时 + 挤压伸展）
fn update_breath_animations(
    mut query: Query<(&mut Transform, &mut BreathAnimation)>,
    time: Res<Time>,
) {
    for (mut transform, mut breath) in query.iter_mut() {
        breath.timer += time.delta_secs();
        
        let breath_cycle = (breath.timer * breath.frequency).sin();
        
        // 2. 挤压与伸展 (Squash and Stretch)
        let stretch_y = 1.0 + breath_cycle * 0.03; 
        let squash_x = 1.0 - breath_cycle * 0.02;  
        
        transform.scale = Vec3::new(squash_x, stretch_y, 1.0);
    }
}

/// 同步系统：将 2D 纹理和颜色同步到 3D 纸片人上
fn sync_2d_to_3d_render(
    mut commands: Commands,
    sprite_query: Query<(Entity, &CharacterSprite, &Sprite, &Transform, Option<&Combatant3d>), (With<SpriteMarker>, Changed<CharacterSprite>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, char_sprite, _sprite_data, transform, combatant_3d) in sprite_query.iter() {
        if combatant_3d.is_none() {
            // 将 2D 像素位置转换为 3D 世界位置
            let mut x_3d = transform.translation.x / 100.0;
            let z_3d = transform.translation.y / 100.0;
            
            if x_3d < -5.0 { x_3d = -3.5; }
            if x_3d > 5.0 { x_3d = 3.5; }

            // 1. 创建角色立牌网格 (3:4 比例)
            let mesh = meshes.add(Rectangle::new(char_sprite.size.x / 50.0, char_sprite.size.y / 50.0));
            
            // 2. 创建材质 (还原高对比度和高饱和度)
            let material = materials.add(StandardMaterial {
                base_color: Color::WHITE,
                base_color_texture: Some(char_sprite.texture.clone()),
                // 移除自发光，改用强光照或完全不发光以保持对比度
                emissive: LinearRgba::BLACK, 
                // 禁用反射，防止灯光让立牌发白
                reflectance: 0.0,
                // 使用 Mask 模式保持边缘锐利
                alpha_mode: AlphaMode::Mask(0.5), 
                ..default()
            });

            // 3. 构造 3D 纸片人
            let home_pos = Vec3::new(x_3d, 1.0, z_3d + 0.1);
            commands.entity(entity).insert((
                Combatant3d { facing_right: true },
                BreathAnimation::default(),
                PhysicalImpact {
                    home_position: home_pos,
                    ..default()
                }, 
                Mesh3d(mesh),
                MeshMaterial3d(material),
                // 强制更新 3D 位置
                Transform::from_translation(home_pos)
                    .with_rotation(Quat::from_rotation_x(-0.2)), 
            )).remove::<Sprite>()
            .with_children(|parent| {
                // 4. 添加物理底座 (圆盘) - 降低亮度
                parent.spawn((
                    Mesh3d(meshes.add(Cylinder::new(0.8, 0.05))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(0.1, 0.1, 0.1), // 深色底座
                        metallic: 0.5,
                        perceptual_roughness: 0.8, // 增加粗糙度，减少反光
                        ..default()
                    })),
                    Transform::from_xyz(0.0, -char_sprite.size.y / 100.0, 0.0),
                ));
            });
        }
    }
}

/// 更新精灵动画
fn update_sprite_animations(
    _commands: Commands,
    mut query: Query<(&mut CharacterSprite, &Sprite)>,
    time: Res<Time>,
) {
    for (mut sprite, _image_sprite) in query.iter_mut() {
        // 跳过单帧动画
        if sprite.total_frames <= 1 {
            continue;
        }

        sprite.elapsed += time.delta_secs();

        // 检查是否需要切换到下一帧
        if sprite.elapsed >= sprite.frame_duration {
            sprite.elapsed -= sprite.frame_duration;
            sprite.current_frame += 1;

            // 检查动画是否结束
            if sprite.current_frame >= sprite.total_frames {
                if sprite.looping {
                    sprite.current_frame = 0;
                } else {
                    sprite.current_frame = sprite.total_frames - 1;

                    // 非循环动画结束后，恢复待机状态
                    match sprite.state {
                        AnimationState::Attack | AnimationState::Hit | AnimationState::ImperialSword | AnimationState::DemonAttack | AnimationState::DemonCast => {
                            sprite.set_idle();
                        }
                        AnimationState::Death => {
                            // 死亡动画结束后保持最后一帧
                        }
                        AnimationState::Idle => {}
                    }
                }
            }

            // TODO: 更新精灵图的纹理区域（实现 sprite sheet）
            // 目前使用单帧占位图，暂不更新
        }
    }
}

/// 处理动画事件
fn handle_animation_events(
    _commands: Commands,
    mut events: EventReader<CharacterAnimationEvent>,
    mut query: Query<&mut CharacterSprite>,
) {
    for event in events.read() {
        if let Ok(mut sprite) = query.get_mut(event.target) {
            match event.animation {
                AnimationState::Attack => {
                    sprite.set_attack(4, 0.3); // 4帧，0.3秒
                    info!("角色 {:?} 开始攻击动画", event.target);
                }
                AnimationState::ImperialSword => {
                    sprite.set_attack(8, 0.5); // 御剑术稍长，8帧，0.5秒
                    info!("角色 {:?} 开始御剑术回旋斩", event.target);
                }
                AnimationState::DemonAttack => {
                    sprite.set_attack(6, 0.4); 
                    info!("角色 {:?} 开始妖术突袭", event.target);
                }
                AnimationState::DemonCast => {
                    sprite.set_attack(4, 0.3);
                    info!("角色 {:?} 开始施展妖术", event.target);
                }
                AnimationState::Hit => {
                    sprite.set_hit(3, 0.2); // 3帧，0.2秒
                    info!("角色 {:?} 开始受击动画", event.target);
                }
                AnimationState::Death => {
                    sprite.set_death(6, 0.5); // 6帧，0.5秒
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

/// 创建角色精灵实体（带占位图）
pub fn spawn_character_sprite(
    commands: &mut Commands,
    character_assets: &CharacterAssets, // 增加参数
    character_type: CharacterType,
    position: Vec3,
    size: Vec2,
) -> Entity {
    // 根据角色类型选择颜色和贴图
    let (color, texture) = match character_type {
        CharacterType::Player => (Color::WHITE, character_assets.player_idle.clone()),
        CharacterType::DemonicWolf => (Color::WHITE, character_assets.wolf.clone()),
        CharacterType::PoisonSpider => (Color::WHITE, character_assets.spider.clone()),
        CharacterType::CursedSpirit => (Color::WHITE, character_assets.spirit.clone()),
        CharacterType::GreatDemon => (Color::WHITE, character_assets.boss.clone()),
    };

    let mut sprite = Sprite {
        color,
        custom_size: Some(size),
        anchor: Anchor::BottomCenter,
        ..default()
    };

    // 根据类型设置不同尺寸
    let sprite_size = match character_type {
        CharacterType::Player => Vec2::new(80.0, 120.0),
        CharacterType::DemonicWolf => Vec2::new(70.0, 100.0),
        CharacterType::PoisonSpider => Vec2::new(70.0, 100.0),
        CharacterType::CursedSpirit => Vec2::new(100.0, 140.0),
        CharacterType::GreatDemon => Vec2::new(150.0, 200.0),
    };

    sprite.custom_size = Some(sprite_size);

    let mut entity_cmd = commands.spawn((
        sprite,
        Transform::from_translation(position),
        GlobalTransform::default(),
        CharacterSprite::new(texture.clone(), sprite_size), // 这里要传入真正的贴图
        SpriteMarker,
    ));

    // 根据角色类型添加标记
    match character_type {
        CharacterType::Player => {
            entity_cmd.insert(PlayerSpriteMarker);
        }
        _ => {
            entity_cmd.insert(EnemySpriteMarker);
        }
    };

    entity_cmd.id()
}
