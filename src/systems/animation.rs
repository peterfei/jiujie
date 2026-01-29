//! 动画系统实现
//!
//! 提供敌人攻击动画效果的系统，包括：
//! - 移动动画更新
//! - 震动动画更新
//! - 浮动伤害数字更新
//! - 敌人攻击事件处理
//!
//! 地图视觉效果系统：
//! - 呼吸动画
//! - 脉冲发光
//! - 悬停交互

use bevy::prelude::*;
use bevy::ui::UiSystem;
use crate::components::animation::{
    FloatingDamageText,
    EnemyUiMarker, PlayerUiMarker, EasingFunction, EnemyAttackEvent
};
use crate::components::map::{BreathingAnimation, PulseAnimation, HoverEffect, MapNodeButton, OriginalSize, RippleEffect, EntranceAnimation, ConnectorDot, BreakthroughButtonMarker};
use crate::states::GameState;

/// 动画插件
///
/// 注册所有动画相关的系统和事件
pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        // 添加事件
        app.add_event::<EnemyAttackEvent>();

        // 添加系统 - 在UI布局之后运行
        app.add_systems(
            Update,
            (
                update_ui_movement_animations,
                update_ui_shake_animations,
                update_floating_damage_texts,
            ).after(UiSystem::Layout)
            .run_if(in_state(GameState::Combat))
        );
        app.add_systems(
            Update,
            handle_enemy_attack_events.run_if(in_state(GameState::Combat))
        );

        // 地图视觉效果系统
        app.add_systems(
            Update,
            (
                update_breathing_animations,
                update_pulse_animations,
                update_hover_effects,
                update_ripple_effects,
                update_entrance_animations,
                update_connector_dots,
            ).after(UiSystem::Layout)
            .run_if(in_state(GameState::Map))
        );
    }
}

/// UI移动动画数据（使用margin偏移）
#[derive(Component)]
struct UiMovementData {
    margin_top_offset: f32,
    target_offset: f32,
    progress: f32,
    duration: f32,
    elapsed: f32,
    return_on_complete: bool,
    is_returning: bool,
    easing: EasingFunction,
    original_margin_top: Val,
}

/// UI震动动画数据
#[derive(Component)]
struct UiShakeData {
    intensity: f32,
    duration: f32,
    elapsed: f32,
    frequency: f32,
    original_margin_top: Val,
    original_margin_left: Val,
}

/// 更新UI移动动画（通过修改margin）
fn update_ui_movement_animations(
    mut commands: Commands,
    mut query: Query<(Entity, &mut UiMovementData, &mut Node)>,
    time: Res<Time>,
) {
    for (entity, mut anim, mut node) in query.iter_mut() {
        anim.elapsed += time.delta_secs();

        // 计算进度
        let raw_progress = (anim.elapsed / anim.duration).min(1.0);
        anim.progress = anim.easing.apply(raw_progress);

        // 计算当前目标偏移
        let target_offset = if anim.is_returning { 0.0 } else { anim.target_offset };
        let current_offset = anim.margin_top_offset + (target_offset - anim.margin_top_offset) * anim.progress;

        // 应用到margin
        if let Val::Px(orig) = anim.original_margin_top {
            node.margin.top = Val::Px(orig + current_offset);
        }

        // 检查动画是否完成
        if raw_progress >= 1.0 {
            if anim.return_on_complete && !anim.is_returning {
                // 开始返回阶段
                anim.is_returning = true;
                anim.elapsed = 0.0;
                anim.progress = 0.0;
                anim.easing = EasingFunction::EaseInCubic;
            } else {
                // 动画完全结束，恢复原始位置并移除组件
                node.margin.top = anim.original_margin_top;
                commands.entity(entity).remove::<UiMovementData>();
            }
        }
    }
}

/// 更新UI震动动画
fn update_ui_shake_animations(
    mut commands: Commands,
    mut query: Query<(Entity, &mut UiShakeData, &mut Node)>,
    time: Res<Time>,
) {
    for (entity, mut anim, mut node) in query.iter_mut() {
        anim.elapsed += time.delta_secs();

        if anim.elapsed >= anim.duration {
            // 震动结束，恢复原始位置
            node.margin.top = anim.original_margin_top;
            node.margin.left = anim.original_margin_left;
            commands.entity(entity).remove::<UiShakeData>();
            continue;
        }

        // 计算当前震动强度（随时间衰减）
        let decay = 1.0 - (anim.elapsed / anim.duration);
        let current_intensity = anim.intensity * decay;

        // 生成随机偏移
        let time_seed = anim.elapsed * anim.frequency;
        let offset_x = (time_seed.sin() * current_intensity).max(-current_intensity).min(current_intensity);
        let offset_y = (time_seed.cos() * current_intensity * 0.7).max(-current_intensity).min(current_intensity);

        // 应用偏移到margin
        if let Val::Px(orig_x) = anim.original_margin_left {
            node.margin.left = Val::Px(orig_x + offset_x);
        }
        if let Val::Px(orig_y) = anim.original_margin_top {
            node.margin.top = Val::Px(orig_y + offset_y);
        }
    }
}

/// 更新浮动伤害数字
///
/// 处理所有带FloatingDamageText组件的实体，使它们向上浮动并淡出
fn update_floating_damage_texts(
    mut commands: Commands,
    mut query: Query<(Entity, &mut FloatingDamageText, &mut Node, &mut TextColor)>,
    time: Res<Time>,
) {
    for (entity, mut anim, mut node, mut text_color) in query.iter_mut() {
        anim.elapsed += time.delta_secs();

        // 计算进度
        let progress = (anim.elapsed / anim.duration).min(1.0);

        // 向上移动 - 修改margin.top
        let float_distance = anim.float_speed * anim.duration;
        if let Val::Px(start_top) = node.top {
            node.top = Val::Px(start_top - float_distance * progress);
        }

        // 淡出效果
        let alpha = anim.start_alpha + (anim.end_alpha - anim.start_alpha) * progress;
        let mut srgba = Srgba::try_from(text_color.0).unwrap_or_else(|_| Srgba::new(1.0, 1.0, 1.0, 1.0));
        srgba.alpha = alpha.clamp(0.0, 1.0);
        text_color.0 = srgba.into();

        // 动画完成，移除实体
        if progress >= 1.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// 处理敌人攻击事件
///
/// 当收到EnemyAttackEvent时，触发以下效果：
/// 1. 敌人UI向下冲刺并返回
/// 2. 玩家UI震动
/// 3. 显示浮动伤害数字
fn handle_enemy_attack_events(
    mut commands: Commands,
    mut events: EventReader<EnemyAttackEvent>,
    enemy_ui_query: Query<(Entity, &Node), With<EnemyUiMarker>>,
    player_ui_query: Query<(Entity, &Node), With<PlayerUiMarker>>,
    asset_server: Res<AssetServer>,
) {
    for event in events.read() {
        info!("处理敌人攻击事件：伤害={}, 破甲={}", event.damage, event.block_broken);

        // 1. 敌人冲刺动画（向下然后返回）
        if let Ok((enemy_entity, enemy_node)) = enemy_ui_query.get_single() {
            commands.entity(enemy_entity).insert(UiMovementData {
                margin_top_offset: 0.0,
                target_offset: 150.0, // 向下150像素
                progress: 0.0,
                duration: 0.2,
                elapsed: 0.0,
                return_on_complete: true,
                is_returning: false,
                easing: EasingFunction::EaseOutQuad,
                original_margin_top: enemy_node.margin.top,
            });
            info!("敌人冲刺动画已添加");
        } else {
            warn!("未找到敌人UI实体");
        }

        // 2. 玩家震动动画
        if let Ok((player_entity, player_node)) = player_ui_query.get_single() {
            commands.entity(player_entity).insert(UiShakeData {
                intensity: 25.0,  // 增大震动强度
                duration: 0.4,     // 增加持续时间
                elapsed: 0.0,
                frequency: 30.0,   // 增加震动频率
                original_margin_left: player_node.margin.left,
                original_margin_top: player_node.margin.top,
            });
            info!("玩家震动动画已添加");
        } else {
            warn!("未找到玩家UI实体");
        }

        // 3. 显示浮动伤害数字（在屏幕中央偏上位置显示）
        let chinese_font: Handle<Font> = asset_server.load("fonts/Arial Unicode.ttf");

        // 伤害文本颜色（破甲时显示特殊颜色）
        let damage_color: Color = if event.block_broken {
            Color::srgb(1.0, 0.3, 0.8) // 紫红色表示破甲
        } else {
            Color::srgb(1.0, 0.2, 0.2) // 红色表示普通伤害
        };

        // 破甲时添加额外文本
        let damage_text = if event.block_broken {
            format!("{} 破甲!", event.damage)
        } else {
            format!("-{}", event.damage)
        };

        info!("生成伤害数字: {}", damage_text);

        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0), // 屏幕水平居中
                    top: Val::Percent(45.0),  // 屏幕垂直偏上
                    width: Val::Auto,
                    height: Val::Auto,
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent
                    .spawn((
                        Text::new(damage_text),
                        TextFont {
                            font: chinese_font,
                            font_size: 48.0,
                            ..default()
                        },
                        TextColor(damage_color),
                    ))
                    .insert(FloatingDamageText::new(
                        80.0, // 每秒向上浮动 80 像素
                        1.0,  // 持续 1.0 秒
                    ));
            });

        info!("伤害数字已生成");
    }
}

// ============================================================================
// 地图视觉效果系统
// ============================================================================

/// 更新呼吸动画
///
/// 为未完成的节点添加平滑的缩放呼吸效果
fn update_breathing_animations(
    mut query: Query<(&mut BreathingAnimation, &OriginalSize, &mut Node), Without<BreakthroughButtonMarker>>,
    time: Res<Time>,
) {
    for (mut anim, original, mut node) in query.iter_mut() {
        anim.phase += time.delta_secs() * anim.speed;

        // 使用正弦波计算缩放值
        let normalized = (anim.phase.sin() + 1.0) / 2.0; // 0.0 到 1.0
        anim.current = anim.min_scale + (anim.max_scale - anim.min_scale) * normalized;

        // 应用缩放到 Node 的 width/height
        if let Val::Px(base_w) = original.width {
            node.width = Val::Px(base_w * anim.current);
        }
        if let Val::Px(base_h) = original.height {
            node.height = Val::Px(base_h * anim.current);
        }
    }
}

/// 更新脉冲发光动画
///
/// 为当前激活节点添加发光脉冲效果
fn update_pulse_animations(
    mut query: Query<(&mut PulseAnimation, &mut BorderColor)>,
    time: Res<Time>,
) {
    for (mut anim, mut border_color) in query.iter_mut() {
        anim.phase += time.delta_secs() * anim.speed;

        // 使用正弦波计算发光强度
        let pulse = (anim.phase.sin() + 1.0) / 2.0; // 0.0 到 1.0
        let intensity = anim.intensity * pulse;

        // 在白色和黄色之间脉冲
        let base_r = 1.0; let base_g = 1.0; let base_b = 1.0; // 白色
        let glow_r = 1.0; let glow_g = 0.9; let glow_b = 0.3;  // 黄色

        let r = base_r + (glow_r - base_r) * intensity;
        let g = base_g + (glow_g - base_g) * intensity;
        let b = base_b + (glow_b - base_b) * intensity;

        *border_color = BorderColor(Color::srgb(r, g, b));
    }
}

/// 更新悬停效果
///
/// 检测鼠标悬停状态并应用缩放效果
fn update_hover_effects(
    mut query: Query<(&HoverEffect, &Interaction, &OriginalSize, &mut Node), Without<BreakthroughButtonMarker>>,
) {
    for (effect, interaction, original, mut node) in query.iter_mut() {
        let target_scale = match interaction {
            Interaction::Hovered => effect.hover_scale,
            _ => effect.base_scale,
        };

        // 安全检查：target_scale 不能为 0
        if target_scale <= 0.0 { continue; }

        // 应用缩放到 Node 的 width/height
        if let Val::Px(base_w) = original.width {
            if base_w > 0.0 {
                node.width = Val::Px(base_w * target_scale);
            }
        }
        if let Val::Px(base_h) = original.height {
            if base_h > 0.0 {
                node.height = Val::Px(base_h * target_scale);
            }
        }
    }
}

/// 更新波纹特效
///
/// 节点点击时创建的波纹扩散动画
fn update_ripple_effects(
    mut commands: Commands,
    mut query: Query<(Entity, &mut RippleEffect, &mut Node, &mut BackgroundColor)>,
    time: Res<Time>,
) {
    for (entity, mut ripple, mut node, mut bg_color) in query.iter_mut() {
        ripple.elapsed += time.delta_secs();

        let progress = (ripple.elapsed / ripple.duration).min(1.0);

        // 扩散半径
        ripple.radius = ripple.max_radius * progress;

        // 淡出效果
        ripple.alpha = 1.0 - progress;

        // 更新波纹尺寸和透明度
        let diameter = ripple.radius * 2.0;
        node.width = Val::Px(diameter);
        node.height = Val::Px(diameter);

        // 更新背景颜色透明度
        if let Color::Srgba(srgba) = bg_color.0 {
            let mut new_srgba = Srgba::from(srgba);
            new_srgba.alpha = ripple.alpha.clamp(0.0, 1.0);
            *bg_color = BackgroundColor(new_srgba.into());
        }

        // 动画完成后移除波纹
        if progress >= 1.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// 更新入场动画
///
/// 节点首次出现时的淡入和缩放动画
fn update_entrance_animations(
    mut commands: Commands,
    mut query: Query<(Entity, &mut EntranceAnimation, &mut Node, &mut BackgroundColor, &OriginalSize), Without<BreakthroughButtonMarker>>,
    time: Res<Time>,
) {
    for (entity, mut anim, mut node, mut bg_color, original) in query.iter_mut() {
        anim.elapsed += time.delta_secs();

        let progress = (anim.elapsed / anim.duration).min(1.0);

        // 使用 easeOutCubic 缓动函数
        let eased_progress = 1.0 - (1.0 - progress).powi(3);

        // 计算当前缩放（从 start_scale 到 1.0）
        let current_scale = anim.start_scale + (1.0 - anim.start_scale) * eased_progress;

        // 计算当前透明度（从 start_alpha 到 1.0）
        let current_alpha = anim.start_alpha + (1.0 - anim.start_alpha) * eased_progress;

        // 应用缩放到 Node 的 width/height
        if let Val::Px(base_w) = original.width {
            node.width = Val::Px(base_w * current_scale);
        }
        if let Val::Px(base_h) = original.height {
            node.height = Val::Px(base_h * current_scale);
        }

        // 应用透明度
        if let Color::Srgba(srgba) = bg_color.0 {
            let mut new_srgba = Srgba::from(srgba);
            new_srgba.alpha = current_alpha.clamp(0.0, 1.0);
            *bg_color = BackgroundColor(new_srgba.into());
        }

        // 动画完成后移除入场动画组件
        if progress >= 1.0 {
            commands.entity(entity).remove::<EntranceAnimation>();
        }
    }
}

/// 更新连接圆点动画
///
/// 连接区域中的圆点脉冲效果，从左到右流动
fn update_connector_dots(
    mut query: Query<(&ConnectorDot, &mut BackgroundColor)>,
    time: Res<Time>,
) {
    let time_elapsed = time.elapsed_secs();

    for (dot, mut bg_color) in query.iter_mut() {
        // 计算脉冲效果，使用相位偏移创建流动感
        let phase = time_elapsed * 2.0 + dot.offset;
        let pulse = ((phase.sin() + 1.0) / 2.0).powf(2.0); // 平方使峰值更尖锐

        // 在 0.4 到 0.9 之间变化透明度
        let alpha = 0.4 + 0.5 * pulse;

        if let Color::Srgba(srgba) = bg_color.0 {
            let mut new_srgba = Srgba::from(srgba);
            new_srgba.alpha = alpha.clamp(0.0, 1.0);
            *bg_color = BackgroundColor(new_srgba.into());
        }
    }
}
