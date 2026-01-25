//! 商店系统

use bevy::prelude::*;
use bevy::text::TextFont;
use crate::components::*;
use crate::components::shop::*;
use crate::components::relic::RelicCollection;
use crate::states::GameState;

const COLOR_GOLD: Color = Color::srgb(1.0, 0.84, 0.0);

/// 商店插件
pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentShopItems>()
            .init_resource::<SelectedCardForRemoval>();

        app.add_systems(OnEnter(GameState::Shop), setup_shop_ui)
            .add_systems(Update, (
                handle_shop_interactions,
                update_gold_display.after(handle_shop_interactions), // 确保在处理交互后更新UI
            ).run_if(in_state(GameState::Shop)))
            .add_systems(OnExit(GameState::Shop), cleanup_shop_ui);

        info!("【商店插件】ShopPlugin 已注册");
    }
}

// ============================================================================
// 商店UI系统
// ============================================================================

/// 设置商店UI
pub fn setup_shop_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut current_items: ResMut<CurrentShopItems>,
    player_deck: Res<PlayerDeck>,
    relic_collection: Res<RelicCollection>,
    mut player_query: Query<&mut Player>,
) {
    info!("【商店系统】设置商店UI");

    // 玩家实体由 init_player 系统统一管理
    // 如果玩家金币为0，给予初始金币
    let current_gold = if let Ok(mut player) = player_query.get_single_mut() {
        if player.gold == 0 {
            player.gold = 100;
            info!("【商店系统】玩家获得初始金币: 100");
        }
        player.gold
    } else {
        // 理论上不应该到达这里，因为 init_player 应该已经创建了 Player
        warn!("【商店系统】未找到玩家实体！init_player 应该已经创建了");
        100
    };

    current_items.items = generate_shop_items(&player_deck, &relic_collection);
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
            BackgroundColor(Color::srgb(0.1, 0.1, 0.15)),
            ShopUiRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("商店"),
                TextFont {
                    font_size: 40.0,
                    font: chinese_font.clone(),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                Text::new(format!("金币: {}", current_gold)),
                TextFont {
                    font_size: 24.0,
                    font: chinese_font.clone(),
                    ..default()
                },
                TextColor(COLOR_GOLD),
                ShopGoldText, // 标记金币文本，用于后续更新
            ));

            // 商品列表
            parent.spawn((
                Node {
                    width: Val::Percent(80.0),
                    height: Val::Percent(50.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(20.0),
                    row_gap: Val::Px(20.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.05, 0.05, 0.1)),
            ))
            .with_children(|items_parent| {
                let item_font = chinese_font.clone();
                for (index, item) in current_items.items.iter().enumerate() {
                    items_parent.spawn((
                        Node {
                            width: Val::Px(150.0),
                            height: Val::Px(200.0),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            padding: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
                        BorderRadius::all(Val::Px(8.0)),
                    ))
                    .with_children(|item_parent| {
                        // 商品名称
                        item_parent.spawn((
                            Text::new(item.get_name()),
                            TextFont {
                                font_size: 16.0,
                                font: item_font.clone(),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                        // 价格
                        item_parent.spawn((
                            Text::new(format!("{}金币", item.get_price())),
                            TextFont {
                                font_size: 14.0,
                                font: item_font.clone(),
                                ..default()
                            },
                            TextColor(COLOR_GOLD),
                        ));

                        // 购买按钮（带商品标记）
                        match item {
                            ShopItem::Card(_) => {
                                item_parent.spawn((
                                    Button,
                                    ShopCardButton { item_index: index },
                                    Interaction::None,
                                    Node {
                                        width: Val::Px(80.0),
                                        height: Val::Px(30.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.3, 0.6, 0.3)),
                                    BorderRadius::all(Val::Px(4.0)),
                                ))
                                .with_children(|btn_parent| {
                                    btn_parent.spawn((
                                        Text::new("购买"),
                                        TextFont {
                                            font_size: 14.0,
                                            font: item_font.clone(),
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                            }
                            ShopItem::Relic(_) => {
                                item_parent.spawn((
                                    Button,
                                    ShopRelicButton { item_index: index },
                                    Interaction::None,
                                    Node {
                                        width: Val::Px(80.0),
                                        height: Val::Px(30.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.3, 0.6, 0.3)),
                                    BorderRadius::all(Val::Px(4.0)),
                                ))
                                .with_children(|btn_parent| {
                                    btn_parent.spawn((
                                        Text::new("购买"),
                                        TextFont {
                                            font_size: 14.0,
                                            font: item_font.clone(),
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                            }
                            ShopItem::RemoveCard => {
                                item_parent.spawn((
                                    Button,
                                    ShopRemoveCardButton,
                                    Interaction::None,
                                    Node {
                                        width: Val::Px(80.0),
                                        height: Val::Px(30.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.3, 0.6, 0.3)),
                                    BorderRadius::all(Val::Px(4.0)),
                                ))
                                .with_children(|btn_parent| {
                                    btn_parent.spawn((
                                        Text::new("购买"),
                                        TextFont {
                                            font_size: 14.0,
                                            font: item_font.clone(),
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                            }
                        }
                    });
                }
            });

            // 返回地图按钮
            parent.spawn((
                Button,
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.3, 0.5, 0.3)),
                BorderRadius::all(Val::Px(5.0)),
                ShopExitButton,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("返回地图"),
                    TextFont {
                        font_size: 20.0,
                        font: chinese_font,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
        });

    info!("【商店系统】商店UI设置完成");
}

/// 生成商店商品
fn generate_shop_items(_player_deck: &PlayerDeck, _relic_collection: &RelicCollection) -> Vec<ShopItem> {
    info!("【商店系统】生成商店商品");

    let all_cards = crate::components::cards::CardPool::all_cards();
    let mut items = vec![];

    // 添加3张随机卡牌
    use rand::seq::SliceRandom;
    let mut rng = rand::thread_rng();

    // 选择3张不同的卡牌
    for card in all_cards.choose_multiple(&mut rng, 3) {
        items.push(ShopItem::Card(card.clone()));
    }

    // 添加移除卡牌服务
    items.push(ShopItem::RemoveCard);

    info!("【商店系统】生成了{}个商品", items.len());
    items
}

/// 处理商店交互
pub fn handle_shop_interactions(
    mut next_state: ResMut<NextState<GameState>>,
    mut player_query: Query<&mut Player>,
    current_items: Res<CurrentShopItems>,
    mut deck: ResMut<PlayerDeck>,
    mut relic_collection: ResMut<RelicCollection>,
    card_buttons: Query<(&Interaction, &ShopCardButton), Changed<Interaction>>,
    relic_buttons: Query<(&Interaction, &ShopRelicButton), Changed<Interaction>>,
    remove_buttons: Query<(&Interaction, &ShopRemoveCardButton), Changed<Interaction>>,
    exit_buttons: Query<(&Interaction, &ShopExitButton), Changed<Interaction>>,
) {
    // 处理返回地图按钮
    for (interaction, _) in exit_buttons.iter() {
        if matches!(interaction, Interaction::Pressed) {
            info!("【商店系统】返回地图");
            next_state.set(GameState::Map);
            return;
        }
    }

    // 处理卡牌购买
    for (interaction, shop_btn) in card_buttons.iter() {
        if matches!(interaction, Interaction::Pressed) {
            let item = &current_items.items[shop_btn.item_index];
            let price = item.get_price();

            if let Ok(mut player) = player_query.get_single_mut() {
                if player.gold >= price {
                    if let ShopItem::Card(card) = item {
                        player.gold -= price;
                        deck.add_card(card.clone());
                        info!("【商店系统】购买卡牌: {}, 价格: {}, 剩余金币: {}",
                              card.name, price, player.gold);
                    }
                } else {
                    info!("【商店系统】金币不足: 需要 {}, 拥有 {}", price, player.gold);
                }
            }
        }
    }

    // 处理遗物购买
    for (interaction, shop_btn) in relic_buttons.iter() {
        if matches!(interaction, Interaction::Pressed) {
            let item = &current_items.items[shop_btn.item_index];
            let price = item.get_price();

            if let Ok(mut player) = player_query.get_single_mut() {
                if player.gold >= price {
                    if let ShopItem::Relic(relic) = item {
                        player.gold -= price;
                        relic_collection.add_relic(relic.clone());
                        info!("【商店系统】购买遗物: {}, 价格: {}, 剩余金币: {}",
                              relic.name, price, player.gold);
                    }
                } else {
                    info!("【商店系统】金币不足: 需要 {}, 拥有 {}", price, player.gold);
                }
            }
        }
    }

    // 处理移除卡牌服务购买
    for (interaction, _) in remove_buttons.iter() {
        if matches!(interaction, Interaction::Pressed) {
            let price = ShopItem::RemoveCard.get_price();

            if let Ok(mut player) = player_query.get_single_mut() {
                if player.gold >= price {
                    player.gold -= price;
                    info!("【商店系统】购买移除卡牌服务, 价格: {}, 剩余金币: {}",
                          price, player.gold);
                    // TODO: 进入移除卡牌选择UI
                } else {
                    info!("【商店系统】金币不足: 需要 {}, 拥有 {}", price, player.gold);
                }
            }
        }
    }
}

/// 更新金币显示
pub fn update_gold_display(
    player_query: Query<&Player>,
    mut gold_text_query: Query<&mut Text, With<ShopGoldText>>,
) {
    // 调试：每帧打印一次状态（前5帧）
    static FRAME_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let frame = FRAME_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    if frame < 5 {
        info!("【商店UI】update_gold_display 系统运行 - 帧: {}", frame);
        let gold_text_count = gold_text_query.iter_mut().count();
        info!("【商店UI】ShopGoldText 组件数量: {}", gold_text_count);
        let player_count = player_query.iter().count();
        info!("【商店UI】Player 组件数量: {}", player_count);
    }

    // 每帧更新金币显示（当Player组件存在时）
    if let Ok(player) = player_query.get_single() {
        for mut text in gold_text_query.iter_mut() {
            let new_text = format!("金币: {}", player.gold);
            // 只在文本变化时更新
            if text.0 != new_text {
                info!("【商店UI】更新金币显示: {} -> {}", text.0, new_text);
                text.0 = new_text;
            }
        }
    }
}

/// 清理商店UI
pub fn cleanup_shop_ui(
    mut commands: Commands,
    ui_query: Query<Entity, With<ShopUiRoot>>,
) {
    info!("【商店系统】清理商店UI");
    for entity in ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
