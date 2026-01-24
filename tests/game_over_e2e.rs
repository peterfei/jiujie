//! 游戏结束界面功能测试
//!
//! 验证 reset 方法和状态转换逻辑

use bevy_card_battler::components::{PlayerDeck, MapProgress, MapConfig};
use bevy_card_battler::components::{Card, CardType, CardEffect, CardRarity};

// ============================================================================
// 测试 PlayerDeck::reset()
// ============================================================================

#[test]
fn test_player_deck_reset_clears_extra_cards() {
    let mut deck = PlayerDeck::new();

    // 添加额外卡牌（模拟奖励）
    for i in 0..5 {
        deck.add_card(Card::new(
            3000 + i, "奖励卡", "测试奖励卡", CardType::Skill,
            1, CardEffect::Heal { amount: 10 }, CardRarity::Rare,
        ));
    }

    assert_eq!(deck.len(), 17, "添加奖励卡后应该有17张（12初始+5奖励）");

    // 调用 reset
    deck.reset();

    assert_eq!(deck.len(), 12, "reset 后应该恢复为12张");
}

#[test]
fn test_player_deck_reset_creates_valid_deck() {
    let mut deck = PlayerDeck::new();

    // 修改牌组
    deck.cards.clear();
    assert!(deck.is_empty(), "清空后应该为空");

    // 调用 reset
    deck.reset();

    assert!(!deck.is_empty(), "reset 后应该不为空");
    assert_eq!(deck.len(), 12, "reset 后应该有12张卡");

    // 验证都是有效卡牌
    for card in &deck.cards {
        assert!(!card.name.is_empty(), "每张卡应该有名称");
        assert!(card.cost >= 0, "能量消耗应该非负");
    }
}

// ============================================================================
// 测试 MapProgress::reset()
// ============================================================================

#[test]
fn test_map_progress_reset_clears_progress() {
    let mut progress = MapProgress::new(&MapConfig::default());

    // 设置一些进度
    progress.set_current_node(5);
    progress.complete_current_node();

    let layer_before = progress.current_layer;
    assert!(layer_before >= 1, "应该有进度");

    // 调用 reset
    progress.reset();

    // 验证重置
    assert_eq!(progress.current_layer, 0, "reset 后层数应该为0");
    assert_eq!(progress.current_node_id, None, "reset 后当前节点应该为None");

    // 验证所有节点都被重置
    let completed_count = progress.nodes.iter().filter(|n| n.completed).count();
    assert_eq!(completed_count, 0, "reset 后不应该有已完成节点");

    // 验证第0层节点解锁
    let first_layer_unlocked = progress.nodes.iter()
        .filter(|n| n.position.0 == 0)
        .all(|n| n.unlocked);
    assert!(first_layer_unlocked, "第0层应该全部解锁");

    // 验证其他层锁定
    let other_layers_locked = progress.nodes.iter()
        .filter(|n| n.position.0 > 0)
        .all(|n| !n.unlocked);
    assert!(other_layers_locked, "其他层应该被锁定");
}

#[test]
fn test_map_progress_default_creates_valid_map() {
    let progress = MapProgress::default();

    // 验证创建了节点
    assert!(!progress.nodes.is_empty(), "应该有地图节点");

    // 验证初始状态
    assert_eq!(progress.current_layer, 0, "初始层数应该为0");
    assert_eq!(progress.current_node_id, None, "初始节点应该为None");
    assert!(!progress.game_completed, "游戏不应该完成");

    // 验证第0层解锁
    let first_layer_count = progress.nodes.iter()
        .filter(|n| n.position.0 == 0)
        .count();
    assert!(first_layer_count > 0, "应该有第0层节点");

    let first_layer_unlocked = progress.nodes.iter()
        .filter(|n| n.position.0 == 0)
        .all(|n| n.unlocked);
    assert!(first_layer_unlocked, "第0层应该解锁");
}

// ============================================================================
// 测试多次重置循环
// ============================================================================

#[test]
fn test_multiple_reset_cycles() {
    let mut deck = PlayerDeck::new();
    let mut progress = MapProgress::new(&MapConfig::default());

    // 添加卡牌并设置进度
    deck.add_card(Card::new(
        9999, "测试卡", "测试", CardType::Attack,
        1, CardEffect::DealDamage { amount: 1 }, CardRarity::Common,
    ));
    progress.set_current_node(3);

    // 执行多次重置
    for cycle in 0..5 {
        deck.reset();
        progress.reset();

        assert_eq!(deck.len(), 12, "循环 {}: 牌组应该重置为12张", cycle + 1);
        assert_eq!(progress.current_layer, 0, "循环 {}: 地图应该重置", cycle + 1);
    }
}

// ============================================================================
// 状态覆盖验证
// ============================================================================

#[test]
fn test_progress_preservation() {
    let mut deck = PlayerDeck::new();
    let mut progress = MapProgress::new(&MapConfig::default());

    // 添加奖励卡
    deck.add_card(Card::new(
        5000, "永久奖励", "永久保留", CardType::Skill,
        2, CardEffect::Heal { amount: 20 }, CardRarity::Rare,
    ));

    // 完成第0层
    progress.set_current_node(0);
    progress.complete_current_node();

    // 完成第1层，增加层数
    progress.set_current_node(4);
    progress.complete_current_node();

    // 保存当前状态
    let deck_size_before = deck.len();
    let layer_before = progress.current_layer;

    // 执行 reset（模拟重新开始）
    deck.reset();
    progress.reset();

    // 验证 reset 清除了进度
    assert!(deck.len() < deck_size_before, "reset 应该清除奖励卡");
    assert_eq!(progress.current_layer, 0, "reset 应该重置层数为0");
    assert!(progress.current_layer < layer_before, "reset 后层数应该小于之前");
}
