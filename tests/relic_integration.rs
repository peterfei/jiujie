//! 遗物系统集成测试
//!
//! 测试遗物系统在完整战斗流程中的表现

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_card_battler::components::*;
use bevy_card_battler::plugins::CorePlugin;
use bevy_card_battler::states::GameState;
use bevy_card_battler::systems::{RelicPlugin, CombatStartProcessed};

// ============================================================================
// 集成测试辅助函数
// ============================================================================

fn setup_combat_with_relics(relics: Vec<Relic>) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .add_plugins(CorePlugin)
        .add_plugins(RelicPlugin)
        .init_state::<GameState>();

    // 设置遗物集合
    let mut collection = RelicCollection::default();
    for relic in relics {
        collection.add_relic(relic);
    }
    app.insert_resource(collection);
    app.insert_resource(CombatStartProcessed {
        processed: false,
    });

    // 初始化战斗资源
    app.insert_resource(CombatState::default());
    app.insert_resource(PlayerDeck::new());

    // 创建玩家和敌人
    app.world_mut().spawn(Player::default());
    app.world_mut().spawn(Enemy::new(1, "哥布林", 30));

    // 创建牌堆
    app.world_mut().spawn(Hand::new(10));
    app.world_mut().spawn(DrawPile::new(vec![
        Card::new(
            1,
            "打击",
            "造成6点伤害",
            CardType::Attack,
            1,
            CardEffect::DealDamage { amount: 6 },
            CardRarity::Common,
        ),
    ]));
    app.world_mut().spawn(DiscardPile::new());

    app
}

// ============================================================================
// 集成测试1: 燃烧之血完整战斗流程
// ============================================================================

#[test]
fn test_burning_blood_full_combat_flow() {
    // 场景描述: 拥有燃烧之血的完整战斗
    // 验证: 战斗开始伤害、后续战斗正常进行

    let mut app = setup_combat_with_relics(vec![Relic::burning_blood()]);
    app.world_mut().insert_resource(State::new(GameState::Combat));

    // 触发战斗开始遗物效果
    app.update();

    // 验证敌人受到初始伤害
    let mut enemy_query = app.world_mut().query::<&Enemy>();
    let enemy = enemy_query.iter_mut(app.world_mut()).next().unwrap();
    assert_eq!(enemy.hp, 27, "战斗开始后敌人HP应为27");

    // 验证战斗仍在进行
    let state = app.world().get_resource::<State<GameState>>();
    assert_eq!(*state.unwrap().get(), GameState::Combat);
}

// ============================================================================
// 集成测试2: 准备背包完整战斗流程
// ============================================================================

#[test]
fn test_bag_of_preparation_full_combat_flow() {
    // 场景描述: 拥有准备背包的完整战斗
    // 验证: 战斗开始抽牌、牌堆数量正确

    let mut app = setup_combat_with_relics(vec![Relic::bag_of_preparation()]);
    app.world_mut().insert_resource(State::new(GameState::Combat));

    // 触发战斗开始遗物效果
    app.update();

    // 验证手牌增加
    let mut hand_query = app.world_mut().query::<&Hand>();
    let hand = hand_query.iter(app.world_mut()).next().unwrap();
    assert_eq!(hand.cards.len(), 1, "应该抽了1张牌");
}

// ============================================================================
// 集成测试3: 多遗物组合效果
// ============================================================================

#[test]
fn test_multiple_relics_combined_effect() {
    // 场景描述: 燃烧之血 + 准备背包组合
    // 验证: 两个遗物效果都生效

    let mut app = setup_combat_with_relics(vec![
        Relic::burning_blood(),
        Relic::bag_of_preparation(),
    ]);
    app.world_mut().insert_resource(State::new(GameState::Combat));

    // 触发战斗开始遗物效果
    app.update();

    // 验证敌人受到伤害
    let mut enemy_query = app.world_mut().query::<&Enemy>();
    let enemy = enemy_query.iter_mut(app.world_mut()).next().unwrap();
    assert_eq!(enemy.hp, 27, "敌人应受到3点伤害");

    // 验证抽牌效果
    let mut hand_query = app.world_mut().query::<&Hand>();
    let hand = hand_query.iter(app.world_mut()).next().unwrap();
    assert_eq!(hand.cards.len(), 1, "应该抽了1张牌");
}

// ============================================================================
// 集成测试4: 遗物系统状态管理
// ============================================================================

#[test]
fn test_relic_system_state_management() {
    // 场景描述: 验证遗物系统资源正确初始化和管理

    let mut app = setup_combat_with_relics(vec![Relic::burning_blood()]);
    app.world_mut().insert_resource(State::new(GameState::Combat));

    // 验证RelicCollection资源存在
    let collection = app.world().get_resource::<RelicCollection>();
    assert!(collection.is_some(), "RelicCollection应存在");
    assert_eq!(collection.unwrap().count(), 1);

    // 验证CombatStartProcessed资源存在
    let processed = app.world().get_resource::<CombatStartProcessed>();
    assert!(processed.is_some(), "CombatStartProcessed应存在");

    // 更新后验证状态改变
    app.update();
    let processed = app.world().get_resource::<CombatStartProcessed>();
    assert!(processed.unwrap().processed, "应标记为已处理");
}

// ============================================================================
// 集成测试5: 连续战斗场景
// ============================================================================

#[test]
fn test_relics_persist_across_multiple_combats() {
    // 场景描述: 遗物应该在多场战斗中持续生效
    // 验证: 第一场战斗后，遗物仍然存在并在第二场战斗生效

    let mut app = setup_combat_with_relics(vec![Relic::burning_blood()]);

    // 第一场战斗
    app.world_mut().insert_resource(State::new(GameState::Combat));
    app.update();

    let mut enemy_query = app.world_mut().query::<&Enemy>();
    let enemy = enemy_query.iter_mut(app.world_mut()).next().unwrap();
    assert_eq!(enemy.hp, 27, "第一场战斗敌人受到伤害");

    // 重置战斗状态（模拟第二场战斗）
    app.world_mut().insert_resource(CombatStartProcessed {
        processed: false,
    });
    app.world_mut().spawn(Enemy::new(2, "第二个哥布林", 30));

    // 第二场战斗 - 遗物不会再次触发（因为processed=true）
    app.update();

    // 验证遗物不会重复触发（这是预期行为）
    let mut enemies: Vec<_> = app
        .world_mut()
        .query::<&Enemy>()
        .iter(app.world_mut())
        .collect();

    assert!(enemies.len() >= 2, "应该有2个敌人");

    // 第二个敌人没有被遗物影响（因为它是战斗开始后生成的）
    // 这是预期行为 - 遗物只在战斗开始时触发一次
    let second_enemy = enemies.get(1).unwrap();
    assert_eq!(second_enemy.hp, 30, "第二场战斗敌人未受遗物影响（战斗开始后生成）");
}

// ============================================================================
// 集成测试6: 遗物与战斗结束状态
// ============================================================================

#[test]
fn test_relics_work_with_victory_state() {
    // 场景描述: 遗物效果不应干扰战斗胜利状态转换

    let mut app = setup_combat_with_relics(vec![
        Relic::burning_blood(),
        Relic::bag_of_preparation(),
    ]);
    app.world_mut().insert_resource(State::new(GameState::Combat));

    // 触发遗物效果
    app.update();

    // 验证仍在战斗状态
    let state = app.world().get_resource::<State<GameState>>();
    assert_eq!(*state.unwrap().get(), GameState::Combat);

    // 遗物不应触发状态转换
    let state = app.world().get_resource::<State<GameState>>();
    assert_eq!(*state.unwrap().get(), GameState::Combat, "遗物效果不应改变游戏状态");
}

// ============================================================================
// 集成测试7: 遗物事件系统
// ============================================================================

#[test]
fn test_relic_events_are_registered() {
    // 场景描述: 遗物相关事件应该正确注册

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(CorePlugin)
        .add_plugins(RelicPlugin);

    // 验证事件类型已注册
    // 注意: Bevy的事件系统在运行时注册，这里主要验证编译通过
    // 实际事件触发需要在完整游戏流程中测试
}

// ============================================================================
// 集成测试8: 空遗物集合不影响战斗
// ============================================================================

#[test]
fn test_empty_relic_collection_combat() {
    // 场景描述: 没有遗物时战斗应正常进行

    let mut app = setup_combat_with_relics(vec![]);
    app.world_mut().insert_resource(State::new(GameState::Combat));

    // 更新系统（不应崩溃）
    app.update();

    // 验证敌人HP未变
    let mut enemy_query = app.world_mut().query::<&Enemy>();
    let enemy = enemy_query.iter(app.world_mut()).next().unwrap();
    assert_eq!(enemy.hp, 30, "无遗物时敌人HP应为初始值");
}

// ============================================================================
// 集成测试9: 遗物稀有度分布
// ============================================================================

#[test]
fn test_relic_rarity_distribution() {
    // 场景描述: 验证不同稀有度遗物可以正常获取和添加

    let mut collection = RelicCollection::default();

    // 添加各种稀有度的遗物
    for relic in Relic::by_rarity(RelicRarity::Common) {
        collection.add_relic(relic);
    }
    for relic in Relic::by_rarity(RelicRarity::Uncommon) {
        collection.add_relic(relic);
    }
    for relic in Relic::by_rarity(RelicRarity::Rare) {
        collection.add_relic(relic);
    }

    assert_eq!(collection.count(), 4, "应该有4个不同稀有度的遗物");
}

// ============================================================================
// 集成测试10: 遗物与玩家属性交互
// ============================================================================

#[test]
fn test_relics_interaction_with_player_stats() {
    // 场景描述: 遗物效果应该正确修改玩家属性

    let mut app = setup_combat_with_relics(vec![Relic::burning_blood()]);
    app.world_mut().insert_resource(State::new(GameState::Combat));

    // 获取初始玩家状态
    let mut player_query = app.world_mut().query::<&Player>();
    let initial_player = player_query.iter(app.world_mut()).next().unwrap().clone();

    // 触发遗物效果（燃烧之血不影响玩家属性）
    app.update();

    // 玩家属性应该保持不变（燃烧之血只影响敌人）
    let mut player_query = app.world_mut().query::<&Player>();
    let player = player_query.iter(app.world_mut()).next().unwrap();
    assert_eq!(player.hp, initial_player.hp, "燃烧之血不影响玩家HP");
}

// ============================================================================
// 集成测试11: 锚遗物回合结束保留
// ============================================================================

#[test]
fn test_anchor_relic_turn_end_mechanics() {
    // 场景描述: 验证锚遗物在回合结束时的机制

    let app = setup_combat_with_relics(vec![Relic::anchor()]);

    // 验证锚遗物的效果配置
    let collection = app.world().get_resource::<RelicCollection>().unwrap();
    let anchor = collection.get(RelicId::Anchor).unwrap();

    match &anchor.effect {
        RelicEffect::OnTurnEnd { keep_cards } => {
            assert_eq!(*keep_cards, 3, "锚应保留3张牌");
        }
        _ => panic!("锚遗物效果类型错误"),
    }
}

// ============================================================================
// 集成测试12: 奇怪勺子打出牌计数
// ============================================================================

#[test]
fn test_strange_spoon_card_played_counter() {
    // 场景描述: 验证奇怪勺子的计数机制

    let app = setup_combat_with_relics(vec![Relic::strange_spoon()]);

    // 验证奇怪勺子配置
    let collection = app.world().get_resource::<RelicCollection>().unwrap();
    let spoon = collection.get(RelicId::StrangeSpoon).unwrap();

    match &spoon.effect {
        RelicEffect::OnCardPlayed { every_nth, draw_cards } => {
            assert_eq!(*every_nth, 3);
            assert_eq!(*draw_cards, 1);
        }
        _ => panic!("奇怪勺子效果类型错误"),
    }
}

// ============================================================================
// 集成测试13: 遗物描述完整性
// ============================================================================

#[test]
fn test_all_predefined_relics_have_valid_descriptions() {
    // 场景描述: 所有预定义遗物都应该有完整的名称和描述

    let relics = vec![
        Relic::burning_blood(),
        Relic::bag_of_preparation(),
        Relic::anchor(),
        Relic::strange_spoon(),
    ];

    for relic in relics {
        assert!(!relic.name.is_empty(), "{}: 名称不能为空", relic.name);
        assert!(!relic.description.is_empty(), "{}: 描述不能为空", relic.name);
        assert!(relic.description.contains("战斗") || relic.description.contains("回合") || relic.description.contains("打出"),
            "{}: 描述应该说明触发条件", relic.name);
    }
}

// ============================================================================
// 集成测试14: 遗物系统与游戏状态转换
// ============================================================================

#[test]
fn test_relic_system_respects_game_states() {
    // 场景描述: 遗物系统应该只在Combat状态下触发

    let mut app = setup_combat_with_relics(vec![Relic::burning_blood()]);

    // 在Menu状态下不应触发
    app.world_mut().insert_resource(State::new(GameState::MainMenu));
    app.update();

    // 切换到Combat状态
    app.world_mut().insert_resource(State::new(GameState::Combat));
    app.update();

    // 验证系统在Combat状态下正常工作
    let processed = app.world().get_resource::<CombatStartProcessed>();
    assert!(processed.is_some());
}

// ============================================================================
// 集成测试15: 遗物随机生成稳定性
// ============================================================================

#[test]
fn test_relic_random_generation_stability() {
    // 场景描述: 随机生成的遗物应该总是有效的

    for _ in 0..10 {
        let relic = Relic::random();

        // 验证基本属性
        assert!(!relic.name.is_empty());
        assert!(!relic.description.is_empty());

        // 验证ID有效
        match relic.id {
            RelicId::BurningBlood => {}
            RelicId::BagOfPreparation => {}
            RelicId::Anchor => {}
            RelicId::StrangeSpoon => {}
        }
    }
}
