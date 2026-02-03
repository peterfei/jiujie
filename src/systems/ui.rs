use bevy::prelude::*;
use crate::components::combat::{DamageNumber, DamageEffectEvent, BlockIconMarker, BlockText, Player, Enemy, StatusIndicator, StatusEffectEvent};
use crate::states::GameState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DamageEffectEvent>();
        app.add_event::<StatusEffectEvent>();
        app.add_systems(Update, (
            spawn_damage_numbers,
            spawn_status_popups,
            update_damage_numbers,
            update_block_visuals,
            update_status_indicators,
        ).run_if(in_state(GameState::Combat)));

        // 支持商店中的漂字系统
        app.add_systems(Update, (
            spawn_status_popups,
            update_damage_numbers,
        ).run_if(in_state(GameState::Shop)));
    }
}

fn spawn_status_popups(
    mut commands: Commands,
    mut events: EventReader<StatusEffectEvent>,
    player_query: Query<&Transform, With<Player>>,
    enemy_query: Query<&Transform, With<Enemy>>,
) {
    for event in events.read() {
        // 尝试从 Transform 获取 2D 坐标
        let pos_opt = if let Ok(t) = player_query.get(event.target) {
            Some(t.translation.truncate())
        } else if let Ok(t) = enemy_query.get(event.target) {
            Some(t.translation.truncate())
        } else {
            None
        };

        let (ui_x, ui_y) = if let Some(pos) = pos_opt {
            (640.0 + pos.x, 360.0 - pos.y - 40.0)
        } else {
            // 兜底位置：屏幕中央偏上 (例如商店场景)
            (640.0, 200.0)
        };

        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(ui_x),
                top: Val::Px(ui_y),
                ..default()
            },
            Text::new(event.msg.clone()),
            TextFont { font_size: 30.0, ..default() },
            TextColor(event.color),
            DamageNumber {
                value: 0,
                timer: 0.0,
                lifetime: 1.5,
                velocity: Vec2::new(0.0, 30.0),
            },
            ZIndex(500), // 确保在商店 UI (300) 之上
        ));
    }
}

fn update_status_indicators(
    mut query: Query<(&mut StatusIndicator, &mut Text, Has<crate::components::PlayerUiMarker>)>,
    player_query: Query<(Entity, &Player)>,
    enemy_query: Query<&Enemy>,
) {
    let player_data = player_query.get_single().ok();

    for (mut indicator, mut text, is_player_ui) in query.iter_mut() {
        let mut status_parts = Vec::new();

        // 核心修复：如果是玩家UI但没有绑定实体，或者实体已失效，尝试重新绑定
        if is_player_ui {
            if let Some((p_ent, player)) = player_data {
                indicator.owner = p_ent; // 自动修复绑定
                if player.weakness > 0 { status_parts.push(format!("虚弱:{}", player.weakness)); }
                if player.vulnerable > 0 { status_parts.push(format!("易伤:{}", player.vulnerable)); }
                if player.poison > 0 { status_parts.push(format!("中毒:{}", player.poison)); }
            }
        } else {
            // 敌人逻辑保持不变
            if let Ok(enemy) = enemy_query.get(indicator.owner) {
                if enemy.weakness > 0 { status_parts.push(format!("虚弱:{}", enemy.weakness)); }
                if enemy.vulnerable > 0 { status_parts.push(format!("易伤:{}", enemy.vulnerable)); }
                if enemy.poison > 0 { status_parts.push(format!("中毒:{}", enemy.poison)); }
            }
        }

        text.0 = status_parts.join(" ");
    }
}

fn update_block_visuals(
    mut block_ui_query: Query<(&BlockIconMarker, &mut Node, &Children)>,
    mut text_query: Query<&mut Text, With<BlockText>>,
    player_query: Query<&Player>,
    enemy_query: Query<&Enemy>,
) {
    for (marker, mut node, children) in block_ui_query.iter_mut() {
        // 获取护甲值
        let block_value = if let Ok(player) = player_query.get(marker.owner) {
            player.block
        } else if let Ok(enemy) = enemy_query.get(marker.owner) {
            enemy.block
        } else {
            0
        };

        // 更新显示状态
        if block_value > 0 {
            if node.display == Display::None {
                info!("【UI调试】显示护甲图标: {}, 实体: {:?}", block_value, marker.owner);
            }
            node.display = Display::Flex;
            // 更新文字内容
            for &child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    text.0 = block_value.to_string();
                }
            }
        } else {
            node.display = Display::None;
        }
    }
}

fn spawn_damage_numbers(
    mut commands: Commands,
    mut events: EventReader<DamageEffectEvent>,
) {
    for event in events.read() {
        // 计算 UI 坐标 (基于 1280x720 逻辑分辨率)
        // 这里的 event.position 已经是恢复后的 2D 坐标 (x_world, y_world)
        let ui_x = 640.0 + event.position.x;
        let ui_y = 360.0 - event.position.y;

        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(ui_x),
                top: Val::Px(ui_y),
                ..default()
            },
            Text::new(format!("-{}", event.amount)),
            TextFont {
                font_size: 40.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 0.2, 0.2, 1.0)),
            DamageNumber::new(event.amount),
            ZIndex(100), // 确保在最上层
        ));
    }
}

fn update_damage_numbers(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DamageNumber, &mut Node, &mut TextColor, &mut TextFont)>,
    time: Res<Time>,
) {
    for (entity, mut dn, mut node, mut color, mut font) in query.iter_mut() {
        dn.timer += time.delta_secs();
        if dn.timer >= dn.lifetime {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        // 向上漂浮 (带一点减速)
        let t = dn.timer / dn.lifetime;
        let upward_speed = dn.velocity.y * (1.0 - t * 0.5);
        if let Val::Px(current_top) = node.top {
            node.top = Val::Px(current_top - upward_speed * time.delta_secs());
        }

        // 缩放动画 (先变大再缩小)
        let scale = if t < 0.2 {
            1.0 + (t / 0.2) * 0.5 // 前 20% 时间放大到 1.5 倍
        } else {
            1.5 - ((t - 0.2) / 0.8) * 0.5 // 后 80% 时间缩回到 1.0 倍
        };
        font.font_size = 40.0 * scale;

        // 淡出效果
        let alpha = 1.0 - t.powi(2); // 加速淡出
        color.0.set_alpha(alpha);
    }
}
