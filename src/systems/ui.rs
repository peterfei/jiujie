use bevy::prelude::*;
use crate::components::combat::{DamageNumber, DamageEffectEvent, BlockIconMarker, BlockText, Player, Enemy, StatusIndicator};
use crate::states::GameState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DamageEffectEvent>();
        app.add_systems(Update, (
            spawn_damage_numbers,
            update_damage_numbers,
            update_block_visuals,
            update_status_indicators,
        ).run_if(in_state(GameState::Combat)));
    }
}

fn update_status_indicators(
    mut query: Query<(&StatusIndicator, &mut Text)>,
    player_query: Query<&Player>,
    enemy_query: Query<&Enemy>,
) {
    for (indicator, mut text) in query.iter_mut() {
        let mut status_parts = Vec::new();

        if let Ok(player) = player_query.get(indicator.owner) {
            if player.weakness > 0 { status_parts.push(format!("虚弱:{}", player.weakness)); }
            if player.vulnerable > 0 { status_parts.push(format!("易伤:{}", player.vulnerable)); }
            if player.poison > 0 { status_parts.push(format!("中毒:{}", player.poison)); }
        } else if let Ok(enemy) = enemy_query.get(indicator.owner) {
            if enemy.weakness > 0 { status_parts.push(format!("虚弱:{}", enemy.weakness)); }
            if enemy.vulnerable > 0 { status_parts.push(format!("易伤:{}", enemy.vulnerable)); }
            if enemy.poison > 0 { status_parts.push(format!("中毒:{}", enemy.poison)); }
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
