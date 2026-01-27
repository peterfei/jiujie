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
            .init_resource::<SelectedCardForRemoval>();

        app.add_systems(OnEnter(GameState::Shop), setup_shop_ui)
            .add_systems(Update, (
                handle_shop_interactions,
                update_gold_display.after(handle_shop_interactions), 
            ).run_if(in_state(GameState::Shop)))
            .add_systems(OnExit(GameState::Shop), cleanup_shop_ui);

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
            BackgroundColor(Color::srgb(0.02, 0.05, 0.02)), 
            ShopUiRoot,
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
                    items_parent.spawn((
                        Node { width: Val::Px(190.0), height: Val::Px(260.0), flex_direction: FlexDirection::Column, justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, padding: UiRect::all(Val::Px(15.0)), border: UiRect::all(Val::Px(1.0)), ..default() },
                        BackgroundColor(Color::srgba(0.05, 0.1, 0.05, 0.9)),
                        BorderColor(Color::srgb(0.3, 0.4, 0.3)),
                        BorderRadius::all(Val::Px(12.0)),
                    ))
                    .with_children(|item_parent| {
                        item_parent.spawn((Text::new(item.get_name()), TextFont { font_size: 22.0, font: chinese_font.clone(), ..default() }, TextColor(Color::WHITE)));
                        if let ShopItem::Card(card) = item {
                            item_parent.spawn((ImageNode::new(asset_server.load(card.image_path.clone())), Node { width: Val::Px(120.0), height: Val::Px(140.0), ..default() }));
                        }
                        item_parent.spawn((Text::new(item.get_description()), TextFont { font_size: 13.0, font: chinese_font.clone(), ..default() }, TextColor(Color::srgb(0.6, 0.6, 0.6)), Node { max_width: Val::Px(160.0), ..default() }));
                        item_parent.spawn((Text::new(format!("{} 灵石", item.get_price())), TextFont { font_size: 18.0, font: chinese_font.clone(), ..default() }, TextColor(COLOR_GOLD)));
                        
                        let action_text = match item {
                            ShopItem::Card(_) => "参悟",
                            ShopItem::Relic(_) => "求取",
                            ShopItem::Elixir { .. } => "服下",
                            ShopItem::ForgetTechnique => "了断",
                        };

                        item_parent.spawn((Button, ShopCardButton { item_index: index }, Node { width: Val::Px(90.0), height: Val::Px(35.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(Color::srgb(0.2, 0.4, 0.2)), BorderRadius::all(Val::Px(6.0)),))
                        .with_children(|btn| { btn.spawn((Text::new(action_text), TextFont { font_size: 16.0, font: chinese_font.clone(), ..default() }, TextColor(Color::WHITE))); });
                    });
                }
            });

            parent.spawn((Button, Node { width: Val::Px(180.0), height: Val::Px(45.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, margin: UiRect::top(Val::Px(20.0)), ..default() }, BackgroundColor(Color::srgb(0.25, 0.35, 0.25)), BorderRadius::all(Val::Px(8.0)), ShopExitButton,))
            .with_children(|p| { p.spawn((Text::new("离开坊市"), TextFont { font_size: 22.0, font: chinese_font.clone(), ..default() }, TextColor(Color::WHITE))); });
        });
}

/// 生成商店商品
fn generate_shop_items(_player_deck: &PlayerDeck, _relic_collection: &RelicCollection) -> Vec<ShopItem> {
    let all_cards = crate::components::cards::CardPool::all_cards();
    let mut items = vec![];
    use rand::seq::SliceRandom;
    let mut rng = rand::thread_rng();
    for card in all_cards.choose_multiple(&mut rng, 3) { items.push(ShopItem::Card(card.clone())); }
    let elixirs = vec![
        ShopItem::Elixir { name: "洗髓丹".to_string(), hp_restore: 20, price: 40, description: "洗筋伐髓，恢复 20 点道行".to_string() },
        ShopItem::Elixir { name: "九转还魂丹".to_string(), hp_restore: 50, price: 90, description: "生死肉骨，恢复 50 点道行".to_string() },
    ];
    if let Some(elixir) = elixirs.choose(&mut rng) { items.push(elixir.clone()); }
    items.push(ShopItem::ForgetTechnique);
    items
}

/// 处理商店交互
pub fn handle_shop_interactions(
    mut next_state: ResMut<NextState<GameState>>,
    mut player_query: Query<(&mut Player, &crate::components::Cultivation)>,
    current_items: Res<CurrentShopItems>,
    mut deck: ResMut<PlayerDeck>,
    mut relic_collection: ResMut<RelicCollection>,
    mut map_progress: ResMut<MapProgress>,
    card_buttons: Query<(&Interaction, &ShopCardButton), Changed<Interaction>>,
    relic_buttons: Query<(&Interaction, &ShopRelicButton), Changed<Interaction>>,
    remove_buttons: Query<(&Interaction, &ShopRemoveCardButton), Changed<Interaction>>,
    exit_buttons: Query<(&Interaction, &ShopExitButton), Changed<Interaction>>,
) {
    let mut should_save = false;

    // 处理离开
    for (interaction, _) in exit_buttons.iter() {
        if matches!(interaction, Interaction::Pressed) {
            map_progress.complete_current_node();
            next_state.set(GameState::Map);
            return;
        }
    }

    // 处理卡牌/灵丹
    for (interaction, shop_btn) in card_buttons.iter() {
        if matches!(interaction, Interaction::Pressed) {
            let item = &current_items.items[shop_btn.item_index];
            let price = item.get_price();
            if let Ok((mut player, _)) = player_query.get_single_mut() {
                if player.gold >= price {
                    match item {
                        ShopItem::Card(card) => {
                            player.gold -= price;
                            deck.add_card(card.clone());
                            info!("【仙家坊市】换取功法: {}", card.name);
                            should_save = true;
                        }
                        ShopItem::Elixir { name, hp_restore, .. } => {
                            player.gold -= price;
                            player.hp = (player.hp + hp_restore).min(player.max_hp);
                            info!("【仙家坊市】服下 {}", name);
                            should_save = true;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // 处理遗物
    for (interaction, shop_btn) in relic_buttons.iter() {
        if matches!(interaction, Interaction::Pressed) {
            let item = &current_items.items[shop_btn.item_index];
            if let Ok((mut player, _)) = player_query.get_single_mut() {
                if player.gold >= item.get_price() {
                    if let ShopItem::Relic(relic) = item {
                        player.gold -= item.get_price();
                        relic_collection.add_relic(relic.clone());
                        should_save = true;
                    }
                }
            }
        }
    }

    // 执行存档
    if should_save {
        if let Ok((player, cultivation)) = player_query.get_single() {
            let save = crate::resources::save::GameStateSave {
                player: player.clone(),
                cultivation: cultivation.clone(),
                deck: deck.cards.clone(),
                relics: relic_collection.relic.clone(),
                map_nodes: map_progress.nodes.clone(),
                current_map_node_id: map_progress.current_node_id,
                current_map_layer: map_progress.current_layer,
            };
            let _ = save.save_to_disk();
            info!("【存档系统】进度已同步");
        }
    }
}

/// 更新金币显示
pub fn update_gold_display(
    player_query: Query<&Player>,
    mut gold_text_query: Query<&mut Text, With<ShopGoldText>>,
) {
    if let Ok(player) = player_query.get_single() {
        for mut text in gold_text_query.iter_mut() {
            text.0 = format!("金石: {}", player.gold);
        }
    }
}

/// 清理商店UI
pub fn cleanup_shop_ui(mut commands: Commands, ui_query: Query<Entity, With<ShopUiRoot>>) {
    for entity in ui_query.iter() { commands.entity(entity).despawn_recursive(); }
}