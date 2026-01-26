//! 商店UI更新集成测试
//!
//! 测试金币变化时UI是否正确更新

use bevy::prelude::*;
use bevy::app::App;
use bevy::state::app::StatesPlugin;
use bevy_card_battler::components::*;
use bevy_card_battler::components::shop::*;
use bevy_card_battler::plugins::CorePlugin;
use bevy_card_battler::states::GameState;
use bevy_card_battler::systems::{ShopPlugin, update_gold_display};

#[test]
fn test_shop_gold_text_component_exists() {
    // 测试：验证金币文本有 ShopGoldText 标记

    let mut app = create_shop_app();

    // 检查：应该有 ShopGoldText 组件
    let mut gold_text_query = app.world_mut().query::<&ShopGoldText>();
    let count = gold_text_query.iter(app.world_mut()).count();
    println!("ShopGoldText 组件数量: {}", count);
    assert!(count > 0, "应该有金币文本标记组件");
}

#[test]
fn test_shop_gold_display_updates_on_gold_change() {
    // 测试：金币变化时UI应该更新

    let mut app = create_shop_app();

    // 获取金币文本实体
    let mut text_query = app.world_mut()
        .query::<(Entity, &Text, &ShopGoldText)>();
    let (text_entity, text_before, _) = text_query.iter(app.world_mut())
        .next()
        .expect("应该有金币文本");

    println!("购买前文本: {}", text_before.0);
    assert!(text_before.0.contains("100"), "初始应该显示100金币");

    // 获取玩家并扣除金币
    {
        let mut player_query = app.world_mut().query::<&mut Player>();
        let mut player = player_query.iter_mut(app.world_mut()).next().unwrap();
        player.gold -= 30; // 模拟购买
    }

    // 运行更新系统（应该触发 Text 更新）
    app.update();

    // 检查文本是否更新
    let mut text_query = app.world_mut()
        .query::<&Text>();
    let text_after = text_query.iter(app.world_mut())
        .filter(|t| t.0.contains("金币"))
        .next()
        .expect("应该有金币文本");

    println!("购买后文本: {}", text_after.0);
    assert!(text_after.0.contains("70"), "应该更新为70金币");
    assert!(!text_after.0.contains("100"), "不应该再显示100金币");

    println!("✅ UI更新测试通过: 文本从 '金币: 100' 更新到 '{}'", text_after.0);
}

#[test]
fn test_shop_update_gold_display_system_runs() {
    // 测试：直接调用 update_gold_display 系统函数

    let mut app = create_shop_app();

    // 修改玩家金币
    {
        let mut player_query = app.world_mut().query::<&mut Player>();
        let mut player = player_query.iter_mut(app.world_mut()).next().unwrap();
        player.gold = 50;
        println!("设置玩家金币为: {}", player.gold);
    }

    // 获取更新前的文本
    let text_before = {
        let mut text_query = app.world_mut().query::<&Text>();
        text_query.iter(app.world_mut())
            .filter(|t| t.0.contains("金币"))
            .next()
            .unwrap()
            .0
            .clone()
    };
    println!("更新前: {}", text_before);

    // 手动调用更新系统的 WorldQuery
    // 注意：我们需要使用 bevy 的系统调度器来正确运行系统
    app.update();

    // 检查更新后的文本
    let text_after = {
        let mut text_query = app.world_mut().query::<&Text>();
        text_query.iter(app.world_mut())
            .filter(|t| t.0.contains("金币"))
            .next()
            .unwrap()
            .0
            .clone()
    };
    println!("更新后: {}", text_after);

    assert!(text_after.contains("50"), "应该显示更新后的50金币");
    println!("✅ 系统运行测试通过");
}

// ============================================================================
// 测试辅助函数
// ============================================================================

fn create_shop_app() -> App {
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
                CardRarity::Common, "textures/cards/default.png"
            )),
        ],
    });

    // 设置状态为 Shop
    app.world_mut().insert_resource(State::new(GameState::Shop));

    // 创建玩家实体（带100金币）
    let mut player = Player::default();
    player.gold = 100;
    app.world_mut().spawn(player);

    // 手动创建商店UI（简化版，无需 AssetServer）
    create_simple_shop_ui(&mut app);

    // 运行一次以注册系统
    app.update();

    app
}

fn create_simple_shop_ui(app: &mut App) {
    // 创建简化的商店UI结构
    app.world_mut().spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        ShopUiRoot,
    ))
    .with_children(|parent| {
        // 金币文本（带标记）
        parent.spawn((
            Text::new("金币: 100"),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            ShopGoldText, // 关键：这个标记用于后续更新
        ));
    });
}
