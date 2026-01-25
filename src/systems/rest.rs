//! 休息系统

use bevy::prelude::*;
use bevy::text::TextFont;
use crate::components::Player;
use crate::states::GameState;

/// 休息UI标记
#[derive(Component)]
pub struct RestUiRoot;

/// 调息恢复按钮
#[derive(Component)]
pub struct BreathButton;

/// 打坐悟道按钮
#[derive(Component)]
pub struct InsightButton;

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
    player_query: Query<(&Player, &crate::components::Cultivation)>,
) {
    info!("【洞府闭关】设置闭关UI");

    let chinese_font: Handle<Font> = asset_server.load("fonts/Arial Unicode.ttf");
    let (player, cultivation) = player_query.get_single().expect("必须有玩家实体");

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(40.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.02, 0.05, 0.02)), // 深邃闭关背景
            RestUiRoot,
        ))
        .with_children(|parent| {
            // 标题
            parent.spawn((
                Text::new("洞 府 闭 关"),
                TextFont {
                    font_size: 52.0,
                    font: chinese_font.clone(),
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.9, 0.6)),
            ));

            // 状态显示
            parent.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(40.0),
                ..default()
            }).with_children(|p| {
                p.spawn((
                    Text::new(format!("当前道行: {}/{}", player.hp, player.max_hp)),
                    TextFont { font_size: 20.0, font: chinese_font.clone(), ..default() },
                    TextColor(Color::srgb(0.8, 0.5, 0.5)),
                ));
                p.spawn((
                    Text::new(format!("当前感悟: {}", cultivation.insight)),
                    TextFont { font_size: 20.0, font: chinese_font.clone(), ..default() },
                    TextColor(Color::srgb(0.5, 0.8, 0.8)),
                ));
            });

            // 选项区域
            parent.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(50.0),
                ..default()
            }).with_children(|choice_area| {
                // 1. 调息
                choice_area.spawn((
                    Button,
                    BreathButton,
                    Node {
                        width: Val::Px(240.0),
                        height: Val::Px(180.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(15.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.2, 0.1, 0.8)),
                    BorderRadius::all(Val::Px(15.0)),
                )).with_children(|btn| {
                    btn.spawn((
                        Text::new("调息恢复"),
                        TextFont { font_size: 28.0, font: chinese_font.clone(), ..default() },
                        TextColor(Color::WHITE),
                    ));
                    btn.spawn((
                        Text::new("运转周天，恢复 30% 道行"),
                        TextFont { font_size: 14.0, font: chinese_font.clone(), ..default() },
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    ));
                });

                // 2. 悟道
                choice_area.spawn((
                    Button,
                    InsightButton,
                    Node {
                        width: Val::Px(240.0),
                        height: Val::Px(180.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(15.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.2, 0.8)),
                    BorderRadius::all(Val::Px(15.0)),
                )).with_children(|btn| {
                    btn.spawn((
                        Text::new("打坐感悟"),
                        TextFont { font_size: 28.0, font: chinese_font.clone(), ..default() },
                        TextColor(Color::WHITE),
                    ));
                    btn.spawn((
                        Text::new("静思己过，获得 20 点感悟"),
                        TextFont { font_size: 14.0, font: chinese_font.clone(), ..default() },
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    ));
                });
            });

            // 离去提示（默认隐藏，点击后可能需要返回地图，但这里我们采用二选一即退出的策略）
            parent.spawn((
                Text::new("请择一机缘而行"),
                TextFont { font_size: 18.0, font: chinese_font.clone(), ..default() },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
        });
}

/// 处理休息交互
pub fn handle_rest_interactions(
    mut next_state: ResMut<NextState<GameState>>,
    mut player_query: Query<(&mut Player, &mut crate::components::Cultivation)>,
    breath_buttons: Query<&Interaction, (Changed<Interaction>, With<BreathButton>)>,
    insight_buttons: Query<&Interaction, (Changed<Interaction>, With<InsightButton>)>,
) {
    let (mut player, mut cultivation) = player_query.get_single_mut().expect("必须有玩家实体");

    // 处理调息
    for interaction in breath_buttons.iter() {
        if matches!(interaction, Interaction::Pressed) {
            let heal_amount = (player.max_hp as f32 * 0.3) as i32;
            player.heal(heal_amount);
            info!("【洞府闭关】调息恢复: 恢复 {} 点道行", heal_amount);
            next_state.set(GameState::Map);
            return;
        }
    }

    // 处理悟道
    for interaction in insight_buttons.iter() {
        if matches!(interaction, Interaction::Pressed) {
            cultivation.gain_insight(20);
            info!("【洞府闭关】打坐感悟: 获得 20 点感悟");
            next_state.set(GameState::Map);
            return;
        }
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
