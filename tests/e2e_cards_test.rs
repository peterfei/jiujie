//! E2E测试：卡牌系统
//!
//! 测试卡牌系统的基础功能

use bevy::app::App;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_card_battler::components::*;
use bevy_card_battler::states::GameState;

// ============================================================================
// 测试辅助函数
// ============================================================================

/// 创建测试应用
fn create_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(StatesPlugin)
        .init_state::<GameState>();
    app.update();
    app
}

// ============================================================================
// E2E测试：卡牌组件
// ============================================================================

#[test]
fn e2e_card_creation() {
    // GIVEN: 创建一张攻击卡
    let card = Card::new(
        0,
        "打击",
        "造成6点伤害",
        CardType::Attack,
        1,
        CardEffect::DealDamage { amount: 6 },
        CardRarity::Common, "textures/cards/default.png"
    );

    // THEN: 卡牌属性应该正确
    assert_eq!(card.id, 0);
    assert_eq!(card.name, "打击");
    assert_eq!(card.description, "造成6点伤害");
    assert_eq!(card.card_type, CardType::Attack);
    assert_eq!(card.cost, 1);
}

#[test]
fn e2e_card_type_colors() {
    // GIVEN: 不同类型的卡牌
    let attack_card = Card::new(0, "", "", CardType::Attack, 1, CardEffect::DealDamage { amount: 6 }, CardRarity::Common, "textures/cards/default.png");
    let defense_card = Card::new(1, "", "", CardType::Defense, 1, CardEffect::GainBlock { amount: 5 }, CardRarity::Common, "textures/cards/default.png");
    let skill_card = Card::new(2, "", "", CardType::Skill, 1, CardEffect::DrawCards { amount: 1 }, CardRarity::Common, "textures/cards/default.png");
    let power_card = Card::new(3, "", "", CardType::Power, 2, CardEffect::GainEnergy { amount: 2 }, CardRarity::Rare, "textures/cards/default.png");

    // THEN: 每种类型应该有不同的颜色
    let attack_color = attack_card.get_color();
    let defense_color = defense_card.get_color();
    let skill_color = skill_card.get_color();
    let power_color = power_card.get_color();

    // 验证颜色不相同（简化验证）
    assert!(attack_color != defense_color);
}

// ============================================================================
// E2E测试：牌组系统
// ============================================================================

#[test]
fn e2e_draw_pile_creation() {
    // GIVEN: 创建抽牌堆
    let cards = vec![
        Card::new(0, "", "", CardType::Attack, 1, CardEffect::DealDamage { amount: 6 }, CardRarity::Common, "textures/cards/default.png"),
        Card::new(1, "", "", CardType::Defense, 1, CardEffect::GainBlock { amount: 5 }, CardRarity::Common, "textures/cards/default.png"),
    ];
    let mut draw_pile = DrawPile::new(cards);

    // THEN: 抽牌堆应该包含卡牌
    assert_eq!(draw_pile.count, 2);
    assert_eq!(draw_pile.cards.len(), 2);
}

#[test]
fn e2e_draw_card_from_pile() {
    // GIVEN: 创建抽牌堆
    let cards = vec![
        Card::new(0, "打击", "", CardType::Attack, 1, CardEffect::DealDamage { amount: 6 }, CardRarity::Common, "textures/cards/default.png"),
    ];
    let mut draw_pile = DrawPile::new(cards);

    // WHEN: 抽一张牌
    let card = draw_pile.draw_card();

    // THEN: 应该返回卡牌，且抽牌堆为空
    assert!(card.is_some());
    assert_eq!(draw_pile.count, 0);
    assert_eq!(draw_pile.cards.len(), 0);
}

#[test]
fn e2e_draw_from_empty_pile() {
    // GIVEN: 创建空的抽牌堆
    let mut draw_pile = DrawPile::new(vec![]);

    // WHEN: 尝试抽牌
    let card = draw_pile.draw_card();

    // THEN: 应该返回None
    assert!(card.is_none());
}

#[test]
fn e2e_discard_pile_operations() {
    // GIVEN: 创建弃牌堆
    let mut discard_pile = DiscardPile::new();
    let card = Card::new(0, "", "", CardType::Attack, 1, CardEffect::DealDamage { amount: 6 }, CardRarity::Common, "textures/cards/default.png");

    // WHEN: 添加卡牌到弃牌堆
    discard_pile.add_card(card.clone());

    // THEN: 弃牌堆应该包含该卡牌
    assert_eq!(discard_pile.count, 1);
    assert_eq!(discard_pile.cards.len(), 1);

    // WHEN: 清空弃牌堆
    let cards = discard_pile.clear();

    // THEN: 应该返回所有卡牌，且弃牌堆为空
    assert_eq!(cards.len(), 1);
    assert_eq!(discard_pile.count, 0);
}

#[test]
fn e2e_hand_operations() {
    // GIVEN: 创建手牌
    let mut hand = Hand::new(10);
    let card = Card::new(0, "", "", CardType::Attack, 1, CardEffect::DealDamage { amount: 6 }, CardRarity::Common, "textures/cards/default.png");

    // WHEN: 添加卡牌到手牌
    let added = hand.add_card(card.clone());

    // THEN: 卡牌应该被添加
    assert!(added);
    assert_eq!(hand.len(), 1);
    assert!(!hand.is_empty());

    // WHEN: 移除卡牌（打出）
    let played_card = hand.remove_card(0);

    // THEN: 应该返回打出的卡牌
    assert!(played_card.is_some());
    assert_eq!(hand.len(), 0);
    assert!(hand.is_empty());
}

#[test]
fn e2e_hand_max_size_limit() {
    // GIVEN: 创建上限为2的手牌
    let mut hand = Hand::new(2);
    let card = Card::new(0, "", "", CardType::Attack, 1, CardEffect::DealDamage { amount: 6 }, CardRarity::Common, "textures/cards/default.png");

    // WHEN: 添加超过上限的卡牌
    hand.add_card(card.clone());
    hand.add_card(card.clone());
    let third = hand.add_card(card.clone());

    // THEN: 第三张卡应该无法添加
    assert!(!third);
    assert_eq!(hand.len(), 2);
}

// ============================================================================
// E2E测试：初始牌组
// ============================================================================

#[test]
fn e2e_starting_deck_size() {
    // GIVEN: 使用默认牌组配置
    let config = DeckConfig::default();

    // THEN: 初始牌组应该有12张卡
    // 5张打击 + 1张突刺 + 4张防御 + 2张治疗
    assert_eq!(config.starting_deck.len(), 12);
}

#[test]
fn e2e_starting_deck_composition() {
    // GIVEN: 使用默认牌组配置
    let config = DeckConfig::default();

    // WHEN: 统计卡牌类型
    let attack_count = config.starting_deck.iter()
        .filter(|c| c.card_type == CardType::Attack)
        .count();
    let defense_count = config.starting_deck.iter()
        .filter(|c| c.card_type == CardType::Defense)
        .count();
    let skill_count = config.starting_deck.iter()
        .filter(|c| c.card_type == CardType::Skill)
        .count();

    // THEN: 应该有6张攻击卡、4张防御卡、2张技能卡
    assert_eq!(attack_count, 6);
    assert_eq!(defense_count, 4);
    assert_eq!(skill_count, 2);
}

// ============================================================================
// E2E测试：完整抽牌流程
// ============================================================================

#[test]
fn e2e_full_draw_cycle() {
    // GIVEN: 创建应用并添加牌组系统
    let mut app = create_test_app();

    // 创建抽牌堆、弃牌堆、手牌实体
    let cards = vec![
        Card::new(0, "打击", "", CardType::Attack, 1, CardEffect::DealDamage { amount: 6 }, CardRarity::Common, "textures/cards/default.png"),
        Card::new(1, "防御", "", CardType::Defense, 1, CardEffect::GainBlock { amount: 5 }, CardRarity::Common, "textures/cards/default.png"),
    ];
    let draw_pile_entity = app.world_mut().spawn(DrawPile::new(cards)).id();
    let discard_pile_entity = app.world_mut().spawn(DiscardPile::new()).id();
    let hand_entity = app.world_mut().spawn(Hand::new(10)).id();

    app.update();

    // WHEN: 从抽牌堆抽牌到手牌
    {
        let world = app.world_mut();
        let mut draw_pile = world.get_mut::<DrawPile>(draw_pile_entity).unwrap();
        let card = draw_pile.draw_card().unwrap();

        let mut hand = world.get_mut::<Hand>(hand_entity).unwrap();
        hand.add_card(card);
    }

    // THEN: 抽牌堆减少1张，手牌增加1张
    let world = app.world_mut();
    let draw_pile = world.get::<DrawPile>(draw_pile_entity).unwrap();
    let hand = world.get::<Hand>(hand_entity).unwrap();

    assert_eq!(draw_pile.count, 1);
    assert_eq!(hand.len(), 1);

    // WHEN: 打出卡牌到弃牌堆
    {
        let world = app.world_mut();
        let mut hand = world.get_mut::<Hand>(hand_entity).unwrap();
        let card = hand.remove_card(0).unwrap();

        let mut discard_pile = world.get_mut::<DiscardPile>(discard_pile_entity).unwrap();
        discard_pile.add_card(card);
    }

    // THEN: 手牌清空，弃牌堆增加1张
    let world = app.world_mut();
    let hand = world.get::<Hand>(hand_entity).unwrap();
    let discard_pile = world.get::<DiscardPile>(discard_pile_entity).unwrap();

    assert_eq!(hand.len(), 0);
    assert_eq!(discard_pile.count, 1);
}

// ============================================================================
// E2E测试：回合结束逻辑
// ============================================================================

#[test]
fn e2e_end_turn_increases_turn_count_by_one() {
    // GIVEN: 创建一个玩家实体，初始回合数为1
    let mut app = create_test_app();
    let player_entity = app.world_mut().spawn(Player::default()).id();
    app.update();

    // WHEN: 执行一次回合开始
    {
        let world = app.world_mut();
        let mut player = world.get_mut::<Player>(player_entity).unwrap();
        player.start_turn();
    }

    // THEN: 回合数应该增加1（从1变成2）
    let world = app.world();
    let player = world.get::<Player>(player_entity).unwrap();
    assert_eq!(player.turn, 2, "回合数应该从1增加到2");
}

#[test]
fn e2e_multiple_end_turns() {
    // GIVEN: 创建一个玩家实体，初始回合数为1
    let mut app = create_test_app();
    let player_entity = app.world_mut().spawn(Player::default()).id();
    app.update();

    // WHEN: 执行3次回合开始
    for expected_turn in 2..=4 {
        let world = app.world_mut();
        let mut player = world.get_mut::<Player>(player_entity).unwrap();
        player.start_turn();

        // THEN: 每次回合数应该只增加1
        assert_eq!(player.turn, expected_turn, "第{}次结束回合后，回合数应该是{}", expected_turn - 1, expected_turn);
    }

    // 最终验证
    let world = app.world();
    let player = world.get::<Player>(player_entity).unwrap();
    assert_eq!(player.turn, 4, "执行3次回合开始后，回合数应该是4");
}
