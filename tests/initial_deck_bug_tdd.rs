use bevy::prelude::*;
use bevy_card_battler::components::combat::{Player, PlayerDeck};

#[test]
fn test_new_game_must_have_cards() {
    let deck = PlayerDeck::new();
    // 基础牌组通常有 10 张牌 (5打击+5防御)
    assert!(deck.cards.len() >= 5, "新轮回开始时必须拥有基础功法，当前数量: {}", deck.cards.len());
    println!("✅ 初始牌组检查通过：数量={}", deck.cards.len());
}

#[test]
fn test_new_game_initial_gold_and_hp() {
    let deck = PlayerDeck::new();
    assert_eq!(deck.gold, 100, "初始灵石应为 100");
    assert_eq!(deck.max_hp, 80, "初始血量上限应为 80");
}
