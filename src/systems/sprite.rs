//! Sprite 角色渲染系统
//!
//! 处理战斗中的角色精灵显示和动画

use bevy::prelude::*;
use bevy::sprite::Anchor;
use crate::states::GameState;
use crate::components::sprite::{
    CharacterSprite, AnimationState, CharacterType,
    CharacterAnimationEvent, SpriteMarker, PlayerSpriteMarker, EnemySpriteMarker,
    Combatant3d, BreathAnimation, PhysicalImpact, CharacterAssets, Rotating, Ghost, ActionType,
    MagicSealMarker
};

pub struct SpritePlugin;

// ... 其他系统定义保持不变 ...

/// 更新法阵脉动效果 (大作级平缓呼吸)
fn update_magic_seal_pulse(
    query: Query<&MeshMaterial3d<StandardMaterial>, With<MagicSealMarker>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    for material_handle in query.iter() {
        if let Some(material) = materials.get_mut(material_handle) {
            // 极低频率 0.5, 微小振幅 0.15
            let pulse = 1.0 + (time.elapsed_secs() * 0.5).sin() * 0.15;
            material.emissive = LinearRgba::new(0.0, 0.8 * pulse, 0.3 * pulse, 1.0);
        }
    }
}

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
                spawn_ghosts,
                cleanup_ghosts,
                update_sprite_animations,
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
                // 确保残影也是双面的
                m.cull_mode = None; 
                m.double_sided = true;
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
        
        // 3. 处理动作计时器逻辑 (特化动作：如狼撕咬)
        let mut action_tilt_offset = 0.0;
        let mut action_pos_offset = Vec3::ZERO;
        
        if impact.action_timer > 0.0 {
            impact.action_timer -= dt;
            let dir = impact.action_direction; 
            
            // 计算动作进度 0.0 (开始) -> 1.0 (结束)
            let action_duration = match impact.action_type {
                ActionType::WolfBite => 1.0,
                ActionType::SpiderWeb => 0.8,
                _ => 1.0,
            };
            let progress = (1.0 - (impact.action_timer / action_duration)).clamp(0.0, 1.0);

            match impact.action_type {
                ActionType::WolfBite => {
                    // 贪狼：抛物线跳跃 + 2 次撕咬
                    let action_phase = impact.action_timer * 12.5;
                    action_tilt_offset = action_phase.sin() * 0.8;
                    
                    // 增加“扑杀”跳跃感 (Y轴弧线)
                    action_pos_offset.y = (progress * std::f32::consts::PI).sin() * 1.5;
                    
                    if (action_tilt_offset * dir) < 0.0 {
                        action_pos_offset.x = action_tilt_offset * 0.3; 
                    }
                    
                    pos_damping = 10.0; // 降低阻尼让位移更顺滑
                    impact.offset_velocity = Vec3::new(8.5 * dir, 0.0, 0.0); 
                },
                ActionType::SpiderWeb => {
                    // 蜘蛛：取消静止，保持匀速向前爬行
                    pos_damping = 5.0;
                    impact.offset_velocity = Vec3::new(5.0 * dir, 0.0, 0.0);
                },
                ActionType::DemonCast => {
                    // 施法：完全锁定 X 轴位移，仅允许原地震颤
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

        // 3.6 模拟特殊回旋弹簧力 (御剑术自转)
        let rot_spring_k = 45.0;
        let rot_damping = 6.0;
        let rot_force = -rot_spring_k * impact.special_rotation;
        impact.special_rotation_velocity += rot_force * dt;
        impact.special_rotation_velocity *= 1.0 - (rot_damping * dt);
        let delta_rot = impact.special_rotation_velocity * dt;
        impact.special_rotation += delta_rot;

        // 4. 限制倾斜角度
        impact.tilt_amount = impact.tilt_amount.clamp(-1.0, 1.0);

        // 6. 整合呼吸动画 Y 轴偏移 (动态抑制 - 极致可靠版)
        let breath_cycle = (breath.timer * breath.frequency).sin();
        
        // 关键逻辑：只要还在动作计时、或者位移没回弹到位，就绝对静止呼吸
        let is_acting = impact.action_timer > 0.0 || impact.current_offset.length() > 0.05 || impact.offset_velocity.length() > 0.5;
        // 降低呼吸幅度到 0.02 (2厘米)，使其更真实
        let breath_y = if is_acting { 0.0 } else { breath_cycle * 0.02 };

        // 7. 应用变换 (消除视觉晃动优化版)
        let tilt_suppression = 1.0 / (1.0 + impact.special_rotation.abs() * 5.0);
        let effective_tilt = impact.tilt_amount * tilt_suppression;

        // 狼的大作级扑杀逻辑 (空中回旋 + 前倾)
        let mut wolf_spin = 0.0;
        let mut wolf_forward_tilt = 0.0;
        if impact.action_timer > 0.0 && impact.action_type == ActionType::WolfBite {
            let action_duration = 1.0;
            let progress = (1.0 - (impact.action_timer / action_duration)).clamp(0.0, 1.0);
            
            // 1. 空中旋转 720 度 (2 圈)
            wolf_spin = progress * std::f32::consts::PI * 4.0;
            
            // 2. 增加一点起跳后的前倾感 (基于方向)
            let dir = impact.action_direction;
            wolf_forward_tilt = (progress * std::f32::consts::PI).sin() * 0.4 * dir;
        }

        // 旋转合成：
        // 增加 wolf_forward_tilt 到 Z 轴，wolf_spin 到 Y 轴
        transform.rotation = Quat::from_rotation_x(-0.2) 
            * Quat::from_rotation_z(effective_tilt + wolf_forward_tilt) 
            * Quat::from_rotation_y(impact.special_rotation + action_tilt_offset + wolf_spin);
        
        // 覆盖 translation
        transform.translation = impact.home_position + impact.current_offset + action_pos_offset + Vec3::new(0.0, breath_y, 0.0);
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
            impact.action_direction = direction; // 记录方向
            
            // 关键修复：如果正在执行长动作（如撕咬、吐丝），不要被普通的受击/攻击逻辑打断
            if impact.action_timer > 0.0 {
                continue;
            }

            match event.animation {
                AnimationState::Hit => {
                    impact.action_type = ActionType::None;
                    // 受击：力道减半，呈现沉重的顿挫感
                    impact.tilt_velocity = 15.0 * direction; 
                    impact.offset_velocity = Vec3::new(-2.0 * direction, 0.0, 0.0);
                }
                AnimationState::Attack => {
                    impact.action_type = ActionType::None;
                    // 普通攻击：略微削减速度
                    impact.tilt_velocity = -40.0 * direction;
                    impact.offset_velocity = Vec3::new(20.0 * direction, 0.0, 0.0);
                }
                AnimationState::ImperialSword => {
                    impact.action_type = ActionType::None;
                    // 御剑术：极速冲锋 + 270 度逆时针暴力回旋 (velocity改为正)
                    impact.tilt_velocity = -10.0 * direction; 
                    impact.offset_velocity = Vec3::new(28.0 * direction, 0.0, 0.0);
                    // 修正：正值代表逆时针
                    impact.special_rotation_velocity = 80.0 * direction; 
                }
                AnimationState::DemonAttack => {
                    // 妖物突袭：更具沉重感
                    impact.tilt_velocity = -20.0 * direction;
                    impact.offset_velocity = Vec3::new(12.0 * direction, 0.0, 0.0);
                }
                AnimationState::DemonCast => {
                    // 施展妖术：不再水平摇头，而是改为原地站立，通过后续逻辑实现能量脉冲缩放
                    impact.action_type = ActionType::DemonCast;
                    impact.tilt_velocity = 0.0; 
                    impact.special_rotation_velocity = 0.0; 
                    impact.action_timer = 0.6; // 稍微延长脉冲时长
                }
                AnimationState::WolfAttack => {
                    // 嗜血妖狼
                    impact.action_type = ActionType::WolfBite;
                    impact.tilt_velocity = -25.0 * direction;
                    impact.offset_velocity = Vec3::new(16.0 * direction, 0.0, 0.0);
                    impact.action_timer = 1.0; 
                }
                AnimationState::SpiderAttack => {
                    // 剧毒蛛
                    impact.action_type = ActionType::SpiderWeb;
                    impact.tilt_velocity = -8.0 * direction;
                    impact.offset_velocity = Vec3::new(10.0 * direction, 0.0, 0.0);
                    impact.action_timer = 0.8;
                }
                AnimationState::SpiritAttack => {
                    // 怨灵：灵体突袭 (快速漂浮 + 不稳定颤动)
                    impact.action_type = ActionType::None; // 幽灵暂不需要 action_timer 锁定
                    impact.tilt_velocity = 50.0; // 原地快速左右摆动
                    impact.offset_velocity = Vec3::new(22.0 * direction, 0.0, 0.0);
                    // 给一个极高的特殊旋转初速度，让灵体看起来在旋转
                    impact.special_rotation_velocity = 120.0; 
                }
                _ => {}
            }
        }
    }
}

/// 更新呼吸动画（2.5D 纸片人：缩放计时 + 挤压伸展）
fn update_breath_animations(
    mut query: Query<(&mut Transform, &mut BreathAnimation, &PhysicalImpact)>,
    time: Res<Time>,
) {
    for (mut transform, mut breath, impact) in query.iter_mut() {
        breath.timer += time.delta_secs();
        
        let is_acting = impact.action_timer > 0.0 || impact.current_offset.length() > 0.01 || impact.offset_velocity.length() > 0.1;
        
                        if is_acting {
        
                            // 动作期间
        
                            if impact.action_type == ActionType::DemonCast {
        
                                // 大作级优化：能量脉冲脉动效果
        
                                // 频率 35.0 (极快)，幅度 1.15 倍缩放
        
                                let pulse = 1.0 + (impact.action_timer * 35.0).sin().abs() * 0.15;
        
                                transform.scale = Vec3::splat(pulse);
        
                            } else {
        
                                // 其他动作保持标准比例
        
                                transform.scale = Vec3::ONE;
        
                            }
        
                        } else {
        
                    let breath_cycle = (breath.timer * breath.frequency).sin();
        
                    // 2. 挤压与伸展 (Squash and Stretch) - 调低比例
        
                    let stretch_y = 1.0 + breath_cycle * 0.015; // 从 3% 降到 1.5%
        
                    let squash_x = 1.0 - breath_cycle * 0.01;   // 从 2% 降到 1%
        
                    
        
                    transform.scale = Vec3::new(squash_x, stretch_y, 1.0);
        
                }
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
            
            // 2. 创建材质 (大作级高保真还原：原色输出)
            let material = materials.add(StandardMaterial {
                // 基础色设为中灰，防止过度曝光
                base_color: Color::WHITE,
                base_color_texture: Some(char_sprite.texture.clone()),
                // 核心：使用自发光纹理还原插画的高饱和度色彩，不受环境光干扰
                emissive: LinearRgba::WHITE, 
                emissive_texture: Some(char_sprite.texture.clone()),
                // 彻底禁用反射，解决“发灰”问题
                reflectance: 0.0,
                metallic: 0.0,
                perceptual_roughness: 1.0,
                alpha_mode: AlphaMode::Mask(0.5), 
                cull_mode: None,
                double_sided: true,
                ..default()
            });

            // 3. 构造 3D 纸片人 (落地修正：高度设为 0.05)
            let home_pos = Vec3::new(x_3d, 0.05, z_3d + 0.1);
            commands.entity(entity).insert((
                Combatant3d { facing_right: true },
                BreathAnimation::default(),
                PhysicalImpact {
                    home_position: home_pos,
                    ..default()
                }, 
                Mesh3d(mesh),
                MeshMaterial3d(material),
                // 关闭阴影投射，防止旋转时产生黑影 (Shadow Acne)
                bevy::pbr::NotShadowCaster,
                // 强制更新 3D 位置
                Transform::from_translation(home_pos)
                    .with_rotation(Quat::from_rotation_x(-0.2)), 
            )).remove::<Sprite>()
            .with_children(|parent| {
                // 4. 添加物理底座 (墨翠玉盘) - 优化为半透明发光质感
                parent.spawn((
                    Mesh3d(meshes.add(Cylinder::new(0.8, 0.02))), 
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgba(0.0, 0.05, 0.0, 0.4),
                        emissive: LinearRgba::new(0.0, 0.2, 0.1, 1.0),
                        metallic: 0.9,
                        perceptual_roughness: 0.1, 
                        alpha_mode: AlphaMode::Blend,
                        ..default()
                    })),
                    // 落地后，底座位置修正 (刚好贴地)
                    Transform::from_xyz(0.0, -0.02, 0.0),
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
                        AnimationState::Attack | AnimationState::Hit | AnimationState::ImperialSword | AnimationState::DemonAttack | AnimationState::DemonCast | AnimationState::WolfAttack | AnimationState::SpiderAttack | AnimationState::SpiritAttack => {
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
                AnimationState::WolfAttack => {
                    sprite.set_attack(10, 1.0); // 狼的啃咬动作较长
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
    character_assets: &CharacterAssets, 
    character_type: CharacterType,
    position: Vec3,
    size: Vec2,
    enemy_id: Option<u32>, // 增加可选 ID 参数
) -> Entity {
    // ... 保持原有逻辑 ...
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

    let sprite_size = match character_type {
        CharacterType::Player => Vec2::new(80.0, 120.0),
        CharacterType::DemonicWolf => Vec2::new(70.0, 100.0),
        CharacterType::PoisonSpider => Vec2::new(70.0, 100.0),
        CharacterType::CursedSpirit => Vec2::new(100.0, 140.0),
        CharacterType::GreatDemon => Vec2::new(150.0, 200.0),
    };

    let mut entity_cmd = commands.spawn((
        sprite,
        Transform::from_translation(position),
        GlobalTransform::default(),
        CharacterSprite::new(texture.clone(), sprite_size), 
        SpriteMarker,
    ));

    // 根据角色类型添加标记
    match character_type {
        CharacterType::Player => {
            entity_cmd.insert(PlayerSpriteMarker);
        }
        _ => {
            if let Some(id) = enemy_id {
                entity_cmd.insert(EnemySpriteMarker { id });
            }
        }
    };

    entity_cmd.id()
}
