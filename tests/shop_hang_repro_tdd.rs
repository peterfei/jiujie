use bevy::prelude::*;
use bevy_card_battler::components::*;
use bevy_card_battler::components::shop::*;
use bevy_card_battler::states::GameState;
use bevy_card_battler::systems::shop::*;

#[test]
fn test_shop_button_component_mismatch_repro() {
    let mut app = App::new();
    app.init_resource::<CurrentShopItems>();
    app.init_resource::<SelectedCardForRemoval>();
    app.init_resource::<RelicCollection>();
    app.init_resource::<PlayerDeck>();
    app.insert_resource(State::new(GameState::Shop));
    app.insert_resource(NextState::<GameState>::default());

    // 1. 模拟一个包含遗物的商店
    let mut current_items = CurrentShopItems::default();
    current_items.items = vec![
        ShopItem::Relic(Relic {
            id: RelicId::Custom(999),
            name: "测试遗物".into(),
            description: "描述".into(),
            rarity: RelicRarity::Common,
            effects: vec![],
        })
    ];
    app.insert_resource(current_items);

    // 2. 模拟玩家拥有 100 灵石
    app.world_mut().spawn((Player { gold: 100, ..Default::default() }, Cultivation::new()));

    // 3. 模拟错误的按钮组件挂载（当前代码的行为）
    // 点击索引 0 的按钮，但它是 ShopCardButton 而不是 ShopRelicButton
    app.world_mut().spawn((
        Interaction::Pressed,
        ShopCardButton { item_index: 0 },
    ));

    // 4. 运行一次交互系统
    app.add_systems(Update, handle_shop_interactions);
    app.update();

    // 验证逻辑：如果组件不匹配，灵石不应该被扣除（因为 match item 落入了 _ 模式）
    let player = app.world().query::<&Player>().iter(app.world()).next().unwrap();
    assert_eq!(player.gold, 100, "组件不匹配导致逻辑失效，但不应扣费");
}

#[test]
fn test_shop_exit_logic_robustness() {
    let mut app = App::new();
    app.init_resource::<CurrentShopItems>();
    app.init_resource::<MapProgress>();
    app.insert_resource(State::new(GameState::Shop));
    app.init_resource::<NextState<GameState>>();

    // 模拟快速连续点击：退出
    app.world_mut().spawn((Interaction::Pressed, ShopExitButton));
    
    app.add_systems(Update, handle_shop_interactions);
    app.update();
    
    let next_state = app.world().resource::<NextState<GameState>>();
    assert!(matches!(next_state.0, Some(GameState::Map)));
}