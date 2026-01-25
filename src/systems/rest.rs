//! 休息系统

use bevy::prelude::*;
use bevy::text::TextFont;
use crate::components::Player;
use crate::states::GameState;

/// 休息UI标记
#[derive(Component)]
pub struct RestUiRoot;

/// 继续按钮标记
#[derive(Component)]
pub struct RestContinueButton;

/// 休息插件
pub struct RestPlugin;

impl Plugin for RestPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Rest), setup_rest_ui)
            .add_systems(Update, handle_rest_interactions.run_if(in_state(GameState::Rest)))
            .add_systems(OnExit(GameState::Rest), cleanup_rest_ui);
    }
}

/// 设置休息UI
pub fn setup_rest_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut player_query: Query<&mut Player>,
) {
    info!("【休息系统】设置休息UI");

    // 获取当前玩家状态
    let (current_hp, max_hp) = if let Ok(player) = player_query.get_single() {
        (player.hp, player.max_hp)
    } else {
        (50, 50) // 默认值
    };

    // 计算恢复量（最大生命值的30%）
    let heal_amount = (max_hp as f32 * 0.3) as i32;
    let new_hp = (current_hp + heal_amount).min(max_hp);

    let chinese_font: Handle<Font> = asset_server.load("fonts/Arial Unicode.ttf");

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.15, 0.1)), // 绿色调背景
            RestUiRoot,
        ))
        .with_children(|parent| {
            // 标题
            parent.spawn((
                Text::new("休息"),
                TextFont {
                    font_size: 40.0,
                    font: chinese_font.clone(),
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.9, 0.6)),
            ));

            // 恢复效果描述
            parent.spawn((
                Text::new(format!("恢复生命值 +{}", heal_amount)),
                TextFont {
                    font_size: 28.0,
                    font: chinese_font.clone(),
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.8, 0.4)),
            ));

            // 生命值变化
            parent.spawn((
                Text::new(format!("{} → {}", current_hp, new_hp)),
                TextFont {
                    font_size: 36.0,
                    font: chinese_font.clone(),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // 提示信息
            parent.spawn((
                Text::new("按空格键或点击按钮继续"),
                TextFont {
                    font_size: 16.0,
                    font: chinese_font.clone(),
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
                Node {
                    margin: UiRect::top(Val::Px(10.0)),
                    ..default()
                },
            ));

            // 继续按钮
            parent.spawn((
                Button,
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.3, 0.6, 0.3)),
                BorderRadius::all(Val::Px(5.0)),
                RestContinueButton,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("继续"),
                    TextFont {
                        font_size: 24.0,
                        font: chinese_font,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
        });

    // 实际恢复生命值
    if let Ok(mut player) = player_query.get_single_mut() {
        player.heal(heal_amount);
        info!("【休息系统】玩家恢复生命值: {} → {}", current_hp, player.hp);
    }
}

/// 处理休息交互
pub fn handle_rest_interactions(
    mut next_state: ResMut<NextState<GameState>>,
    button_queries: Query<&Interaction, (Changed<Interaction>, With<RestContinueButton>)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // 点击继续按钮返回地图
    for interaction in button_queries.iter() {
        if matches!(interaction, Interaction::Pressed) {
            info!("【休息系统】返回地图");
            next_state.set(GameState::Map);
            return;
        }
    }

    // 按空格或回车键返回地图
    if keyboard_input.just_pressed(KeyCode::Space) || keyboard_input.just_pressed(KeyCode::Enter) {
        info!("【休息系统】按键返回地图");
        next_state.set(GameState::Map);
    }
}

/// 清理休息UI
pub fn cleanup_rest_ui(
    mut commands: Commands,
    ui_query: Query<Entity, With<RestUiRoot>>,
) {
    info!("【休息系统】清理休息UI");
    for entity in ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
