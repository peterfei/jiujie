//! 游戏状态定义

use bevy::prelude::*;

/// 游戏主要状态
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Reflect)]
pub enum GameState {
    #[default]
    MainMenu,
    /// 序章：交代背景
    Prologue,
    Map,
    Combat,
    Reward,
    Shop,
    Rest,
    GameOver,
    /// 渡劫：面临天雷考验
    Tribulation,
}
