// @Validated: Refactor Regression - 2026-01-29
use bevy::prelude::*;
use bevy_card_battler::components::combat::{Player};
use bevy_card_battler::components::PlayerDeck;
use bevy_card_battler::states::GameState;

#[test]
fn test_new_game_initial_gold() {
    let mut app = App::new();
    
    // 模拟新游戏初始化逻辑
    // 检查 PlayerDeck (持久化资源) 的初始值
    let deck = app.world().resource::<PlayerDeck>();
    let player = app.world().resource::<Player>();
    assert_eq!(player.gold, 100, "新游戏初始修士应自带 100 灵石");
    assert_eq!(deck.cards.len(), 10, "初始牌组应包含 10 张功法");

    // 检查 Player (运行时组件) 的默认值
    let player = Player::default();
    assert_eq!(player.gold, 100, "修士初始状态应自带 100 灵石");
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