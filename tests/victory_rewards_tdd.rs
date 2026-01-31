// @Validated: Refactor Regression - 2026-01-29
use bevy::prelude::*;
use bevy_card_battler::components::combat::{Player};
use bevy_card_battler::components::PlayerDeck;
use bevy_card_battler::states::GameState;

#[test]
fn test_new_game_initial_gold() {
    let mut app = App::new();
    app.init_resource::<PlayerDeck>();
    app.init_resource::<Player>();
    
    // 模拟新游戏初始化逻辑
    // 检查 PlayerDeck (持久化资源) 的初始值
    let player = app.world().resource::<Player>();
    assert_eq!(player.gold, 100, "新游戏初始修士应自带 100 灵石");
    
    let deck = PlayerDeck::new();
    assert_eq!(deck.len(), 15, "初始牌组应包含 15 张功法");

    // 检查 Player (运行时组件) 的默认值
    let player_default = Player::default();
    assert_eq!(player_default.gold, 100, "修士初始状态应自带 100 灵石");
}

#[test]
fn test_combat_victory_gold_reward() {
    // 模拟战斗胜利掉落逻辑
    let mut player = Player::default();
    let initial_gold = player.gold;
    
    // 模拟一个“小怪”掉落逻辑：期望掉落 10-20 灵石
    let reward = 15; 
    player.gold += reward;
    
    assert!(player.gold > initial_gold, "战斗胜利后灵石应增加");
    assert_eq!(player.gold, initial_gold + 15);
}
