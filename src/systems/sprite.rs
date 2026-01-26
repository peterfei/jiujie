//! Sprite 角色渲染系统
//!
//! 处理战斗中的角色精灵显示和动画

use bevy::prelude::*;
use bevy::sprite::Anchor;
use crate::components::sprite::{
    CharacterSprite, AnimationState, CharacterType,
    CharacterAnimationEvent, SpriteMarker, PlayerSpriteMarker, EnemySpriteMarker,
    Combatant3d, BreathAnimation, PhysicalImpact, CharacterAssets, Rotating
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
            ).run_if(in_state(GameState::Combat))
        );
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
    mut query: Query<(&mut Transform, &mut PhysicalImpact)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (mut transform, mut impact) in query.iter_mut() {
        // 1. 模拟弹簧力将倾斜拉回 0
        let spring_k = 15.0; 
        let damping = 5.0;
        
        let force = -spring_k * impact.tilt_amount;
        impact.tilt_velocity += force * dt;
        impact.tilt_velocity *= 1.0 - (damping * dt);
        impact.tilt_amount += impact.tilt_velocity * dt;

        // 2. 模拟位置偏移弹回
        let pos_spring_k = 12.0;
        let pos_force = -pos_spring_k * impact.current_offset;
        impact.offset_velocity += pos_force * dt;
        impact.offset_velocity *= 1.0 - (damping * dt);
        
        let delta_offset = impact.offset_velocity * dt;
        impact.current_offset += delta_offset;

        // 3. 应用到变换
        // 保持垂直方向的呼吸动画与物理位移叠加
        transform.rotation = Quat::from_rotation_z(impact.tilt_amount) * Quat::from_rotation_x(-0.2);
    }
}

/// 监听受击，触发物理反馈
fn trigger_hit_feedback(
    mut events: EventReader<CharacterAnimationEvent>,
    mut query: Query<&mut PhysicalImpact>,
) {
    for event in events.read() {
        if let Ok(mut impact) = query.get_mut(event.target) {
            match event.animation {
                AnimationState::Hit => {
                    // 受击：立牌向后倒（正向倾斜），并伴随随机的小幅度震荡
                    impact.tilt_velocity = 8.0; 
                    impact.offset_velocity = Vec3::new(0.5, 0.2, 0.0);
                }
                AnimationState::Attack => {
                    // 攻击：立牌向前冲（负向倾斜）
                    impact.tilt_velocity = -12.0;
                    impact.offset_velocity = Vec3::new(-1.2, 0.0, 0.0);
                }
                _ => {}
            }
        }
    }
}

/// 更新呼吸动画（2.5D 纸片人上下浮动）
fn update_breath_animations(
    mut query: Query<(&mut Transform, &mut BreathAnimation)>,
    time: Res<Time>,
) {
    for (mut transform, mut breath) in query.iter_mut() {
        breath.timer += time.delta_secs();
        // 使用绝对值设置 Y 轴，而不是累加
        // 频率调低到 1.0 (一秒一个周期)，幅度调低到 0.02 (2厘米)
        let offset = (breath.timer * 1.0).sin() * 0.02;
        transform.translation.y = offset; 
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
            commands.entity(entity).insert((
                Combatant3d { facing_right: true },
                BreathAnimation::default(),
                PhysicalImpact::default(), 
                Mesh3d(mesh),
                MeshMaterial3d(material),
                // 强制更新 3D 位置
                // 增加 z 轴 0.1 的偏移，确保立牌在视觉上领先于任何背景元素
                Transform::from_xyz(x_3d, 1.0, z_3d + 0.1) 
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
                        AnimationState::Attack | AnimationState::Hit => {
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

    commands
        .spawn((
            sprite,
            Transform::from_translation(position),
            GlobalTransform::default(),
            CharacterSprite::new(texture.clone(), sprite_size), // 这里要传入真正的贴图
            SpriteMarker,
        ))
        .with_children(|parent| {
            // 根据角色类型添加不同的标记
            match character_type {
                CharacterType::Player => {
                    parent.spawn(PlayerSpriteMarker);
                }
                _ => {
                    parent.spawn(EnemySpriteMarker);
                }
            }
        })
        .id()
}
