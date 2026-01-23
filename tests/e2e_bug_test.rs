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
    let enemy = Enemy::new(0, "哥布林", 30);

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
    let mut enemy = Enemy::new(0, "哥布林", 30);

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
    let enemy_entity = app.world_mut().spawn(Enemy::new(0, "哥布林", 30)).id();

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
    let enemy_entity = app.world_mut().spawn(Enemy::new(0, "哥布林", 30)).id();

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
