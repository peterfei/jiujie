//! 商店系统

use bevy::prelude::*;
use bevy::text::TextFont;
use crate::components::*;
use crate::components::shop::*;
use crate::components::relic::RelicCollection;
use crate::components::map::MapProgress;
use crate::states::GameState;

const COLOR_GOLD: Color = Color::srgb(1.0, 0.84, 0.0);

/// 商店插件
pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentShopItems>()
            .init_resource::<SelectedCardForRemoval>()
            .add_event::<ShopItemPurchased>();

        app.add_systems(OnEnter(GameState::Shop), setup_shop_ui)
            .add_systems(Update, (
                handle_shop_interactions,
                update_gold_display.after(handle_shop_interactions), 
            ).run_if(in_state(GameState::Shop)))
            .add_systems(OnExit(GameState::Shop), cleanup_shop_ui);

        // --- 移除卡牌服务系统 ---
        app.add_systems(OnEnter(GameState::CardRemoval), setup_card_removal_ui)
            .add_systems(Update, (
                handle_card_removal_interaction,
                update_card_removal_scroll,
            ).run_if(in_state(GameState::CardRemoval)))
            .add_systems(OnExit(GameState::CardRemoval), cleanup_card_removal_ui);

        info!("【商店插件】ShopPlugin 已注册");
    }
}

/// 设置商店UI
pub fn setup_shop_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut current_items: ResMut<CurrentShopItems>,
    player_deck: Res<PlayerDeck>,
    relic_collection: Res<RelicCollection>,
    player_query: Query<&Player>,
) {
    info!("【仙家坊市】设置坊市UI");

    let current_gold = if let Ok(player) = player_query.get_single() {
        player.gold
    } else {
        100
    };

    if current_items.items.is_empty() {
        current_items.items = generate_shop_items(&player_deck, &relic_collection);
    }
    
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
            BackgroundColor(Color::srgb(0.02, 0.05, 0.02)), 
            ShopUiRoot,
            ZIndex(300), // 确保在最高层级 (高于地图的 100 和战斗的 200)
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("仙 家 坊 市"),
                TextFont { font_size: 52.0, font: chinese_font.clone(), ..default() },
                TextColor(Color::srgb(0.6, 0.9, 0.6)),
            ));

            parent.spawn((Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: Val::Px(10.0), ..default() },)).with_children(|p| {
                p.spawn((Text::new("所持灵石:"), TextFont { font_size: 24.0, font: chinese_font.clone(), ..default() }, TextColor(Color::srgb(0.7, 0.7, 0.7))));
                p.spawn((Text::new(format!("{}", current_gold)), TextFont { font_size: 28.0, font: chinese_font.clone(), ..default() }, TextColor(COLOR_GOLD), ShopGoldText));
            });

            parent.spawn((Node { width: Val::Percent(90.0), height: Val::Percent(60.0), flex_direction: FlexDirection::Row, flex_wrap: FlexWrap::Wrap, justify_content: JustifyContent::Center, align_items: AlignItems::Center, column_gap: Val::Px(25.0), row_gap: Val::Px(25.0), ..default() },))
            .with_children(|items_parent| {
                for (index, item) in current_items.items.iter().enumerate() {
                    let is_sold_out = matches!(item, ShopItem::SoldOut { .. });
                    
                    let border_color = match item {
                        ShopItem::Relic(relic) => relic.rarity.color(),
                        ShopItem::SoldOut { .. } => Color::srgb(0.2, 0.2, 0.2),
                        _ => Color::srgb(0.3, 0.4, 0.3),
                    };

                    let bg_color = if is_sold_out {
                        Color::srgba(0.02, 0.02, 0.02, 0.95)
                    } else {
                        Color::srgba(0.05, 0.1, 0.05, 0.9)
                    };

                    items_parent.spawn((
                        Node { width: Val::Px(190.0), height: Val::Px(260.0), flex_direction: FlexDirection::Column, justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, padding: UiRect::all(Val::Px(15.0)), border: UiRect::all(Val::Px(1.0)), ..default() },
                        BackgroundColor(bg_color),
                        BorderColor(border_color),
                        BorderRadius::all(Val::Px(12.0)),
                    ))
                    .with_children(|item_parent| {
                        let name_color = if is_sold_out { Color::srgb(0.4, 0.4, 0.4) } else { Color::WHITE };
                        item_parent.spawn((Text::new(item.get_name()), TextFont { font_size: 22.0, font: chinese_font.clone(), ..default() }, TextColor(name_color)));
                        
                        if let ShopItem::Card(card) = item {
                            item_parent.spawn((ImageNode::new(asset_server.load(card.image_path.clone())), Node { width: Val::Px(120.0), height: Val::Px(140.0), ..default() }));
                        } else if let ShopItem::Relic(_) = item {
                            // 法宝占位图 (后续可根据 ID 换)
                            item_parent.spawn((ImageNode::new(asset_server.load("textures/relics/default.png")), Node { width: Val::Px(80.0), height: Val::Px(80.0), ..default() }));
                        }

                        let desc_color = if is_sold_out { Color::srgb(0.3, 0.3, 0.3) } else { Color::srgb(0.6, 0.6, 0.6) };
                        item_parent.spawn((Text::new(item.get_description()), TextFont { font_size: 13.0, font: chinese_font.clone(), ..default() }, TextColor(desc_color), Node { max_width: Val::Px(160.0), ..default() }));
                        
                        if !is_sold_out {
                            item_parent.spawn((Text::new(format!("{} 灵石", item.get_price())), TextFont { font_size: 18.0, font: chinese_font.clone(), ..default() }, TextColor(COLOR_GOLD)));
                        }
                        
                        let action_text = match item {
                            ShopItem::Card(_) => "参悟",
                            ShopItem::Relic(_) => "求取",
                            ShopItem::Elixir { .. } => "服下",
                            ShopItem::ForgetTechnique => "了断",
                            ShopItem::SoldOut { .. } => "已换取",
                        };

                        let btn_bg = if is_sold_out { Color::srgb(0.1, 0.1, 0.1) } else { Color::srgb(0.2, 0.4, 0.2) };
                        let mut btn_cmd = item_parent.spawn((
                            Button,
                            Node { width: Val::Px(90.0), height: Val::Px(35.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                            BackgroundColor(btn_bg),
                            BorderRadius::all(Val::Px(6.0)),
                        ));

                        if !is_sold_out {
                            // 根据类型挂载正确的标记组件
                            match item {
                                ShopItem::Card(_) | ShopItem::Elixir { .. } => {
                                    btn_cmd.insert(ShopCardButton { item_index: index });
                                }
                                ShopItem::Relic(_) => {
                                    btn_cmd.insert(ShopRelicButton { item_index: index });
                                }
                                ShopItem::ForgetTechnique => {
                                    btn_cmd.insert(ShopRemoveCardButton);
                                }
                                _ => {}
                            }
                        }

                        btn_cmd.with_children(|btn| {
                            btn.spawn((Text::new(action_text), TextFont { font_size: 16.0, font: chinese_font.clone(), ..default() }, TextColor(if is_sold_out { Color::srgb(0.4, 0.4, 0.4) } else { Color::WHITE })));
                        });
                    });
                }
            });

            parent.spawn((Button, Node { width: Val::Px(180.0), height: Val::Px(45.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, margin: UiRect::top(Val::Px(20.0)), ..default() }, BackgroundColor(Color::srgb(0.25, 0.35, 0.25)), BorderRadius::all(Val::Px(8.0)), ShopExitButton,))
            .with_children(|p| { p.spawn((Text::new("离开坊市"), TextFont { font_size: 22.0, font: chinese_font.clone(), ..default() }, TextColor(Color::WHITE))); });
        });
}

/// 生成商店商品
fn generate_shop_items(_player_deck: &PlayerDeck, relic_collection: &RelicCollection) -> Vec<ShopItem> {
    let all_cards = crate::components::cards::CardPool::all_cards();
    let mut items = vec![];
    use rand::seq::SliceRandom;
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // 1. 功法生成 (3张)
    // --- 测试专用：强制加入万剑归宗 ---
    if let Some(wan_jian) = all_cards.iter().find(|c| c.name == "万剑归宗") {
        items.push(ShopItem::Card(wan_jian.clone()));
    }

    let remaining_cards: Vec<_> = all_cards.iter()
        .filter(|c| c.name != "万剑归宗")
        .cloned()
        .collect();
    for card in remaining_cards.choose_multiple(&mut rng, 2) { 
        items.push(ShopItem::Card(card.clone())); 
    }

    // 2. 法宝生成 (1-2个)
    let num_relics = rng.gen_range(1..=2);
    let mut generated_relic_ids = std::collections::HashSet::new();
    
    for _ in 0..num_relics {
        // 概率：70% 常见, 25% 罕见, 5% 稀有
        let roll = rng.gen::<f32>();
        let rarity = if roll < 0.7 {
            crate::components::relic::RelicRarity::Common
        } else if roll < 0.95 {
            crate::components::relic::RelicRarity::Uncommon
        } else {
            crate::components::relic::RelicRarity::Rare
        };

        let mut available_relics = crate::components::relic::Relic::by_rarity(rarity);
        // 过滤掉玩家已有的和本次已经生成的
        available_relics.retain(|r| !relic_collection.has(r.id) && !generated_relic_ids.contains(&r.id));

        if let Some(relic) = available_relics.choose(&mut rng) {
            generated_relic_ids.insert(relic.id);
            items.push(ShopItem::Relic(relic.clone()));
        } else if rarity != crate::components::relic::RelicRarity::Common {
            // 如果高稀有度没货了，尝试降级生成
            let mut common_relics = crate::components::relic::Relic::by_rarity(crate::components::relic::RelicRarity::Common);
            common_relics.retain(|r| !relic_collection.has(r.id) && !generated_relic_ids.contains(&r.id));
            if let Some(common) = common_relics.choose(&mut rng) {
                generated_relic_ids.insert(common.id);
                items.push(ShopItem::Relic(common.clone()));
            }
        }
    }

    // 3. 灵丹生成 (1个)
    let elixirs = vec![
        ShopItem::Elixir { name: "洗髓丹".to_string(), hp_restore: 20, price: 40, description: "洗筋伐髓，恢复 20 点道行".to_string() },
        ShopItem::Elixir { name: "九转还魂丹".to_string(), hp_restore: 50, price: 90, description: "生死肉骨，恢复 50 点道行".to_string() },
    ];
    if let Some(elixir) = elixirs.choose(&mut rng) { items.push(elixir.clone()); }

    // 4. 服务项目
    items.push(ShopItem::ForgetTechnique);
    items
}

/// 处理商店交互
pub fn handle_shop_interactions(
    mut next_state: ResMut<NextState<GameState>>,
    mut player_query: Query<(&mut Player, &crate::components::Cultivation)>,
    mut current_items: ResMut<CurrentShopItems>,
    mut deck: ResMut<PlayerDeck>,
    mut relic_collection: ResMut<RelicCollection>,
    mut map_progress: ResMut<MapProgress>,
    card_buttons: Query<(&Interaction, &ShopCardButton), Changed<Interaction>>,
    relic_buttons: Query<(&Interaction, &ShopRelicButton), Changed<Interaction>>,
    remove_buttons: Query<&Interaction, (With<ShopRemoveCardButton>, Changed<Interaction>)>,
    exit_buttons: Query<(&Interaction, &ShopExitButton), Changed<Interaction>>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    // 1. 处理离开 (优先级最高)
    for (interaction, _) in exit_buttons.iter() {
        if matches!(interaction, Interaction::Pressed) {
            sfx_events.send(PlaySfxEvent::new(SfxType::UiClick));
            info!("【仙家坊市】告辞离开");
            map_progress.complete_current_node();
            next_state.set(GameState::Map);
            return; 
        }
    }

    // 2. 处理卡牌/灵丹
    let mut purchased_index = None;
    for (interaction, shop_btn) in card_buttons.iter() {
        if matches!(interaction, Interaction::Pressed) {
            let index = shop_btn.item_index;
            if index >= current_items.items.len() { continue; }

            let item = &current_items.items[index];
            let price = item.get_price();
            if let Ok((mut player, _)) = player_query.get_single_mut() {
                if player.gold >= price {
                    match item {
                        ShopItem::Card(card) => {
                            player.gold -= price;
                            deck.add_card(card.clone());
                            sfx_events.send(PlaySfxEvent::new(SfxType::GoldGain));
                            info!("【仙家坊市】换取功法: {}", card.name);
                            purchased_index = Some(index);
                        }
                        ShopItem::Elixir { name, hp_restore, .. } => {
                            player.gold -= price;
                            player.hp = (player.hp + hp_restore).min(player.max_hp);
                            sfx_events.send(PlaySfxEvent::new(SfxType::Heal));
                            info!("【仙家坊市】服下 {}", name);
                            purchased_index = Some(index);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // 3. 处理遗物
    if purchased_index.is_none() {
        for (interaction, shop_btn) in relic_buttons.iter() {
            if matches!(interaction, Interaction::Pressed) {
                let index = shop_btn.item_index;
                if index >= current_items.items.len() { continue; }
                
                let item = &current_items.items[index];
                if let Ok((mut player, cultivation)) = player_query.get_single_mut() {
                    let price = item.get_price();
                    if player.gold >= price {
                        if let ShopItem::Relic(relic) = item {
                            if relic_collection.add_relic(relic.clone(), cultivation) {
                                player.gold -= price;
                                sfx_events.send(PlaySfxEvent::new(SfxType::GoldGain));
                                info!("【仙家坊市】购得法宝: {}", relic.name);
                                purchased_index = Some(index);
                            } else {
                                warn!("【仙家坊市】法宝位已满或已拥有该法宝，无法求取");
                            }
                        }
                    }
                }
            }
        }
    }

    // 4. 处理移除卡牌 (了断因果)
    if purchased_index.is_none() {
        for interaction in remove_buttons.iter() {
            if matches!(interaction, Interaction::Pressed) {
                if let Ok((player, _)) = player_query.get_single() {
                    if player.gold >= 50 {
                        sfx_events.send(PlaySfxEvent::new(SfxType::UiClick));
                        info!("【仙家坊市】准备遗忘功法，进入识海...");
                        next_state.set(GameState::CardRemoval);
                    } else {
                        warn!("【仙家坊市】灵石不足，无法了断因果 (需要 50)");
                    }
                }
            }
        }
    }

    // 如果发生了购买，标记为售罄并刷新 UI
    if let Some(index) = purchased_index {
        let original_name = current_items.items[index].get_name().to_string();
        current_items.items[index] = ShopItem::SoldOut { original_name };
        // 强制刷新 UI：重新进入商店状态
        next_state.set(GameState::Shop);
    }
}

/// 更新金币显示
pub fn update_gold_display(
    player_query: Query<&Player>,
    mut gold_text_query: Query<&mut Text, With<ShopGoldText>>,
) {
    if let Ok(player) = player_query.get_single() {
        for mut text in gold_text_query.iter_mut() {
            text.0 = format!("灵石: {}", player.gold);
        }
    }
}

/// 清理商店UI
pub fn cleanup_shop_ui(
    mut commands: Commands, 
    ui_query: Query<Entity, With<ShopUiRoot>>,
    mut current_items: ResMut<CurrentShopItems>,
) {
    for entity in ui_query.iter() { commands.entity(entity).despawn_recursive(); }
    current_items.items.clear();
}

// ============================================================================
// 移除卡牌服务系统 (Card Removal Service)
// ============================================================================

#[derive(Component)]
pub struct CardRemovalUiRoot;

#[derive(Component)]
pub struct CardRemovalScrollGrid;

#[derive(Component)]
pub struct CardRemovalItem {
    pub card_index: usize,
}

#[derive(Component)]
pub struct CardRemovalCancelButton;

/// 设置移除卡牌UI
pub fn setup_card_removal_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_deck: Res<PlayerDeck>,
) {
    info!("【识海了断】展示功法列表");
    let chinese_font: Handle<Font> = asset_server.load("fonts/Arial Unicode.ttf");

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(40.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.01, 0.02, 0.01, 0.95)),
            CardRemovalUiRoot,
            ZIndex(400),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("了 断 因 果"),
                TextFont { font_size: 48.0, font: chinese_font.clone(), ..default() },
                TextColor(Color::srgb(0.8, 0.4, 0.4)),
            ));

            parent.spawn((
                Text::new("请选择一门要永久遗忘的功法 (花费 50 灵石)"),
                TextFont { font_size: 20.0, font: chinese_font.clone(), ..default() },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node { margin: UiRect::bottom(Val::Px(30.0)), ..default() },
            ));

            // 卡牌网格 (支持滚动)
            parent.spawn((
                Node {
                    width: Val::Percent(95.0),
                    height: Val::Percent(75.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(15.0),
                    row_gap: Val::Px(15.0),
                    overflow: Overflow::scroll_y(), // 启用纵向滚动
                    ..default()
                },
                ScrollPosition::default(),
                CardRemovalScrollGrid, // 新增标记组件
            )).with_children(|grid| {
                for (index, card) in player_deck.cards.iter().enumerate() {
                    grid.spawn((
                        Button,
                        Node {
                            width: Val::Px(140.0),
                            height: Val::Px(200.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::SpaceBetween,
                            padding: UiRect::all(Val::Px(10.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.1, 0.05, 0.05, 0.8)),
                        BorderColor(Color::srgb(0.4, 0.2, 0.2)),
                        CardRemovalItem { card_index: index },
                    )).with_children(|btn| {
                        btn.spawn((
                            Text::new(card.name.clone()),
                            TextFont { font_size: 18.0, font: chinese_font.clone(), ..default() },
                            TextColor(Color::WHITE),
                        ));
                        btn.spawn((
                            ImageNode::new(asset_server.load(card.image_path.clone())),
                            Node { width: Val::Px(100.0), height: Val::Px(120.0), ..default() },
                        ));
                        btn.spawn((
                            Text::new(card.rarity.get_chinese_name()),
                            TextFont { font_size: 12.0, font: chinese_font.clone(), ..default() },
                            TextColor(Color::srgb(0.6, 0.6, 0.6)),
                        ));
                    });
                }
            });

            // 取消按钮
            parent.spawn((
                Button,
                Node {
                    width: Val::Px(150.0),
                    height: Val::Px(45.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(30.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                CardRemovalCancelButton,
            )).with_children(|btn| {
                btn.spawn((
                    Text::new("暂且保留"),
                    TextFont { font_size: 20.0, font: chinese_font.clone(), ..default() },
                    TextColor(Color::WHITE),
                ));
            });
        });
}

/// 处理移除卡牌交互
pub fn handle_card_removal_interaction(
    mut next_state: ResMut<NextState<GameState>>,
    mut player_query: Query<&mut Player>,
    mut player_deck: ResMut<PlayerDeck>,
    item_query: Query<(&Interaction, &CardRemovalItem), Changed<Interaction>>,
    cancel_query: Query<&Interaction, (With<CardRemovalCancelButton>, Changed<Interaction>)>,
    mut current_shop: ResMut<CurrentShopItems>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    // 1. 处理取消
    for interaction in cancel_query.iter() {
        if matches!(interaction, Interaction::Pressed) {
            sfx_events.send(PlaySfxEvent::new(SfxType::UiClick));
            next_state.set(GameState::Shop);
            return;
        }
    }

    // 2. 处理移除选中卡牌
    for (interaction, item) in item_query.iter() {
        if matches!(interaction, Interaction::Pressed) {
            if let Ok(mut player) = player_query.get_single_mut() {
                if player.gold >= 50 {
                    if item.card_index < player_deck.cards.len() {
                        let removed_card = player_deck.cards.remove(item.card_index);
                        player.gold -= 50;
                        sfx_events.send(PlaySfxEvent::new(SfxType::UiClick));
                        info!("【识海了断】永久遗忘了功法：{}", removed_card.name);

                        // 移除商店中的“移除卡牌”商品 (坊市规则：此类服务一次性)
                        current_shop.items.retain(|i| !matches!(i, ShopItem::ForgetTechnique));

                        next_state.set(GameState::Shop);
                        return;
                    }
                }
            }
        }
    }
}

/// 清理移除卡牌UI
pub fn cleanup_card_removal_ui(
    mut commands: Commands,
    query: Query<Entity, With<CardRemovalUiRoot>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

/// 更新移除卡牌界面的滚动位置
pub fn update_card_removal_scroll(
    mut mouse_wheel_events: EventReader<bevy::input::mouse::MouseWheel>,
    mut query: Query<&mut ScrollPosition, With<CardRemovalScrollGrid>>,
) {
    use bevy::input::mouse::MouseScrollUnit;
    for event in mouse_wheel_events.read() {
        for mut scroll in query.iter_mut() {
            let dy = match event.unit {
                MouseScrollUnit::Line => event.y * 30.0, // 每一行移动 30 像素
                MouseScrollUnit::Pixel => event.y,
            };
            
            // 更新滚动位置
            scroll.offset_y -= dy;
            
            // 限制滚动范围，防止滚出虚空
            if scroll.offset_y < 0.0 {
                scroll.offset_y = 0.0;
            }
        }
    }
}