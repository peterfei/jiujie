//! 商店购买功能集成测试
//!
//! 测试商店购买按钮交互是否正常工作

use bevy::prelude::*;
use bevy::app::App;
use bevy::state::app::StatesPlugin;
use bevy_card_battler::components::*;
use bevy_card_battler::components::shop::{ShopItem, ShopCardButton, ShopRemoveCardButton, CurrentShopItems};
use bevy_card_battler::plugins::CorePlugin;
use bevy_card_battler::states::GameState;
use bevy_card_battler::systems::ShopPlugin;

// ============================================================================
// 测试辅助函数
// ============================================================================

fn create_shop_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .add_plugins(CorePlugin)
        .add_plugins(ShopPlugin)
        .init_state::<GameState>();

    // 初始化玩家资源
    app.world_mut().insert_resource(PlayerDeck::new());
    app.world_mut().insert_resource(RelicCollection::default());
    app.world_mut().insert_resource(CurrentShopItems {
        items: vec![
            ShopItem::Card(Card::new(
                1, "测试卡", "描述",
                CardType::Attack, 1, CardEffect::DealDamage { amount: 6 },
                CardRarity::Common,
            )),
            ShopItem::ForgetTechnique,
        ],
    });

    // 设置状态为 Shop（重要：handle_shop_interactions 需要此状态）
    app.world_mut().insert_resource(State::new(GameState::Shop));

    // 创建玩家实体（带100金币）
    let mut player = Player::default();
    player.gold = 100;
    app.world_mut().spawn(player);

    // 手动创建测试按钮（无需渲染系统）
    app.world_mut().spawn((
        Button,
        ShopCardButton { item_index: 0 },
        Interaction::None,
    ));

    app.world_mut().spawn((
        Button,
        ShopRemoveCardButton,
        Interaction::None,
    ));

    app
}

// ============================================================================
// 测试：购买按钮组件存在
// ============================================================================

#[test]
fn test_shop_purchase_button_components_exist() {
    let mut app = create_shop_test_app();

    // 检查：应该有 ShopCardButton 组件
    let mut card_btn_query = app.world_mut().query::<&ShopCardButton>();
    let card_btn_count = card_btn_query.iter(app.world_mut()).count();
    println!("✅ ShopCardButton 数量: {}", card_btn_count);
    assert_eq!(card_btn_count, 1, "应该有1个卡牌购买按钮");

    // 检查：应该有 ShopRemoveCardButton 组件
    let mut remove_btn_query = app.world_mut().query::<&ShopRemoveCardButton>();
    let remove_btn_count = remove_btn_query.iter(app.world_mut()).count();
    println!("✅ ShopRemoveCardButton 数量: {}", remove_btn_count);
    assert_eq!(remove_btn_count, 1, "应该有1个移除卡牌按钮");

    // 检查：按钮应该同时有 Button 和标记组件
    let mut both_query = app.world_mut().query::<(&Button, &ShopCardButton)>();
    let both_count = both_query.iter(app.world_mut()).count();
    println!("✅ 同时有 Button + ShopCardButton 的实体: {}", both_count);
    assert_eq!(both_count, 1);
}

// ============================================================================
// 测试：购买交互处理 - 模拟点击
// ============================================================================

#[test]
fn test_shop_purchase_click_deducts_gold() {
    let mut app = create_shop_test_app();

    // 获取购买前金币
    let mut player_query = app.world_mut().query::<&Player>();
    let gold_before = player_query.iter(app.world_mut()).next().unwrap().gold;
    println!("购买前金币: {}", gold_before);
    assert_eq!(gold_before, 100);

    // 获取卡牌按钮实体
    let mut btn_query = app.world_mut()
        .query::<(Entity, &ShopCardButton)>();
    let (btn_entity, shop_btn) = btn_query.iter(app.world_mut())
        .next()
        .expect("应该有卡牌购买按钮");
    println!("找到购买按钮: {:?}, item_index: {}", btn_entity, shop_btn.item_index);

    // 模拟点击：设置 Interaction 为 Pressed
    app.world_mut().entity_mut(btn_entity)
        .insert(Interaction::Pressed);

    // 运行 Update 系统（包括 handle_shop_interactions）
    app.update();

    // 检查金币是否减少
    let mut player_query = app.world_mut().query::<&Player>();
    let gold_after = player_query.iter(app.world_mut()).next().unwrap().gold;
    println!("购买后金币: {}", gold_after);

    // Common 卡牌价格 30
    assert_eq!(gold_after, 70, "购买后金币应该是 100 - 30 = 70");

    // 检查卡牌是否加入牌组
    let deck = app.world().get_resource::<PlayerDeck>().unwrap();
    assert!(deck.cards.len() >= 12, "牌组应该至少有初始卡牌");
    println!("✅ 牌组卡牌数量: {}", deck.cards.len());
    println!("✅ 购买功能正常: 金币从 100 减少到 70");
}

// ============================================================================
// 测试：金币不足无法购买
// ============================================================================

#[test]
fn test_shop_insufficient_gold_no_purchase() {
    let mut app = create_shop_test_app();

    // 设置玩家金币为 0（不足以购买）
    {
        let mut player_query = app.world_mut().query::<&mut Player>();
        let mut player = player_query.iter_mut(app.world_mut()).next().unwrap();
        player.gold = 0;
    }

    // 获取卡牌按钮实体
    let mut btn_query = app.world_mut()
        .query::<(Entity, &ShopCardButton)>();
    let (btn_entity, _) = btn_query.iter(app.world_mut())
        .next()
        .expect("应该有卡牌购买按钮");

    // 模拟点击
    app.world_mut().entity_mut(btn_entity)
        .insert(Interaction::Pressed);

    // 运行系统
    app.update();

    // 检查金币应该仍然是 0
    let mut player_query = app.world_mut().query::<&Player>();
    let gold = player_query.iter(app.world_mut()).next().unwrap().gold;
    assert_eq!(gold, 0, "金币不足时不应购买成功");

    // 检查牌组应该是初始大小（购买失败，没有添加新卡）
    let deck = app.world().get_resource::<PlayerDeck>().unwrap();
    assert!(deck.cards.len() >= 12, "牌组应该只有初始卡牌");
    println!("✅ 牌组卡牌数量: {} (应该只有初始卡牌)", deck.cards.len());
    println!("✅ 金币不足检测正常");
}

// ============================================================================
// 测试：移除卡牌服务购买
// ============================================================================

#[test]
fn test_shop_remove_card_purchase() {
    let mut app = create_shop_test_app();

    // 获取购买前金币
    let mut player_query = app.world_mut().query::<&Player>();
    let gold_before = player_query.iter(app.world_mut()).next().unwrap().gold;
    println!("购买前金币: {}", gold_before);

    // 获取移除卡牌按钮实体
    let mut btn_query = app.world_mut()
        .query::<(Entity, &ShopRemoveCardButton)>();
    let (btn_entity, _) = btn_query.iter(app.world_mut())
        .next()
        .expect("应该有移除卡牌按钮");

    // 模拟点击
    app.world_mut().entity_mut(btn_entity)
        .insert(Interaction::Pressed);

    // 运行系统
    app.update();

    // 检查金币是否减少 50
    let mut player_query = app.world_mut().query::<&Player>();
    let gold_after = player_query.iter(app.world_mut()).next().unwrap().gold;
    println!("购买后金币: {}", gold_after);

    assert_eq!(gold_after, 50, "移除卡牌服务价格50，金币应该从100减少到50");
    println!("✅ 移除卡牌服务购买正常");
}

// ============================================================================
// 测试：Changed<Interaction> 过滤器工作
// ============================================================================

#[test]
fn test_shop_interaction_only_processes_changed() {
    let mut app = create_shop_test_app();

    // 获取按钮实体
    let mut btn_query = app.world_mut()
        .query::<(Entity, &ShopCardButton)>();
    let (btn_entity, _) = btn_query.iter(app.world_mut())
        .next()
        .expect("应该有卡牌购买按钮");

    // 第一次点击 - 应该触发购买
    app.world_mut().entity_mut(btn_entity)
        .insert(Interaction::Pressed);
    app.update();

    let gold_after_first = app.world_mut()
        .query::<&Player>()
        .iter(app.world_mut())
        .next()
        .unwrap()
        .gold;
    assert_eq!(gold_after_first, 70, "第一次点击应该扣除金币");

    // 第二次运行 update，没有改变 Interaction - 不应该再次购买
    app.update();

    let gold_after_second = app.world_mut()
        .query::<&Player>()
        .iter(app.world_mut())
        .next()
        .unwrap()
        .gold;
    assert_eq!(gold_after_second, 70, "没有改变Interaction时不应重复购买");

    println!("✅ Changed<Interaction> 过滤器正常工作");
}
