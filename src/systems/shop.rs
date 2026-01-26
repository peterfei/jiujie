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
    info!("【仙家坊市】设置坊市UI");

    let current_gold = if let Ok(player) = player_query.get_single() {
        player.gold
    } else {
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
                row_gap: Val::Px(25.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.02, 0.05, 0.02)), // 更深邃的墨绿背景
            ShopUiRoot,
        ))
        .with_children(|parent| {
            // 坊市大标题
            parent.spawn((
                Text::new("仙 家 坊 市"),
                TextFont {
                    font_size: 52.0,
                    font: chinese_font.clone(),
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.9, 0.6)),
            ));

            // 灵石存量
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    ..default()
                },
            )).with_children(|p| {
                p.spawn((
                    Text::new("所持灵石:"),
                    TextFont { font_size: 24.0, font: chinese_font.clone(), ..default() },
                    TextColor(Color::srgb(0.7, 0.7, 0.7)),
                ));
                p.spawn((
                    Text::new(format!("{}", current_gold)),
                    TextFont { font_size: 28.0, font: chinese_font.clone(), ..default() },
                    TextColor(COLOR_GOLD),
                    ShopGoldText,
                ));
            });

            // 商品货架
            parent.spawn((
                Node {
                    width: Val::Percent(90.0),
                    height: Val::Percent(60.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(25.0),
                    row_gap: Val::Px(25.0),
                    ..default()
                },
            ))
            .with_children(|items_parent| {
                let item_font = chinese_font.clone();
                for (index, item) in current_items.items.iter().enumerate() {
                    items_parent.spawn((
                        Node {
                            width: Val::Px(190.0),
                            height: Val::Px(260.0),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            padding: UiRect::all(Val::Px(15.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.05, 0.1, 0.05, 0.9)),
                        BorderColor(Color::srgb(0.3, 0.4, 0.3)),
                        BorderRadius::all(Val::Px(12.0)),
                    ))
                    .with_children(|item_parent| {
                        // 商品名称
                        item_parent.spawn((
                            Text::new(item.get_name()),
                            TextFont { font_size: 22.0, font: item_font.clone(), ..default() },
                            TextColor(Color::WHITE),
                        ));

                        // 如果是卡牌，展示插画 (Nano Banana 风格优化)
                        if let ShopItem::Card(card) = item {
                            item_parent.spawn((
                                ImageNode::new(asset_server.load(card.image_path.clone())),
                                Node {
                                    width: Val::Px(120.0),
                                    height: Val::Px(140.0),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BorderColor(Color::srgba(1.0, 1.0, 1.0, 0.1)),
                            ));
                        }

                        // 描述
                        item_parent.spawn((
                            Text::new(item.get_description()),
                            TextFont { font_size: 13.0, font: item_font.clone(), ..default() },
                            TextColor(Color::srgb(0.6, 0.6, 0.6)),
                            Node { max_width: Val::Px(160.0), ..default() },
                        ));

                        // 价格
                        item_parent.spawn((
                            Text::new(format!("{} 灵石", item.get_price())),
                            TextFont { font_size: 18.0, font: item_font.clone(), ..default() },
                            TextColor(COLOR_GOLD),
                        ));

                        // 动作按钮
                        let action_text = match item {
                            ShopItem::Card(_) => "参悟",
                            ShopItem::Relic(_) => "求取",
                            ShopItem::Elixir { .. } => "服下",
                            ShopItem::ForgetTechnique => "了断",
                        };

                        item_parent.spawn((
                            Button,
                            ShopCardButton { item_index: index },
                            Node {
                                width: Val::Px(90.0),
                                height: Val::Px(35.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.4, 0.2)),
                            BorderRadius::all(Val::Px(6.0)),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new(action_text),
                                TextFont { font_size: 16.0, font: item_font.clone(), ..default() },
                                TextColor(Color::WHITE),
                            ));
                        });
                    });
                }
            });

            // 离去按钮
            parent.spawn((
                Button,
                Node {
                    width: Val::Px(180.0),
                    height: Val::Px(45.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.25, 0.35, 0.25)),
                BorderRadius::all(Val::Px(8.0)),
                ShopExitButton,
            ))
            .with_children(|p| {
                p.spawn((
                    Text::new("离开坊市"),
                    TextFont { font_size: 22.0, font: chinese_font.clone(), ..default() },
                    TextColor(Color::WHITE),
                ));
            });
        });

    info!("【商店系统】商店UI设置完成");
}

/// 生成商店商品
fn generate_shop_items(_player_deck: &PlayerDeck, _relic_collection: &RelicCollection) -> Vec<ShopItem> {
    info!("【仙家坊市】生成机缘商品");

    let all_cards = crate::components::cards::CardPool::all_cards();
    let mut items = vec![];
    use rand::seq::SliceRandom;
    let mut rng = rand::thread_rng();

    // 1. 随机添加 3 张功法（卡牌）
    for card in all_cards.choose_multiple(&mut rng, 3) {
        items.push(ShopItem::Card(card.clone()));
    }

    // 2. 随机添加 1 个法宝（遗物）
    // TODO: 目前遗物池较小，暂不随机，或直接添加一个
    
    // 3. 随机添加 1 颗灵丹
    let elixirs = vec![
        ShopItem::Elixir {
            name: "洗髓丹".to_string(),
            hp_restore: 20,
            price: 40,
            description: "洗筋伐髓，恢复 20 点道行".to_string(),
        },
        ShopItem::Elixir {
            name: "聚灵散".to_string(),
            hp_restore: 10,
            price: 25,
            description: "聚拢灵气，恢复 10 点道行".to_string(),
        },
        ShopItem::Elixir {
            name: "九转还魂丹".to_string(),
            hp_restore: 50,
            price: 90,
            description: "生死肉骨，恢复 50 点道行".to_string(),
        },
    ];
    if let Some(elixir) = elixirs.choose(&mut rng) {
        items.push(elixir.clone());
    }

    // 4. 固定提供“遗忘功法”服务
    items.push(ShopItem::ForgetTechnique);

    info!("【仙家坊市】机缘已至，共生成了 {} 个商品", items.len());
    items
}

/// 处理商店交互
pub fn handle_shop_interactions(
    mut next_state: ResMut<NextState<GameState>>,
    mut player_query: Query<&mut Player>,
    current_items: Res<CurrentShopItems>,
    mut deck: ResMut<PlayerDeck>,
    mut relic_collection: ResMut<RelicCollection>,
    mut map_progress: ResMut<MapProgress>, // 引入地图进度
    card_buttons: Query<(&Interaction, &ShopCardButton), Changed<Interaction>>,
    relic_buttons: Query<(&Interaction, &ShopRelicButton), Changed<Interaction>>,
    remove_buttons: Query<(&Interaction, &ShopRemoveCardButton), Changed<Interaction>>,
    exit_buttons: Query<(&Interaction, &ShopExitButton), Changed<Interaction>>,
) {
    // 处理返回地图按钮
    for (interaction, _) in exit_buttons.iter() {
        if matches!(interaction, Interaction::Pressed) {
            info!("【商店系统】离开坊市，因缘已了");
            // 关键修复：离开商店时标记节点完成
            map_progress.complete_current_node();
            next_state.set(GameState::Map);
            return;
        }
    }

    // 处理卡牌与灵丹购买
    for (interaction, shop_btn) in card_buttons.iter() {
        if matches!(interaction, Interaction::Pressed) {
            let item = &current_items.items[shop_btn.item_index];
            let price = item.get_price();

            if let Ok(mut player) = player_query.get_single_mut() {
                if player.gold >= price {
                    match item {
                        ShopItem::Card(card) => {
                            player.gold -= price;
                            deck.add_card(card.clone());
                            info!("【仙家坊市】换取功法: {}, 消耗灵石: {}, 剩余灵石: {}",
                                  card.name, price, player.gold);
                        }
                        ShopItem::Elixir { name, hp_restore, .. } => {
                            player.gold -= price;
                            player.hp = (player.hp + hp_restore).min(player.max_hp);
                            info!("【仙家坊市】服下 {}: 恢复 {} 点道行, 消耗灵石: {}, 剩余灵石: {}",
                                  name, hp_restore, price, player.gold);
                        }
                        _ => {}
                    }
                } else {
                    info!("【仙家坊市】灵石不足: 需要 {}, 拥有 {}", price, player.gold);
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
                        info!("【仙家坊市】求取法宝: {}, 消耗灵石: {}, 剩余灵石: {}",
                              relic.name, price, player.gold);
                    }
                } else {
                    info!("【仙家坊市】灵石不足: 需要 {}, 拥有 {}", price, player.gold);
                }
            }
        }
    }

    // 处理遗忘功法服务购买
    for (interaction, _) in remove_buttons.iter() {
        if matches!(interaction, Interaction::Pressed) {
            let price = ShopItem::ForgetTechnique.get_price();

            if let Ok(mut player) = player_query.get_single_mut() {
                if player.gold >= price {
                    player.gold -= price;
                    info!("【仙家坊市】了断尘缘, 消耗灵石: {}, 剩余灵石: {}",
                          price, player.gold);
                    // TODO: 进入移除卡牌选择UI
                } else {
                    info!("【仙家坊市】灵石不足: 需要 {}, 拥有 {}", price, player.gold);
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
