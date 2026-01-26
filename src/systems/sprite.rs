//! Sprite 角色渲染系统
//!
//! 处理战斗中的角色精灵显示和动画

use bevy::prelude::*;
use bevy::sprite::Anchor;
use crate::components::sprite::{
    CharacterSprite, AnimationState, CharacterType,
    CharacterAnimationEvent, SpriteMarker, PlayerSpriteMarker, EnemySpriteMarker,
    Combatant3d, BreathAnimation
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
            ).run_if(in_state(GameState::Combat))
        );
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
    for (entity, char_sprite, sprite_data, transform, combatant_3d) in sprite_query.iter() {
        if combatant_3d.is_none() {
            // 将 2D 像素位置转换为 3D 世界位置 (缩小 100 倍)
            let mut x_3d = transform.translation.x / 100.0;
            let z_3d = transform.translation.y / 100.0; // 将 2D 的 Y 映射到 3D 的深度 Z
            
            // 修正：玩家在左，敌人在右
            // 我们稍微拉近一点距离
            if x_3d < -5.0 { x_3d = -3.0; }
            if x_3d > 5.0 { x_3d = 3.0; }

            let mesh = meshes.add(Rectangle::new(char_sprite.size.x / 100.0, char_sprite.size.y / 100.0));
            
            let material = materials.add(StandardMaterial {
                base_color: Color::WHITE,
                base_color_texture: Some(char_sprite.texture.clone()),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            });

            commands.entity(entity).insert((
                Combatant3d { facing_right: true },
                BreathAnimation::default(),
                Mesh3d(mesh),
                MeshMaterial3d(material),
                // 强制更新 3D 位置
                Transform::from_xyz(x_3d, 0.0, z_3d),
            )).remove::<Sprite>(); // 移除旧的 2D 占位块
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
    character_type: CharacterType,
    position: Vec3,
    size: Vec2,
) -> Entity {
    // 根据角色类型选择颜色
    let color = match character_type {
        CharacterType::Player => Color::srgb(0.2, 0.6, 1.0),  // 蓝色
        CharacterType::NormalEnemy => Color::srgb(1.0, 0.3, 0.3), // 红色
        CharacterType::EliteEnemy => Color::srgb(1.0, 0.5, 0.0),  // 橙色
        CharacterType::Boss => Color::srgb(0.8, 0.1, 0.8),        // 紫色
    };

    // 使用纯色矩形作为占位精灵
    let placeholder_color: Color = color;
    let mut sprite = Sprite {
        color: placeholder_color,
        custom_size: Some(size),
        anchor: Anchor::BottomCenter,
        ..default()
    };

    // 根据类型设置不同尺寸
    let sprite_size = match character_type {
        CharacterType::Player => Vec2::new(80.0, 120.0),
        CharacterType::NormalEnemy => Vec2::new(70.0, 100.0),
        CharacterType::EliteEnemy => Vec2::new(100.0, 140.0),
        CharacterType::Boss => Vec2::new(150.0, 200.0),
    };

    sprite.custom_size = Some(sprite_size);

    commands
        .spawn((
            sprite,
            Transform::from_translation(position),
            GlobalTransform::default(),
            CharacterSprite::new(Handle::default(), sprite_size),
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
