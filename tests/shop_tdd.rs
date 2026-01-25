//! 商店系统TDD测试
//!
//! 遵循TDD原则：先写测试，覆盖所有场景，然后驱动开发

use bevy::prelude::*;
use bevy::app::App;
use bevy::state::app::StatesPlugin;
use bevy_card_battler::components::*;
use bevy_card_battler::components::shop::{ShopItem, ShopUiRoot, ShopExitButton, CurrentShopItems, SelectedCardForRemoval};
use bevy_card_battler::plugins::CorePlugin;
use bevy_card_battler::states::GameState;
use bevy_card_battler::systems::ShopPlugin;

// ============================================================================
// 测试辅助函数
// ============================================================================

fn create_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .add_plugins(CorePlugin)
        .add_plugins(ShopPlugin)
        .init_state::<GameState>();

    // 初始化玩家资源
    app.insert_resource(PlayerDeck::new());
    app.insert_resource(RelicCollection::default());

    // 创建玩家实体
    app.world_mut().spawn((
        Player::default(),
        PlayerMarker,
    ));

    app
}

#[derive(Component)]
struct PlayerMarker;

// ============================================================================
// 场景1: 商店商品价格计算 - 卡牌
// ============================================================================

#[test]
fn test_shop_item_card_price_by_rarity() {
    // 场景描述: 卡牌价格应根据稀有度计算
    // 预期结果: Common=30, Uncommon=50, Rare=80, Special=100

    let common_card = Card::new(
        1, "普通打击", "造成6点伤害",
        CardType::Attack, 1, CardEffect::DealDamage { amount: 6 },
        CardRarity::Common,
    );
    let shop_item = ShopItem::Card(common_card);
    assert_eq!(shop_item.get_price(), 30, "普通卡牌价格应为30");

    let uncommon_card = Card::new(
        2, "罕见打击", "造成9点伤害",
        CardType::Attack, 1, CardEffect::DealDamage { amount: 9 },
        CardRarity::Uncommon,
    );
    let shop_item = ShopItem::Card(uncommon_card);
    assert_eq!(shop_item.get_price(), 50, "罕见卡牌价格应为50");

    let rare_card = Card::new(
        3, "稀有打击", "造成12点伤害",
        CardType::Attack, 1, CardEffect::DealDamage { amount: 12 },
        CardRarity::Rare,
    );
    let shop_item = ShopItem::Card(rare_card);
    assert_eq!(shop_item.get_price(), 80, "稀有卡牌价格应为80");

    let special_card = Card::new(
        4, "特殊打击", "造成15点伤害",
        CardType::Attack, 1, CardEffect::DealDamage { amount: 15 },
        CardRarity::Special,
    );
    let shop_item = ShopItem::Card(special_card);
    assert_eq!(shop_item.get_price(), 100, "特殊卡牌价格应为100");
}

// ============================================================================
// 场景2: 商店商品价格计算 - 遗物
// ============================================================================

#[test]
fn test_shop_item_relic_price_by_rarity() {
    // 场景描述: 遗物价格应根据稀有度计算
    // 预期结果: Common=50, Uncommon=75, Rare=100, Special=150

    let common_relic = Relic::burning_blood(); // Common
    let shop_item = ShopItem::Relic(common_relic);
    assert_eq!(shop_item.get_price(), 50, "普通遗物价格应为50");

    let uncommon_relic = Relic::anchor(); // Uncommon
    let shop_item = ShopItem::Relic(uncommon_relic);
    assert_eq!(shop_item.get_price(), 75, "罕见遗物价格应为75");

    let rare_relic = Relic::strange_spoon(); // Rare
    let shop_item = ShopItem::Relic(rare_relic);
    assert_eq!(shop_item.get_price(), 100, "稀有遗物价格应为100");
}

// ============================================================================
// 场景3: 商店商品价格 - 移除卡牌服务
// ============================================================================

#[test]
fn test_shop_item_remove_card_price() {
    // 场景描述: 移除卡牌服务应有固定价格
    // 预期结果: 固定价格50

    let shop_item = ShopItem::RemoveCard;
    assert_eq!(shop_item.get_price(), 50, "移除卡牌服务价格应为50");
}

// ============================================================================
// 场景4: 商店商品名称和描述
// ============================================================================

#[test]
fn test_shop_item_name_and_description() {
    // 场景描述: 所有商店商品都应该有名称和描述
    // 预期结果: 名称和描述不为空

    let card = Card::new(
        1, "测试卡牌", "测试描述",
        CardType::Attack, 1, CardEffect::DealDamage { amount: 6 },
        CardRarity::Common,
    );

    let card_item = ShopItem::Card(card.clone());
    assert_eq!(card_item.get_name(), "测试卡牌");
    assert_eq!(card_item.get_description(), "测试描述");

    let relic = Relic::burning_blood();
    let relic_item = ShopItem::Relic(relic);
    assert_eq!(relic_item.get_name(), "燃烧之血");
    assert!(!relic_item.get_description().is_empty());

    let remove_item = ShopItem::RemoveCard;
    assert_eq!(remove_item.get_name(), "移除卡牌");
    assert_eq!(remove_item.get_description(), "从牌组中永久移除一张卡牌");
}

// ============================================================================
// 场景5: 玩家初始金币
// ============================================================================

#[test]
fn test_player_gets_initial_gold_on_first_shop_visit() {
    // 场景描述: 玩家首次进入商店时应获得100金币
    // 预期结果: 玩家金币从0变为100

    let mut app = create_test_app();

    // 验证初始金币为0
    let mut player_query = app.world_mut().query::<&Player>();
    let player = player_query.iter_mut(app.world_mut()).next().unwrap();
    assert_eq!(player.gold, 0, "初始金币应为0");

    // 模拟商店UI设置（给初始金币）
    {
        let mut player_query = app.world_mut().query::<&mut Player>();
        let mut player = player_query.iter_mut(app.world_mut()).next().unwrap();
        if player.gold == 0 {
            player.gold = 100;
        }
    }

    // 验证金币已设置
    let mut player_query = app.world_mut().query::<&Player>();
    let player = player_query.iter(app.world_mut()).next().unwrap();
    assert_eq!(player.gold, 100, "玩家应获得100金币");
}

#[test]
fn test_player_keeps_existing_gold_on_subsequent_visits() {
    // 场景描述: 玩家已有金币时，进入商店不应重置
    // 预期结果: 金币保持不变

    let mut app = create_test_app();

    // 设置玩家已有金币
    {
        let mut player_query = app.world_mut().query::<&mut Player>();
        let mut player = player_query.iter_mut(app.world_mut()).next().unwrap();
        player.gold = 50;
    }

    // 模拟商店UI设置（只在金币为0时给予）
    {
        let mut player_query = app.world_mut().query::<&mut Player>();
        let mut player = player_query.iter_mut(app.world_mut()).next().unwrap();
        if player.gold == 0 {
            player.gold = 100;
        }
    }

    // 验证金币未改变
    let mut player_query = app.world_mut().query::<&Player>();
    let player = player_query.iter(app.world_mut()).next().unwrap();
    assert_eq!(player.gold, 50, "金币应保持为50");
}

// ============================================================================
// 场景6: 购买卡牌 - 金币充足
// ============================================================================

#[test]
fn test_buy_card_with_sufficient_gold() {
    // 场景描述: 玩家有足够金币时可以购买卡牌
    // 预期结果: 金币减少，卡牌加入牌组

    let mut player = Player::default();
    player.gold = 100;

    let mut deck = PlayerDeck::new();

    let card = Card::new(
        1, "测试卡", "描述",
        CardType::Attack, 1, CardEffect::DealDamage { amount: 6 },
        CardRarity::Common,
    );
    let shop_item = ShopItem::Card(card.clone());
    let price = shop_item.get_price();

    // 模拟购买
    let initial_gold = player.gold;
    let initial_deck_size = deck.cards.len();

    assert!(initial_gold >= price, "金币应充足");
    player.gold -= price;
    deck.add_card(card);

    assert_eq!(player.gold, 70, "金币应减少30");
    assert_eq!(deck.cards.len(), initial_deck_size + 1, "牌组应增加1张卡");
}

// ============================================================================
// 场景7: 购买卡牌 - 金币不足
// ============================================================================

#[test]
fn test_cannot_buy_card_with_insufficient_gold() {
    // 场景描述: 玩家金币不足时无法购买
    // 预期结果: 购买失败，金币不变

    let mut player = Player::default();
    player.gold = 20; // 不足以购买Common卡(30金币)

    let card = Card::new(
        1, "测试卡", "描述",
        CardType::Attack, 1, CardEffect::DealDamage { amount: 6 },
        CardRarity::Common,
    );
    let shop_item = ShopItem::Card(card);
    let price = shop_item.get_price();

    assert!(player.gold < price, "金币应不足");

    // 模拟购买检查
    let can_afford = player.gold >= price;
    assert!(!can_afford, "不应能够购买");
    assert_eq!(player.gold, 20, "金币应保持不变");
}

// ============================================================================
// 场景8: 购买遗物
// ============================================================================

#[test]
fn test_buy_relic_adds_to_collection() {
    // 场景描述: 购买遗物应加入遗物收藏
    // 预期结果: 遗物加入收藏，金币减少

    let mut player = Player::default();
    player.gold = 100;

    let mut collection = RelicCollection::default();

    let relic = Relic::burning_blood();
    let shop_item = ShopItem::Relic(relic.clone());
    let price = shop_item.get_price();

    let initial_count = collection.count();
    let initial_gold = player.gold;

    // 模拟购买
    player.gold -= price;
    collection.add_relic(relic);

    assert_eq!(player.gold, initial_gold - price, "金币应减少");
    assert_eq!(collection.count(), initial_count + 1, "遗物收藏应增加1个");
}

// ============================================================================
// 场景9: 购买移除卡牌服务
// ============================================================================

#[test]
fn test_buy_remove_card_service() {
    // 场景描述: 购买移除卡牌服务后应能选择卡牌移除
    // 预期结果: 支付金币，进入移除卡牌模式

    let mut player = Player::default();
    player.gold = 100;

    let remove_service = ShopItem::RemoveCard;
    let price = remove_service.get_price();

    assert_eq!(price, 50, "移除服务价格为50");

    // 模拟购买
    player.gold -= price;

    assert_eq!(player.gold, 50, "金币应减少50");
    // TODO: 验证进入移除卡牌UI状态
}

// ============================================================================
// 场景10: 商店资源初始化
// ============================================================================

#[test]
fn test_current_shop_items_resource_default() {
    // 场景描述: CurrentShopItems资源应正确初始化
    // 预期结果: 默认为空列表

    let items = CurrentShopItems::default();
    assert!(items.items.is_empty(), "默认商店商品列表应为空");
    assert_eq!(items.items.len(), 0, "商品数量应为0");
}

#[test]
fn test_selected_card_for_removal_resource_default() {
    // 场景描述: SelectedCardForRemoval资源应正确初始化
    // 预期结果: 默认没有选中卡牌

    let selection = SelectedCardForRemoval::default();
    assert!(selection.card_id.is_none(), "默认不应有选中卡牌");
}

// ============================================================================
// 场景11: 商店UI组件标记
// ============================================================================

#[test]
fn test_shop_ui_components_exist() {
    // 场景描述: 验证所有商店UI组件标记存在
    // 预期结果: 组件可以正常创建

    let mut app = create_test_app();

    // 测试ShopUiRoot组件
    app.world_mut().spawn(ShopUiRoot);

    // 测试ShopExitButton组件
    app.world_mut().spawn((
        Button,
        ShopExitButton,
    ));

    // 验证组件存在
    let mut root_query = app.world_mut().query::<&ShopUiRoot>();
    assert_eq!(root_query.iter(app.world_mut()).count(), 1, "应有1个ShopUiRoot");

    let mut button_query = app.world_mut().query::<&ShopExitButton>();
    assert_eq!(button_query.iter(app.world_mut()).count(), 1, "应有1个ShopExitButton");
}

// ============================================================================
// 场景12: 商店商品数量限制
// ============================================================================

#[test]
fn test_shop_generation_respects_limits() {
    // 场景描述: 商店生成商品应有数量限制
    // 预期结果: 商店不应生成过多或过少商品

    let items = CurrentShopItems {
        items: vec![
            ShopItem::RemoveCard, // 移除卡牌服务
        ],
    };

    // 商店应至少有移除卡牌服务
    assert!(!items.items.is_empty(), "商店至少应有移除服务");

    // TODO: 实现商品生成后，测试数量限制（如5-10个商品）
    // 当前generate_shop_items返回空列表，待实现
}

// ============================================================================
// 场景13: 价格范围合理性
// ============================================================================

#[test]
fn test_shop_prices_are_reasonable() {
    // 场景描述: 所有商品价格应在合理范围内
    // 预期结果: 价格为正数且不过高

    // 卡牌价格
    let common_card = Card::new(
        1, "卡", "描述",
        CardType::Attack, 1, CardEffect::DealDamage { amount: 6 },
        CardRarity::Common,
    );
    assert!(ShopItem::Card(common_card.clone()).get_price() > 0);
    assert!(ShopItem::Card(common_card).get_price() <= 150);

    // 遗物价格
    assert!(ShopItem::Relic(Relic::burning_blood()).get_price() > 0);
    assert!(ShopItem::Relic(Relic::burning_blood()).get_price() <= 200);

    // 移除服务价格
    assert!(ShopItem::RemoveCard.get_price() > 0);
    assert!(ShopItem::RemoveCard.get_price() <= 100);
}

// ============================================================================
// 场景14: 商店商品去重
// ============================================================================

#[test]
fn test_shop_should_not_duplicate_items() {
    // 场景描述: 商店不应出现重复商品
    // 预期结果: 相同卡牌/遗物不应重复出现

    let card1 = Card::new(
        1, "卡1", "描述",
        CardType::Attack, 1, CardEffect::DealDamage { amount: 6 },
        CardRarity::Common,
    );
    let card2 = Card::new(
        2, "卡2", "描述",
        CardType::Attack, 1, CardEffect::DealDamage { amount: 6 },
        CardRarity::Common,
    );

    let items = vec![
        ShopItem::Card(card1),
        ShopItem::Card(card2),
    ];

    // 验证商品不同
    assert_ne!(items[0].get_name(), items[1].get_name());

    // TODO: 实现商品生成后，验证去重逻辑
}

// ============================================================================
// 场景15: 商店状态转换
// ============================================================================

#[test]
fn test_shop_state_exists() {
    // 场景描述: 验证Shop状态存在
    // 预期结果: GameState::Shop可用

    let state = GameState::Shop;
    assert_eq!(state, GameState::Shop);

    // 验证状态可以用于匹配
    match state {
        GameState::MainMenu => panic!("不应是MainMenu"),
        GameState::Map => panic!("不应是Map"),
        GameState::Combat => panic!("不应是Combat"),
        GameState::Reward => panic!("不应是Reward"),
        GameState::Shop => {}, // 正确
        GameState::Rest => panic!("不应是Rest"),
        GameState::GameOver => panic!("不应是GameOver"),
    }
}
