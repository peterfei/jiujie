//! 音效系统组件与事件

use bevy::prelude::*;

/// 音效类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SfxType {
    /// 出牌音效
    CardPlay,
    /// 天雷落下音效
    LightningStrike,
    /// 境界突破成功音效
    BreakthroughSuccess,
    /// 玩家受击音效
    PlayerHit,
    /// 敌人受击音效
    EnemyHit,
    /// UI点击音效
    UiClick,
}

/// 播放音效事件
#[derive(Event, Debug, Clone)]
pub struct PlaySfxEvent {
    pub sfx_type: SfxType,
    /// 音量 (0.0 - 1.0)
    pub volume: f32,
}

impl PlaySfxEvent {
    pub fn new(sfx_type: SfxType) -> Self {
        Self {
            sfx_type,
            volume: 1.0,
        }
    }

    pub fn with_volume(sfx_type: SfxType, volume: f32) -> Self {
        Self {
            sfx_type,
            volume,
        }
    }
}
