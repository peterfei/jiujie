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
    /// 结算奖励状态
    Reward,
    /// 商店状态
    Shop,
    /// 休息状态
    Rest,
    /// 渡劫状态
    Tribulation,
    /// 机缘事件状态
    Event,
    /// 游戏结束状态
    GameOver,
}
