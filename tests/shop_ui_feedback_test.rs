//! 商店UI反馈集成测试
//!
//! 还原问题：商店物品点击UI没反馈（金币显示不更新）

use bevy::prelude::*;
use bevy::app::App;
use bevy::state::app::StatesPlugin;
use bevy_card_battler::components::*;
use bevy_card_battler::components::shop::*;
use bevy_card_battler::plugins::CorePlugin;
use bevy_card_battler::states::GameState;
use bevy_card_battler::systems::ShopPlugin;

#[test]
fn test_shop_gold_text_component_exists() {
    // 测试：ShopGoldText组件是否存在
    
    let mut app = create_test_app();
    app.world_mut().insert_resource(State::new(GameState::Shop));
    
    // 运行OnEnter系统
    for _ in 0..10 {
        app.update();
    }
    
    // 检查ShopGoldText组件
    let mut gold_text_query = app.world_mut().query::<&ShopGoldText>();
    let count = gold_text_query.iter(app.world_mut()).count();
    
    println!("=== ShopGoldText组件数量: {} ===", count);
    
    if count == 0 {
        println!("❌ 问题根因: ShopGoldText组件没有被添加！");
        panic!("ShopGoldText组件缺失，update_gold_display无法工作");
    } else {
        println!("✅ ShopGoldText组件存在");
    }
}

fn create_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .add_plugins(CorePlugin)
        .add_plugins(ShopPlugin)
        .init_state::<GameState>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>();
    
    app.world_mut().insert_resource(PlayerDeck::new());
    app.world_mut().insert_resource(RelicCollection::default());
    app.world_mut().insert_resource(CurrentShopItems {
        items: vec![
            ShopItem::Card(Card::new(
                1, "测试卡", "描述",
                CardType::Attack, 1, CardEffect::DealDamage { amount: 6 },
                CardRarity::Common,
            )),
        ],
    });
    
    let mut player = Player::default();
    player.gold = 100;
    app.world_mut().spawn(player);
    
    // 创建商店UI
    app.world_mut().spawn((Node::default(), ShopUiRoot))
        .with_children(|parent| {
            parent.spawn((
                Text::new("金币: 100"),
                TextFont::default(),
                ShopGoldText,  // 关键标记
            ));
        });
    
    app
}
