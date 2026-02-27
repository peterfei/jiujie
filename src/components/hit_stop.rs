use bevy::prelude::*;

/// 顿帧请求事件
#[derive(Event)]
pub struct HitStopEvent {
    pub duration: f32, // 持续时间 (秒)
    pub speed: f32,    // 缩放速度 (通常为 0.05 - 0.1)
}

/// 全局顿帧状态
#[derive(Resource, Default)]
pub struct HitStopState {
    pub timer: Timer,
    pub original_speed: f32,
    pub is_active: bool,
}

impl HitStopState {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(0.0, TimerMode::Once),
            original_speed: 1.0,
            is_active: false,
        }
    }
}
