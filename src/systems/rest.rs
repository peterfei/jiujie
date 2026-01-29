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

/// 功法精进按钮
#[derive(Component)]
pub struct UpgradeButton;

/// 离去按钮标记
#[derive(Component)]
pub struct LeaveButton;

/// 选项区域标记（用于隐藏）
#[derive(Component)]
pub struct ChoiceArea;

/// 结果文本标记
#[derive(Component)]
pub struct ResultText;

/// 继续按钮标记
#[derive(Component)]
pub struct RestContinueButton;

/// 休息插件
pub struct RestPlugin;

impl Plugin for RestPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Rest), setup_rest_ui)
            .add_systems(Update, (
                handle_rest_interactions,
                handle_leave_interaction,
            ).run_if(in_state(GameState::Rest)))
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
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(50.0),
                    ..default()
                },
                ChoiceArea,
            )).with_children(|choice_area| {
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

                // 2. 进阶
                choice_area.spawn((
                    Button,
                    UpgradeButton,
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
                        Text::new("功法精进"),
                        TextFont { font_size: 28.0, font: chinese_font.clone(), ..default() },
                        TextColor(Color::WHITE),
                    ));
                    btn.spawn((
                        Text::new("磨砺招式，提升功法威力"),
                        TextFont { font_size: 14.0, font: chinese_font.clone(), ..default() },
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    ));
                });
            });

            // 结果展示区域 (初始隐藏)
            parent.spawn((
                Node {
                    display: Display::None,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(30.0),
                    ..default()
                },
                ResultText,
            )).with_children(|res| {
                res.spawn((
                    Text::new("功法精进成功！"),
                    TextFont { font_size: 32.0, font: chinese_font.clone(), ..default() },
                    TextColor(Color::srgb(1.0, 0.8, 0.3)),
                ));
                
                res.spawn((
                    Button,
                    LeaveButton,
                    Node {
                        padding: UiRect::axes(Val::Px(40.0), Val::Px(15.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                    BorderRadius::all(Val::Px(10.0)),
                )).with_children(|btn| {
                    btn.spawn((
                        Text::new("离开洞府"),
                        TextFont { font_size: 24.0, font: chinese_font.clone(), ..default() },
                        TextColor(Color::WHITE),
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
    mut player_query: Query<&mut Player>,
    mut player_deck: ResMut<crate::components::PlayerDeck>,
    breath_buttons: Query<&Interaction, (Changed<Interaction>, With<BreathButton>)>,
    upgrade_buttons: Query<&Interaction, (Changed<Interaction>, With<UpgradeButton>)>,
    mut choice_area_query: Query<&mut Node, (With<ChoiceArea>, Without<ResultText>)>,
    mut result_area_query: Query<(&mut Node, Entity), (With<ResultText>, Without<ChoiceArea>)>,
    mut text_query: Query<&mut Text>,
    children_query: Query<&Children>,
) {
    let player = player_query.get_single().expect("必须有玩家实体");

    let mut trigger_result = |msg: String| {
        // 1. 隐藏选项
        if let Ok(mut node) = choice_area_query.get_single_mut() {
            node.display = Display::None;
        }
        // 2. 显示结果
        if let Ok((mut node, entity)) = result_area_query.get_single_mut() {
            node.display = Display::Flex;
            
            // 3. 寻找结果文本实体并更新内容
            if let Ok(children) = children_query.get(entity) {
                if let Some(&text_entity) = children.first() {
                    if let Ok(mut text) = text_query.get_mut(text_entity) {
                        text.0 = msg;
                    }
                }
            }
        }
    };

    // 处理调息
    for interaction in breath_buttons.iter() {
        if matches!(interaction, Interaction::Pressed) {
            let heal_amount = (player.max_hp as f32 * 0.3) as i32;
            // [关键修复] 立即执行治疗
            if let Ok(mut player_mut) = player_query.get_single_mut() {
                player_mut.heal(heal_amount);
            }
            trigger_result(format!("运转周天，恢复了 {} 点道行！", heal_amount));
            return;
        }
    }

    // 处理功法进阶
    for interaction in upgrade_buttons.iter() {
        if matches!(interaction, Interaction::Pressed) {
            // 找出所有未进阶的卡牌
            let upgradable_indices: Vec<usize> = player_deck.cards.iter()
                .enumerate()
                .filter(|(_, card)| !card.upgraded)
                .map(|(i, _)| i)
                .collect();

            if !upgradable_indices.is_empty() {
                use rand::seq::SliceRandom;
                let mut rng = rand::thread_rng();
                if let Some(&index) = upgradable_indices.choose(&mut rng) {
                    let old_name = player_deck.cards[index].name.clone();
                    player_deck.cards[index].upgrade();
                    let new_name = player_deck.cards[index].name.clone();
                    trigger_result(format!("{} 已进阶为 {}！", old_name, new_name));
                }
            } else {
                trigger_result("已无功法可进阶".to_string());
            }
            return;
        }
    }
}

/// 处理离开交互
pub fn handle_leave_interaction(
    mut next_state: ResMut<NextState<GameState>>,
    mut map_progress: ResMut<crate::components::map::MapProgress>,
    leave_buttons: Query<&Interaction, (Changed<Interaction>, With<LeaveButton>)>,
    player_query: Query<(&crate::components::Player, &crate::components::Cultivation)>,
    player_deck: Res<crate::components::PlayerDeck>,
    relic_collection: Res<crate::components::relic::RelicCollection>,
) {
    for interaction in leave_buttons.iter() {
        if matches!(interaction, Interaction::Pressed) {
            // 只有在没切换状态时才执行
            info!("【洞府闭关】玩家点击离开，正在结算...");
            
            // 标记当前节点为完成
            map_progress.complete_current_node();

            // --- [关键修复] 离开洞府时强制执行一次存档，防止无限刷级 ---
            if let Ok((player, cultivation)) = player_query.get_single() {
                let save = crate::resources::save::GameStateSave {
                    player: player.clone(),
                    cultivation: cultivation.clone(),
                    deck: player_deck.cards.clone(),
                    relics: relic_collection.relic.clone(),
                    map_nodes: map_progress.nodes.clone(),
                    current_map_node_id: map_progress.current_node_id,
                    current_map_layer: map_progress.current_layer,
                };
                let _ = save.save_to_disk();
                info!("【存档系统】闭关结束，状态已持久化");
            }
            
            // 切换状态
            next_state.set(GameState::Map);
            return; // 立即返回，防止在同一帧处理其他点击
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
