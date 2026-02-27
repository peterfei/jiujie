use bevy::prelude::*;

/// 残影与拖尾配置
#[derive(Component, Debug, Clone)]
pub struct AfterImageConfig {
    pub speed_threshold: f32,      // 触发速度阈值 (单位/秒)
    pub snapshot_interval: f32,    // 产生残影的间隔
    pub ghost_ttl: f32,            // 残影存活时间
    pub color: Color,              // 残影与拖尾主色调
    pub timer: Timer,              // 内部节奏控制器
    pub is_active: bool,           // 当前是否处于高速移动状态
    pub force_snapshot: bool,      // [新增] 强制触发一次快照标记
}

impl Default for AfterImageConfig {
    fn default() -> Self {
        Self {
            speed_threshold: 10.0,
            snapshot_interval: 0.1,
            ghost_ttl: 0.5,
            color: Color::srgba(0.0, 0.8, 1.0, 0.6),
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            is_active: false,
            force_snapshot: false,
        }
    }
}

/// 残影实例组件
#[derive(Component)]
pub struct GhostInstance {
    pub ttl: Timer,
}

/// 标记角色上的拖尾发射点
#[derive(Component)]
pub struct TrailSource;
