//! 遗物系统
//!
//! 处理遗物效果的触发和应用

use bevy::prelude::*;
use crate::components::*;
use crate::components::relic::{RelicUiMarker, RelicItemMarker};
use crate::states::GameState;

/// 遗物插件
pub struct RelicPlugin;

impl Plugin for RelicPlugin {
    fn build(&self, app: &mut App) {
        // 注册遗物获取事件
        app.add_event::<RelicObtainedEvent>();
        app.add_event::<RelicTriggeredEvent>();

        // 初始化遗物背包
        app.init_resource::<RelicCollection>();
        app.init_resource::<CombatStartProcessed>();

        // 遗物效果系统
        app.add_systems(Update, (
            trigger_relics_on_combat_start.run_if(in_state(GameState::Combat)),
            trigger_relics_on_turn_start.run_if(in_state(GameState::Combat)),
            trigger_relics_on_turn_end.run_if(in_state(GameState::Combat)),
            trigger_relics_on_draw.run_if(in_state(GameState::Combat)),
            trigger_relics_on_card_played.run_if(in_state(GameState::Combat)),
        ));
    }
}

// ============================================================================
// 遗物效果触发系统
// ============================================================================

/// 标记战斗开始时是否已处理过遗物
#[derive(Resource, Default)]
pub struct CombatStartProcessed {
    pub processed: bool,
}

/// 战斗开始时触发遗物效果
fn trigger_relics_on_combat_start(
    mut combat_start_processed: ResMut<CombatStartProcessed>,
    relic_collection: Res<RelicCollection>,
    mut enemy_query: Query<&mut Enemy>,
    mut player_query: Query<&mut Player>,
    mut hand_query: Query<&mut Hand>,
    mut draw_pile_query: Query<&mut DrawPile>,
    mut discard_pile_query: Query<&mut DiscardPile>,
) {
    // 防止重复触发
    if combat_start_processed.processed {
        return;
    }
    combat_start_processed.processed = true;

    info!("【遗物系统】战斗开始，触发遗物效果");

    for relic in &relic_collection.relic {
        match &relic.effect {
            RelicEffect::OnCombatStart { damage, block, draw_cards } => {
                // 造成伤害
                if *damage > 0 {
                    if let Ok(mut enemy) = enemy_query.get_single_mut() {
                        enemy.take_damage(*damage);
                        info!("  遗物 [{}] 触发：对敌人造成 {} 点伤害", relic.name, damage);
                    }
                }

                // 获得护甲
                if *block > 0 {
                    if let Ok(mut player) = player_query.get_single_mut() {
                        player.gain_block(*block);
                        info!("  遗物 [{}] 触发：获得 {} 点护甲", relic.name, block);
                    }
                }

                // 抽牌
                if *draw_cards > 0 {
                    let mut drawn = 0;
                    for _ in 0..*draw_cards {
                        if let Ok(mut draw_pile) = draw_pile_query.get_single_mut() {
                            if draw_pile.count == 0 {
                                if let Ok(mut discard_pile) = discard_pile_query.get_single_mut() {
                                    let cards = discard_pile.clear();
                                    if !cards.is_empty() {
                                        draw_pile.shuffle_from_discard(cards);
                                    }
                                }
                            }

                            if let Some(card) = draw_pile.draw_card() {
                                if let Ok(mut hand) = hand_query.get_single_mut() {
                                    if hand.add_card(card) {
                                        drawn += 1;
                                    }
                                }
                            }
                        }
                    }
                    info!("  遗物 [{}] 触发：抽了 {} 张牌", relic.name, drawn);
                }
            }
            _ => {}
        }
    }
}

/// 回合开始时触发遗物效果
fn trigger_relics_on_turn_start(
    relic_collection: Res<RelicCollection>,
    mut player_query: Query<&mut Player>,
) {
    for relic in &relic_collection.relic {
        match &relic.effect {
            RelicEffect::OnTurnStart { energy, .. } => {
                // 获得能量
                if *energy > 0 {
                    if let Ok(mut player) = player_query.get_single_mut() {
                        player.gain_energy(*energy);
                        info!("  遗物 [{}] 回合开始：获得 {} 点能量", relic.name, energy);
                    }
                }
            }
            _ => {}
        }
    }
}

/// 回合结束时触发遗物效果（如锚）
fn trigger_relics_on_turn_end(
    relic_collection: Res<RelicCollection>,
    hand_query: Query<&Hand>,
) {
    for relic in &relic_collection.relic {
        match &relic.effect {
            RelicEffect::OnTurnEnd { keep_cards } => {
                let hand = hand_query.get_single().unwrap();
                if hand.cards.len() > *keep_cards as usize {
                    info!("  遗物 [{}] 回合结束：保留最多 {} 张牌", relic.name, keep_cards);
                }
            }
            _ => {}
        }
    }
}

/// 抽牌时触发遗物效果
fn trigger_relics_on_draw(
    _relic_collection: Res<RelicCollection>,
) {
    // 额外抽牌逻辑在 draw_cards 系统中处理
}

/// 打出牌时触发遗物效果（如奇怪勺子）
fn trigger_relics_on_card_played(
    relic_collection: Res<RelicCollection>,
    mut card_played_count: Local<CardPlayedCount>,
    mut hand_query: Query<&mut Hand>,
    mut draw_pile_query: Query<&mut DrawPile>,
    mut discard_pile_query: Query<&mut DiscardPile>,
) {
    card_played_count.0 += 1;

    for relic in &relic_collection.relic {
        match &relic.effect {
            RelicEffect::OnCardPlayed { every_nth, draw_cards } => {
                if card_played_count.0 % *every_nth == 0 {
                    // 触发抽牌
                    let mut drawn = 0;
                    for _ in 0..*draw_cards {
                        if let Ok(mut draw_pile) = draw_pile_query.get_single_mut() {
                            if draw_pile.count == 0 {
                                if let Ok(mut discard_pile) = discard_pile_query.get_single_mut() {
                                    let cards = discard_pile.clear();
                                    if !cards.is_empty() {
                                        draw_pile.shuffle_from_discard(cards);
                                    }
                                }
                            }

                            if let Some(card) = draw_pile.draw_card() {
                                if let Ok(mut hand) = hand_query.get_single_mut() {
                                    if hand.add_card(card) {
                                        drawn += 1;
                                    }
                                }
                            }
                        }
                    }

                    if drawn > 0 {
                        info!("  遗物 [{}] 触发：第 {} 张牌，抽 {} 张", relic.name, card_played_count.0, drawn);
                    }
                }
            }
            _ => {}
        }
    }
}

/// 打出的牌计数（本地资源）
#[derive(Resource, Default)]
struct CardPlayedCount(pub i32);

// ============================================================================
// 遗物UI系统
// ============================================================================

/// 遗物UI插件
pub struct RelicUiPlugin;

impl Plugin for RelicUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Combat), setup_relic_ui)
            .add_systems(Update, update_relic_ui.run_if(in_state(GameState::Combat)))
            .add_systems(OnExit(GameState::Combat), cleanup_relic_ui);
    }
}

/// 设置遗物UI
fn setup_relic_ui(
    mut commands: Commands,
    relic_collection: Res<RelicCollection>,
) {
    info!("【遗物UI】设置遗物UI，遗物数量: {}", relic_collection.count());

    // 如果没有遗物，不创建UI
    if relic_collection.is_empty() {
        info!("【遗物UI】没有遗物，跳过UI创建");
        return;
    }

    // 创建遗物UI根节点（位于屏幕右侧）
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(20.0),
                top: Val::Px(100.0),
                width: Val::Px(200.0),
                height: Val::Auto,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
            BorderRadius::MAX,
            RelicUiMarker,
        ))
        .with_children(|parent| {
            // 标题
            parent.spawn((
                Text::new("遗物"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // 为每个遗物创建显示项
            for relic in &relic_collection.relic {
                create_relic_item(parent, relic);
            }
        });
}

/// 创建单个遗物显示项
fn create_relic_item(parent: &mut ChildBuilder, relic: &Relic) {
    let rarity_color = relic.rarity.color();
    let text_color = relic.rarity.text_color();

    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(8.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
            BackgroundColor(rarity_color),
            BorderRadius::all(Val::Px(4.0)),
            RelicItemMarker,
        ))
        .with_children(|parent| {
            // 遗物名称
            parent.spawn((
                Text::new(relic.name.clone()),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(text_color),
            ));

            // 遗物描述（小字）
            parent.spawn((
                Text::new(relic.description.clone()),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(text_color),
            ));
        });
}

/// 更新遗物UI（响应遗物获取）
fn update_relic_ui(
    relic_collection: Res<RelicCollection>,
    ui_query: Query<Entity, With<RelicUiMarker>>,
    item_query: Query<Entity, With<RelicItemMarker>>,
    mut commands: Commands,
) {
    // 检查遗物数量是否变化
    let current_item_count = item_query.iter().count();
    let expected_count = relic_collection.count();

    // 如果没有UI但有遗物，创建UI（处理运行时添加遗物的情况）
    let ui_entity = match ui_query.get_single() {
        Ok(entity) => Some(entity),
        Err(_) => {
            // UI不存在，如果有遗物就创建
            if !relic_collection.is_empty() {
                info!("【遗物UI】UI不存在但有遗物，创建UI");
                // 创建UI根节点
                let entity = commands
                    .spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            right: Val::Px(20.0),
                            top: Val::Px(100.0),
                            width: Val::Px(200.0),
                            height: Val::Auto,
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(10.0),
                            padding: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
                        BorderRadius::all(Val::Px(8.0)),
                        RelicUiMarker,
                    ))
                    .with_children(|parent| {
                        // 标题
                        parent.spawn((
                            Text::new("遗物"),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                        // 为每个遗物创建显示项
                        for relic in &relic_collection.relic {
                            create_relic_item(parent, relic);
                        }
                    })
                    .id();
                Some(entity)
            } else {
                None
            }
        }
    };

    if current_item_count == expected_count {
        return; // 没有变化，不需要更新
    }

    info!("【遗物UI】遗物数量变化: {} -> {}", current_item_count, expected_count);

    // 如果UI存在，重建遗物项
    if let Some(ui_entity) = ui_entity {
        // 重建遗物UI（简单实现：删除旧的，创建新的）
        // 先删除所有遗物项
        for entity in item_query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        // 如果有遗物，重新创建
        if !relic_collection.is_empty() {
            if let Some(mut ui_commands) = commands.get_entity(ui_entity) {
                for relic in &relic_collection.relic {
                    ui_commands.with_children(|parent| {
                        create_relic_item(parent, relic);
                    });
                }
            }
        }
    }
}

/// 清理遗物UI
fn cleanup_relic_ui(
    mut commands: Commands,
    ui_query: Query<Entity, With<RelicUiMarker>>,
) {
    info!("【遗物UI】清理遗物UI");

    for entity in ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
