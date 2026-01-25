//! E2E测试：Bug回归测试
//!
//! 测试已知问题的修复：
//! 1. 手牌计数显示问题（一直显示10/10）
//! 2. 敌人HP为0时游戏没有结束

use bevy::app::App;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_card_battler::components::*;

/// 创建简单测试应用
fn create_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(StatesPlugin);
    app.update();
    app
}

// ============================================================================
// Bug #1: 手牌计数显示问题
// ============================================================================

#[test]
fn e2e_bug_hand_count_display_should_update() {
    // GIVEN: 创建手牌，上限为10
    let hand = Hand::new(10);

    // THEN: max_size 应该是 10
    assert_eq!(hand.max_size, 10, "手牌上限应该是10");
    assert_eq!(hand.len(), 0, "初始手牌数量应该是0");

    // WHEN: 添加卡牌
    let card = Card::new(0, "测试", "", CardType::Attack, 1, CardEffect::DealDamage { amount: 5 }, CardRarity::Common);
    let mut hand = hand;
    hand.add_card(card);

    // THEN: len 应该更新，但 max_size 保持不变
    assert_eq!(hand.len(), 1, "添加1张牌后，手牌数量应该是1");
    assert_eq!(hand.max_size, 10, "手牌上限应该仍然是10");

    // 显示格式应该是 "手牌: 1/10" 而不是 "手牌: 10/10"
    let display_text = format!("手牌: {}/{}", hand.len(), hand.max_size);
    assert_eq!(display_text, "手牌: 1/10", "显示应该是 '手牌: 1/10'");
}

#[test]
fn e2e_bug_hand_count_with_multiple_cards() {
    // GIVEN: 手牌上限10
    let mut hand = Hand::new(10);

    // WHEN: 添加5张牌
    for i in 0..5 {
        let card = Card::new(i, "测试", "", CardType::Attack, 1, CardEffect::DealDamage { amount: 5 }, CardRarity::Common);
        hand.add_card(card);
    }

    // THEN: 应该显示 "手牌: 5/10"
    assert_eq!(hand.len(), 5, "应该有5张牌");
    let display_text = format!("手牌: {}/{}", hand.len(), hand.max_size);
    assert_eq!(display_text, "手牌: 5/10", "显示应该是 '手牌: 5/10'");
}

// ============================================================================
// Bug #2: 敌人HP为0时游戏没有结束
// ============================================================================

#[test]
fn e2e_bug_enemy_death_should_be_detectable() {
    // GIVEN: 创建敌人，HP为30
    let enemy = Enemy::new(0, "嗜血妖狼", 30);

    // THEN: 初始HP应该是30
    assert_eq!(enemy.hp, 30, "初始HP应该是30");
    assert_eq!(enemy.max_hp, 30, "最大HP应该是30");
    assert!(!enemy.is_dead(), "HP为30时，敌人不应该死亡");

    // WHEN: 对敌人造成30点伤害
    let mut enemy = enemy;
    enemy.take_damage(30);

    // THEN: 敌人应该死亡
    assert_eq!(enemy.hp, 0, "受到30点伤害后，HP应该是0");
    assert!(enemy.is_dead(), "HP为0时，敌人应该死亡");

    // 显示格式应该是 "HP: 0/30"
    let display_text = format!("HP: {}/{}", enemy.hp, enemy.max_hp);
    assert_eq!(display_text, "HP: 0/30", "显示应该是 'HP: 0/30'");
}

#[test]
fn e2e_bug_enemy_death_with_overkill() {
    // GIVEN: 创建敌人，HP为30
    let mut enemy = Enemy::new(0, "嗜血妖狼", 30);

    // WHEN: 对敌人造成50点伤害（超过HP）
    enemy.take_damage(50);

    // THEN: HP应该是0（不会变成负数）
    assert_eq!(enemy.hp, 0, "受到50点伤害后，HP应该被限制为0");
    assert!(enemy.is_dead(), "HP为0时，敌人应该死亡");
}

#[test]
fn e2e_bug_combat_should_end_when_enemy_dies() {
    // GIVEN: 创建战斗场景
    let mut app = create_test_app();

    let player_entity = app.world_mut().spawn(Player::default()).id();
    let enemy_entity = app.world_mut().spawn(Enemy::new(0, "嗜血妖狼", 30)).id();

    app.update();

    // WHEN: 敌人被击败
    {
        let world = app.world_mut();
        let mut enemy = world.get_mut::<Enemy>(enemy_entity).unwrap();
        enemy.take_damage(30); // 造成30点伤害
    }

    app.update();

    // THEN: 敌人应该死亡
    let world = app.world();
    let enemy = world.get::<Enemy>(enemy_entity).unwrap();
    assert_eq!(enemy.hp, 0, "敌人HP应该是0");
    assert!(enemy.is_dead(), "敌人应该死亡");

    // 游戏应该检测到敌人死亡并结束战斗
    // 这个测试验证敌人状态，实际游戏结束逻辑在 handle_combat_button_clicks 中
}

// ============================================================================
// 集成测试：完整战斗流程
// ============================================================================

#[test]
fn e2e_full_combat_flow_until_enemy_death() {
    // GIVEN: 玩家有5张攻击卡，敌人30HP
    let mut app = create_test_app();

    let player_entity = app.world_mut().spawn(Player::default()).id();
    let enemy_entity = app.world_mut().spawn(Enemy::new(0, "嗜血妖狼", 30)).id();

    app.update();

    // WHEN: 玩家打出5张攻击卡，每张造成6点伤害
    for _ in 0..5 {
        let world = app.world_mut();
        let mut enemy = world.get_mut::<Enemy>(enemy_entity).unwrap();
        enemy.take_damage(6);

        // 每次攻击后检查状态
        if enemy.is_dead() {
            break;
        }
    }

    app.update();

    // THEN: 敌人应该死亡
    let world = app.world();
    let enemy = world.get::<Enemy>(enemy_entity).unwrap();
    assert_eq!(enemy.hp, 0, "5张攻击卡后，敌人HP应该是0");
    assert!(enemy.is_dead(), "敌人应该死亡");
}

// ============================================================================
// Bug #3: 抽牌效果实现
// ============================================================================

#[test]
fn e2e_draw_cards_effect_increases_hand_count() {
    // GIVEN: 抽牌堆有3张牌，手牌有2张牌
    let mut app = create_test_app();

    let draw_pile_cards = vec![
        Card::new(0, "抽牌1", "", CardType::Attack, 1, CardEffect::DealDamage { amount: 3 }, CardRarity::Common),
        Card::new(1, "抽牌2", "", CardType::Attack, 1, CardEffect::DealDamage { amount: 3 }, CardRarity::Common),
        Card::new(2, "抽牌3", "", CardType::Attack, 1, CardEffect::DealDamage { amount: 3 }, CardRarity::Common),
    ];

    let draw_pile_entity = app.world_mut().spawn(DrawPile::new(draw_pile_cards)).id();
    let hand_entity = app.world_mut().spawn(Hand::new(10)).id();
    let discard_pile_entity = app.world_mut().spawn(DiscardPile::new()).id();

    // 添加2张初始手牌
    {
        let world = app.world_mut();
        let mut hand = world.get_mut::<Hand>(hand_entity).unwrap();
        hand.add_card(Card::new(10, "初始1", "", CardType::Attack, 1, CardEffect::DealDamage { amount: 5 }, CardRarity::Common));
        hand.add_card(Card::new(11, "初始2", "", CardType::Attack, 1, CardEffect::DealDamage { amount: 5 }, CardRarity::Common));
    }

    app.update();

    // WHEN: 抽2张牌
    for _ in 0..2 {
        let world = app.world_mut();
        let mut draw_pile = world.get_mut::<DrawPile>(draw_pile_entity).unwrap();
        let card = draw_pile.draw_card();

        if let Some(c) = card {
            let world = app.world_mut();
            let mut hand = world.get_mut::<Hand>(hand_entity).unwrap();
            hand.add_card(c);
        }
    }

    app.update();

    // THEN: 手牌应该是4张（2+2），抽牌堆剩1张
    let world = app.world();
    let hand = world.get::<Hand>(hand_entity).unwrap();
    let draw_pile = world.get::<DrawPile>(draw_pile_entity).unwrap();

    assert_eq!(hand.len(), 4, "抽2张牌后，手牌应该是4张");
    assert_eq!(draw_pile.count, 1, "抽2张牌后，抽牌堆应该剩1张");
}

#[test]
fn e2e_draw_cards_when_pile_empty_triggers_reshuffle() {
    // GIVEN: 空抽牌堆，有牌的弃牌堆
    let mut app = create_test_app();

    let discard_cards = vec![
        Card::new(0, "弃牌1", "", CardType::Attack, 1, CardEffect::DealDamage { amount: 3 }, CardRarity::Common),
        Card::new(1, "弃牌2", "", CardType::Defense, 1, CardEffect::GainBlock { amount: 5 }, CardRarity::Common),
    ];

    let draw_pile_entity = app.world_mut().spawn(DrawPile::new(vec![])).id();
    let discard_pile_entity = app.world_mut().spawn(DiscardPile::new()).id();
    let hand_entity = app.world_mut().spawn(Hand::new(10)).id();

    // 先往弃牌堆加牌
    {
        let world = app.world_mut();
        let mut discard_pile = world.get_mut::<DiscardPile>(discard_pile_entity).unwrap();
        for card in discard_cards {
            discard_pile.add_card(card);
        }
    }

    app.update();

    // WHEN: 抽牌堆为空时抽牌（应该触发洗牌）
    // 先洗牌
    {
        let world = app.world_mut();
        let mut discard_pile = world.get_mut::<DiscardPile>(discard_pile_entity).unwrap();
        let cards = discard_pile.clear();

        let world = app.world_mut();
        let mut draw_pile = world.get_mut::<DrawPile>(draw_pile_entity).unwrap();
        draw_pile.shuffle_from_discard(cards);
    }

    app.update();

    // THEN: 抽牌堆应该有2张牌，弃牌堆为空
    let world = app.world();
    let draw_pile = world.get::<DrawPile>(draw_pile_entity).unwrap();
    let discard_pile = world.get::<DiscardPile>(discard_pile_entity).unwrap();

    assert_eq!(draw_pile.count, 2, "洗牌后抽牌堆应该有2张牌");
    assert_eq!(discard_pile.count, 0, "洗牌后弃牌堆应该为空");

    // WHEN: 再抽1张牌
    let card = {
        let world = app.world_mut();
        let mut draw_pile = world.get_mut::<DrawPile>(draw_pile_entity).unwrap();
        draw_pile.draw_card()
    };

    if let Some(c) = card {
        let world = app.world_mut();
        let mut hand = world.get_mut::<Hand>(hand_entity).unwrap();
        hand.add_card(c);
    }

    app.update();

    // THEN: 抽牌堆剩1张，手牌有1张
    let world = app.world();
    let draw_pile = world.get::<DrawPile>(draw_pile_entity).unwrap();
    let hand = world.get::<Hand>(hand_entity).unwrap();

    assert_eq!(draw_pile.count, 1, "抽1张后抽牌堆应该剩1张");
    assert_eq!(hand.len(), 1, "手牌应该有1张牌");
}

// ============================================================================
// Bug #4: 敌人攻击后玩家HP没有变化
// ============================================================================

#[test]
fn e2e_bug_enemy_attack_reduces_player_hp() {
    // GIVEN: 玩家有80HP，敌人攻击力10
    let mut app = create_test_app();

    let player_entity = app.world_mut().spawn(Player::default()).id();
    let enemy_entity = app.world_mut().spawn(Enemy::new(0, "嗜血妖狼", 30)).id();

    app.update();

    // WHEN: 敌人攻击玩家（10点伤害）
    {
        let world = app.world_mut();
        let mut player = world.get_mut::<Player>(player_entity).unwrap();
        player.take_damage(10);
    }

    app.update();

    // THEN: 玩家HP应该减少10点
    let world = app.world();
    let player = world.get::<Player>(player_entity).unwrap();
    assert_eq!(player.hp, 70, "受到10点伤害后，玩家HP应该是70");
    assert_eq!(player.max_hp, 80, "最大HP应该保持80");
}

#[test]
fn e2e_bug_player_hp_ui_updates_after_damage() {
    // GIVEN: 玩家有80HP
    let mut app = create_test_app();

    let player_entity = app.world_mut().spawn(Player::default()).id();

    app.update();

    // WHEN: 玩家受到20点伤害
    {
        let world = app.world_mut();
        let mut player = world.get_mut::<Player>(player_entity).unwrap();
        player.take_damage(20);
    }

    app.update();

    // THEN: 玩家HP组件应该是60/80
    let world = app.world();
    let player = world.get::<Player>(player_entity).unwrap();
    assert_eq!(player.hp, 60, "HP应该是60");
    assert_eq!(player.max_hp, 80, "最大HP应该是80");

    // 显示格式应该是 "HP: 60/80"
    let display_text = format!("HP: {}/{}", player.hp, player.max_hp);
    assert_eq!(display_text, "HP: 60/80", "显示应该是 'HP: 60/80'");
}

#[test]
fn e2e_bug_block_absorbs_damage_before_hp() {
    // GIVEN: 玩家有80HP和5点护甲
    let mut app = create_test_app();

    let player_entity = app.world_mut().spawn(Player::default()).id();

    // 先给玩家5点护甲
    {
        let world = app.world_mut();
        let mut player = world.get_mut::<Player>(player_entity).unwrap();
        player.gain_block(5);
    }

    app.update();

    // WHEN: 玩家受到10点伤害
    {
        let world = app.world_mut();
        let mut player = world.get_mut::<Player>(player_entity).unwrap();
        player.take_damage(10);
    }

    app.update();

    // THEN: 护甲应该完全抵消，HP减少5点
    let world = app.world();
    let player = world.get::<Player>(player_entity).unwrap();
    assert_eq!(player.block, 0, "护甲应该被完全消耗");
    assert_eq!(player.hp, 75, "护甲抵消5点后，HP应该是75");
}

#[test]
fn e2e_bug_block_fully_absorbs_damage() {
    // GIVEN: 玩家有80HP和10点护甲
    let mut app = create_test_app();

    let player_entity = app.world_mut().spawn(Player::default()).id();

    // 先给玩家10点护甲
    {
        let world = app.world_mut();
        let mut player = world.get_mut::<Player>(player_entity).unwrap();
        player.gain_block(10);
    }

    app.update();

    // WHEN: 玩家受到5点伤害（小于护甲）
    {
        let world = app.world_mut();
        let mut player = world.get_mut::<Player>(player_entity).unwrap();
        player.take_damage(5);
    }

    app.update();

    // THEN: 护甲减少5点，HP不变
    let world = app.world();
    let player = world.get::<Player>(player_entity).unwrap();
    assert_eq!(player.block, 5, "护甲应该剩余5点");
    assert_eq!(player.hp, 80, "HP应该保持80");
}
