use bevy::prelude::*;
use bevy_card_battler::components::*;
use bevy_card_battler::components::shop::*;
use bevy_card_battler::states::GameState;
use bevy_card_battler::systems::shop::*;
use bevy_card_battler::components::audio::PlaySfxEvent;
use bevy_card_battler::components::relic::RelicObtainedEvent;
use bevy_card_battler::components::combat::StatusEffectEvent;

#[test]
fn test_shop_button_component_mismatch_repro() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.add_event::<PlaySfxEvent>();
    app.add_event::<RelicObtainedEvent>();
    app.add_event::<StatusEffectEvent>();
    
    app.init_resource::<CurrentShopItems>();
    app.init_resource::<SelectedCardForRemoval>();
    app.init_resource::<RelicCollection>();
    app.init_resource::<PlayerDeck>();
    app.init_resource::<bevy_card_battler::components::map::MapProgress>();
    
    app.init_state::<GameState>();
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Shop);
    app.update(); // 进入 Shop 状态

    // 1. 模拟遗物
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

    // 2. 模拟玩家
    app.world_mut().spawn(
        Player { 
            gold: 100, 
            hp: 80, 
            max_hp: 80, 
            ..default()
        }
    );


    // 3. 模拟点击
    app.world_mut().spawn((
        Button,
        Interaction::Pressed,
        ShopCardButton { item_index: 0 },
    ));

    // 4. 运行系统
    app.add_systems(Update, handle_shop_interactions);
    app.update();

    // 验证不崩溃即可
    let player = app.world_mut().query::<&Player>().iter(app.world()).next().unwrap();
    assert_eq!(player.gold, 100);
}

#[test]
fn test_shop_exit_logic_robustness() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.add_event::<PlaySfxEvent>();
    app.add_event::<RelicObtainedEvent>();
    app.add_event::<StatusEffectEvent>();
    
    app.init_resource::<CurrentShopItems>();
    app.init_resource::<SelectedCardForRemoval>();
    app.init_resource::<RelicCollection>();
    app.init_resource::<PlayerDeck>();
    app.init_resource::<bevy_card_battler::components::map::MapProgress>();
    
    app.init_state::<GameState>();
    // 强制设为 Shop 状态
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Shop);
    app.update(); 

    // 创建按钮
    let btn = app.world_mut().spawn((
        Button,
        Interaction::None,
        ShopExitButton
    )).id();
    
    // 注册系统
    app.add_systems(Update, handle_shop_interactions);
    app.update(); // 初始帧
    
    // 点击按钮
    app.world_mut().entity_mut(btn).insert(Interaction::Pressed);
    
    // 运行多帧
    for _ in 0..10 {
        app.update();
    }
    
    let current_state = app.world().resource::<State<GameState>>().get();
    assert_eq!(*current_state, GameState::Map, "点击退出按钮后应转换到 Map 状态");
}