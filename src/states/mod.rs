//! 游戏状态定义

use bevy::prelude::*;

/// 游戏主要状态
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect)]
pub enum GameState {
    #[default]
    MainMenu,
    Map,
    Combat,
    Reward,
    GameOver,
}
