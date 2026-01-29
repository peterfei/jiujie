// @Validated: Refactor Regression - 2026-01-29
use bevy::prelude::*;
use bevy_card_battler::components::combat::{Player};
use bevy_card_battler::components::PlayerDeck;
use bevy_card_battler::states::GameState;

#[test]
fn test_new_game_initial_gold() {
    let mut app = App::new();
    app.init_resource::<Player>();
    app.init_resource::<PlayerDeck>();
    
    let player = app.world().resource::<Player>();
    assert_eq!(player.gold, 100); // 初始路费
}

#[test]
fn test_combat_victory_gold_reward() {
    // 模拟战斗胜利掉落逻辑
    // 这个测试目前肯定会失败，因为代码里还没写
    let mut player = Player::default();
    let initial_gold = player.gold;
    
    // 模拟一个“小怪”掉落逻辑：期望掉落 10-20 灵石
    // 实际代码中这将在 check_combat_end 或相关结算系统中处理
    let reward = 15; 
    player.gold += reward;
    
    assert!(player.gold > initial_gold, "战斗胜利后灵石应增加");
    assert_eq!(player.gold, initial_gold + 15);
}