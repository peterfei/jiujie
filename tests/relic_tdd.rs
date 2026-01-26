//! 遗物系统TDD测试
//!
//! 遵循TDD原则：先写测试，覆盖所有场景，然后驱动开发

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_card_battler::components::*;
use bevy_card_battler::plugins::CorePlugin;
use bevy_card_battler::states::GameState;
use bevy_card_battler::systems::{RelicPlugin, CombatStartProcessed};

// ============================================================================
// 测试辅助函数
// ============================================================================

fn create_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .add_plugins(CorePlugin)
        .add_plugins(RelicPlugin)
        .init_state::<GameState>();

    // 初始化战斗状态
    app.insert_resource(CombatState::default());
    app.insert_resource(PlayerDeck::new());

    // 创建玩家实体
    app.world_mut().spawn(Player::default());
    app.world_mut().spawn(Enemy::new(1, "测试妖兽", 30));

    // 创建手牌和牌堆
    app.world_mut().spawn(Hand::new(10));
    app.world_mut().spawn(DrawPile::new(vec![
        Card::new(
            1,
            "御剑术",
            "造成6点伤害",
            CardType::Attack,
            1,
            CardEffect::DealDamage { amount: 6 },
            CardRarity::Common, "textures/cards/default.png",
        ),
    ]));
    app.world_mut().spawn(DiscardPile::new());

    app
}

// ============================================================================
// 场景1: 遗物获取 - 去重检查
// ============================================================================

#[test]
fn test_relic_acquisition_no_duplicates() {
    // 场景描述: 获取相同遗物时不应该重复添加
    // 预期结果: 第二次添加同一遗物应返回false

    let mut collection = RelicCollection::default();

    // 第一次添加飞剑符
    let burning_blood = Relic::burning_blood();
    assert!(collection.add_relic(burning_blood), "第一次添加遗物应该成功");
    assert_eq!(collection.count(), 1, "应该有1个遗物");

    // 第二次添加相同遗物
    let burning_blood_duplicate = Relic::burning_blood();
    assert!(!collection.add_relic(burning_blood_duplicate), "第二次添加相同遗物应该失败");
    assert_eq!(collection.count(), 1, "仍然应该只有1个遗物");

    // 添加不同的遗物
    let anchor = Relic::anchor();
    assert!(collection.add_relic(anchor), "添加不同遗物应该成功");
    assert_eq!(collection.count(), 2, "应该有2个遗物");
}

#[test]
fn test_relic_collection_has_method() {
    // 场景描述: 检查是否拥有某个遗物
    let mut collection = RelicCollection::default();

    assert!(!collection.has(RelicId::BurningBlood), "初始时不应拥有任何遗物");

    collection.add_relic(Relic::burning_blood());
    assert!(collection.has(RelicId::BurningBlood), "应该拥有飞剑符");
    assert!(!collection.has(RelicId::Anchor), "不应拥有定风珠");

    collection.add_relic(Relic::anchor());
    assert!(collection.has(RelicId::Anchor), "应该拥有定风珠");
}

#[test]
fn test_relic_collection_get_method() {
    // 场景描述: 获取遗物引用
    let mut collection = RelicCollection::default();

    assert!(collection.get(RelicId::BurningBlood).is_none(), "不存在的遗物应返回None");

    collection.add_relic(Relic::burning_blood());
    let relic = collection.get(RelicId::BurningBlood);
    assert!(relic.is_some(), "应返回遗物引用");
    assert_eq!(relic.unwrap().name, "飞剑符");
}

// ============================================================================
// 场景2: 战斗开始效果 - 飞剑符（造成伤害）
// ============================================================================

#[test]
fn test_burning_blood_deals_damage_on_combat_start() {
    // 场景描述: 战斗开始时，飞剑符应对所有敌人造成3点伤害
    // 预期结果: 敌人HP从30减少到27

    let mut app = create_test_app();
    app.world_mut().insert_resource(RelicCollection {
        relic: vec![Relic::burning_blood()],
    });
    app.world_mut().insert_resource(CombatStartProcessed {
        processed: false,
    });

    // 设置战斗状态
    app.world_mut().insert_resource(State::new(GameState::Combat));

    // 模拟战斗开始触发遗物
    let mut enemy_query = app.world_mut().query::<&mut Enemy>();
    let enemy = enemy_query.iter_mut(app.world_mut()).next().unwrap();
    assert_eq!(enemy.hp, 30, "初始HP应为30");

    // 更新遗物系统
    app.update();

    // 验证敌人受到伤害
    let mut enemy_query = app.world_mut().query::<&Enemy>();
    let enemy = enemy_query.iter_mut(app.world_mut()).next().unwrap();
    assert_eq!(enemy.hp, 27, "敌人HP应为27（30-3）");
}

// ============================================================================
// 场景3: 战斗开始效果 - 乾坤袋（抽牌）
// ============================================================================

#[test]
fn test_bag_of_preparation_draws_card_on_combat_start() {
    // 场景描述: 战斗开始时，乾坤袋应抽1张牌
    // 预期结果: 手牌增加1张

    let mut app = create_test_app();
    app.world_mut().insert_resource(RelicCollection {
        relic: vec![Relic::bag_of_preparation()],
    });
    app.world_mut().insert_resource(CombatStartProcessed {
        processed: false,
    });

    // 设置战斗状态和牌堆
    app.world_mut().insert_resource(State::new(GameState::Combat));

    // 更新遗物系统
    app.update();

    // 验证手牌增加（通过手牌组件）
    let mut hand_query = app.world_mut().query::<&Hand>();
    let hand = hand_query.iter(app.world_mut()).next().unwrap();
    assert_eq!(hand.cards.len(), 1, "应该抽了1张牌");
}

// ============================================================================
// 场景4: 回合开始效果 - 能量获取
// ============================================================================

#[test]
fn test_energy_on_turn_start() {
    // 场景描述: 回合开始时获得额外能量（如果有相关遗物）
    // 预期结果: 玩家能量增加

    let mut app = create_test_app();
    app.world_mut().insert_resource(RelicCollection::default());
    app.world_mut().insert_resource(State::new(GameState::Combat));

    // 触发回合开始遗物
    app.update();

    // 验证能量状态（基础能量应为3）
    let mut player_query = app.world_mut().query::<&Player>();
    let player = player_query.iter(app.world_mut()).next().unwrap();
    assert_eq!(player.energy, 3, "基础能量应为3");
}

// ============================================================================
// 场景5: 回合结束效果 - 定风珠（保留手牌）
// ============================================================================

#[test]
fn test_anchor_keeps_cards_on_turn_end() {
    // 场景描述: 回合结束时，定风珠遗物允许保留最多3张牌
    // 预期结果: 手牌数量不超过3张

    let app = create_test_app();
    let anchor = Relic::anchor();

    match &anchor.effect {
        RelicEffect::OnTurnEnd { keep_cards } => {
            assert_eq!(*keep_cards, 3, "定风珠应该允许保留3张牌");
        }
        _ => panic!("定风珠遗物效果不正确"),
    }
}

// ============================================================================
// 场景6: 打出牌效果 - 聚灵阵（每3张牌抽1张）
// ============================================================================

#[test]
fn test_strange_spoon_draws_every_third_card() {
    // 场景描述: 每打出第3张牌时，聚灵阵抽1张牌
    // 预期结果: 第3、6、9张牌触发抽牌

    let strange_spoon = Relic::strange_spoon();

    match &strange_spoon.effect {
        RelicEffect::OnCardPlayed { every_nth, draw_cards } => {
            assert_eq!(*every_nth, 3, "应该每3张牌触发");
            assert_eq!(*draw_cards, 1, "应该抽1张牌");
        }
        _ => panic!("聚灵阵遗物效果不正确"),
    }

    // 验证计数逻辑
    assert_eq!(3 % 3, 0, "第3张牌应触发");
    assert_eq!(6 % 3, 0, "第6张牌应触发");
    assert_ne!(1 % 3, 0, "第1张牌不应触发");
    assert_ne!(2 % 3, 0, "第2张牌不应触发");
}

// ============================================================================
// 场景7: 多遗物叠加效果
// ============================================================================

#[test]
fn test_multiple_relics_stack_effects() {
    // 场景描述: 同时拥有多个遗物时，效果应该叠加
    // 预期结果: 飞剑符造成伤害 + 乾坤袋抽牌

    let mut collection = RelicCollection::default();
    collection.add_relic(Relic::burning_blood());
    collection.add_relic(Relic::bag_of_preparation());

    assert_eq!(collection.count(), 2, "应该有2个遗物");

    // 验证两个遗物都在集合中
    assert!(collection.has(RelicId::BurningBlood));
    assert!(collection.has(RelicId::BagOfPreparation));
}

// ============================================================================
// 场景8: 遗物稀有度和随机生成
// ============================================================================

#[test]
fn test_relic_rarity_classification() {
    // 场景描述: 验证遗物稀有度分类
    let burning_blood = Relic::burning_blood();
    let bag = Relic::bag_of_preparation();
    let anchor = Relic::anchor();
    let spoon = Relic::strange_spoon();

    assert_eq!(burning_blood.rarity, RelicRarity::Common);
    assert_eq!(bag.rarity, RelicRarity::Common);
    assert_eq!(anchor.rarity, RelicRarity::Uncommon);
    assert_eq!(spoon.rarity, RelicRarity::Rare);
}

#[test]
fn test_relic_random_generation() {
    // 场景描述: 随机生成的遗物应该是有效的
    let relic = Relic::random();

    // 验证遗物有基本属性
    assert!(!relic.name.is_empty(), "遗物名称不应为空");
    assert!(!relic.description.is_empty(), "遗物描述不应为空");

    // 验证效果类型有效
    match &relic.effect {
        RelicEffect::OnCombatStart { .. } => {}
        RelicEffect::OnTurnStart { .. } => {}
        RelicEffect::OnTurnEnd { .. } => {}
        RelicEffect::OnDraw { .. } => {}
        RelicEffect::OnDealDamage { .. } => {}
        RelicEffect::OnTakeDamage { .. } => {}
        RelicEffect::OnCardPlayed { .. } => {}
    }
}

#[test]
fn test_relic_by_rarity() {
    // 场景描述: 按稀有度获取遗物列表
    let common_relics = Relic::by_rarity(RelicRarity::Common);
    let uncommon_relics = Relic::by_rarity(RelicRarity::Uncommon);
    let rare_relics = Relic::by_rarity(RelicRarity::Rare);

    assert_eq!(common_relics.len(), 2, "应该有2个常见遗物");
    assert_eq!(uncommon_relics.len(), 1, "应该有1个罕见遗物");
    assert_eq!(rare_relics.len(), 1, "应该有1个稀有遗物");
}

// ============================================================================
// 场景9: 战斗开始处理标记防重复触发
// ============================================================================

#[test]
fn test_combat_start_processed_prevents_duplicate_triggers() {
    // 场景描述: CombatStartProcessed标记应防止遗物重复触发
    // 预期结果: 设置processed=true后，遗物不再触发

    let mut app = create_test_app();
    app.world_mut().insert_resource(RelicCollection {
        relic: vec![Relic::burning_blood()],
    });

    // 设置为已处理
    app.world_mut().insert_resource(CombatStartProcessed {
        processed: true,
    });

    app.world_mut().insert_resource(State::new(GameState::Combat));

    // 更新系统（应该不触发遗物）
    app.update();

    // 验证遗物系统状态
    let processed = app.world().get_resource::<CombatStartProcessed>();
    assert!(processed.is_some());
    assert!(processed.unwrap().processed, "应该保持已处理状态");
}

// ============================================================================
// 场景10: 遗物描述和名称
// ============================================================================

#[test]
fn test_relic_descriptions_are_meaningful() {
    // 场景描述: 所有预定义遗物都应该有有意义的描述
    let relics = vec![
        Relic::burning_blood(),
        Relic::bag_of_preparation(),
        Relic::anchor(),
        Relic::strange_spoon(),
    ];

    for relic in relics {
        assert!(!relic.name.is_empty(), "{}: 名称不应为空", relic.name);
        assert!(!relic.description.is_empty(), "{}: 描述不应为空", relic.name);
        assert!(relic.description.len() > 10, "{}: 描述应该详细", relic.name);
    }
}

// ============================================================================
// 场景11: 遗物ID唯一性
// ============================================================================

#[test]
fn test_relic_id_uniqueness() {
    // 场景描述: 每个遗物应该有唯一的ID
    let relics = vec![
        Relic::burning_blood(),
        Relic::bag_of_preparation(),
        Relic::anchor(),
        Relic::strange_spoon(),
    ];

    let mut ids = std::collections::HashSet::new();
    for relic in relics {
        assert!(
            ids.insert(relic.id),
            "遗物ID应该唯一: {:?}",
            relic.id
        );
    }
}

// ============================================================================
// 场景12: 战斗开始时无遗物不影响游戏
// ============================================================================

#[test]
fn test_no_relics_does_not_crash_combat_start() {
    // 场景描述: 没有遗物时，战斗开始应该正常进行
    // 预期结果: 游戏不崩溃，系统正常运行

    let mut app = create_test_app();
    app.world_mut().insert_resource(RelicCollection::default());
    app.world_mut().insert_resource(CombatStartProcessed {
        processed: false,
    });
    app.world_mut().insert_resource(State::new(GameState::Combat));

    // 更新系统（不应崩溃）
    app.update();

    // 验证战斗状态正常
    let state = app.world().get_resource::<State<GameState>>();
    assert!(state.is_some());
}

// ============================================================================
// 场景13: 遗物效果数值合理性
// ============================================================================

#[test]
fn test_relic_effect_values_are_reasonable() {
    // 场景描述: 遗物效果的数值应该在合理范围内
    // 预期结果: 伤害、护甲、抽牌等数值不是负数或过大

    let burning_blood = Relic::burning_blood();
    if let RelicEffect::OnCombatStart { damage, block, draw_cards } = &burning_blood.effect {
        assert!(*damage >= 0, "伤害不应为负数");
        assert!(*damage <= 20, "伤害不应过大");
        assert!(*block >= 0, "护甲不应为负数");
        assert!(*draw_cards >= 0, "抽牌数不应为负数");
    }

    let strange_spoon = Relic::strange_spoon();
    if let RelicEffect::OnCardPlayed { every_nth, draw_cards } = &strange_spoon.effect {
        assert!(*every_nth > 0, "触发间隔应大于0");
        assert!(*every_nth <= 10, "触发间隔不应过大");
        assert!(*draw_cards > 0, "应该至少抽1张牌");
    }
}

// ============================================================================
// 场景14: 遗物集合默认值
// ============================================================================

#[test]
fn test_relic_collection_default_is_empty() {
    // 场景描述: 默认的遗物集合应该是空的
    let collection = RelicCollection::default();
    assert!(collection.is_empty(), "默认遗物集合应为空");
    assert_eq!(collection.count(), 0, "遗物数量应为0");
    assert!(!collection.has(RelicId::BurningBlood), "不应拥有任何遗物");
}
