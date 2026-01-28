//! 屏幕特效组件定义

use bevy::prelude::*;

/// 相机震动组件
#[derive(Component, Debug, Clone)]
pub struct CameraShake {
    /// 震动强度 0.0 ~ 1.0 (trauma)
    pub trauma: f32,
    /// 衰减速度
    pub decay: f32,
    /// 当前位移偏移量
    pub offset: Vec2,
    /// 初始相机位置 (用于 3D 相对震动与还原)
    pub base_translation: Option<Vec3>,
}

impl CameraShake {
    pub fn new(trauma: f32) -> Self {
        Self {
            trauma: trauma.clamp(0.0, 1.0),
            decay: 0.8,
            offset: Vec2::ZERO,
            base_translation: None,
        }
    }

    pub fn with_decay(mut self, decay: f32) -> Self {
        self.decay = decay;
        self
    }
}

/// 屏幕闪光组件
#[derive(Component, Debug, Clone)]
pub struct ScreenFlash {
    /// 闪光颜色
    pub color: Color,
    /// 总持续时间
    pub duration: f32,
    /// 已播放时间
    pub elapsed: f32,
}

impl ScreenFlash {
    pub fn new(color: Color, duration: f32) -> Self {
        Self {
            color,
            duration,
            elapsed: 0.0,
        }
    }

    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.duration
    }

    pub fn current_alpha(&self) -> f32 {
        let t = 1.0 - (self.elapsed / self.duration);
        t.clamp(0.0, 1.0)
    }
}

/// 屏幕特效事件
#[derive(Event, Debug, Clone)]
pub enum ScreenEffectEvent {
    /// 相机震动
    Shake {
        trauma: f32,
        decay: f32,
    },
    /// 屏幕闪光
    Flash {
        color: Color,
        duration: f32,
    },
}

impl ScreenEffectEvent {
    pub fn shake(trauma: f32) -> Self {
        Self::Shake { trauma, decay: 0.8 }
    }

    pub fn light_shake() -> Self {
        Self::Shake { trauma: 0.3, decay: 1.0 }
    }

    pub fn heavy_shake() -> Self {
        Self::Shake { trauma: 0.8, decay: 0.5 }
    }

    pub fn flash(color: Color, duration: f32) -> Self {
        Self::Flash { color, duration }
    }

    pub fn red_flash(duration: f32) -> Self {
        Self::Flash { color: Color::srgba(1.0, 0.0, 0.0, 0.5), duration }
    }

    pub fn white_flash(duration: f32) -> Self {
        Self::Flash { color: Color::srgba(1.0, 1.0, 1.0, 0.8), duration }
    }
}

/// 屏幕特效标记（用于 UI 根实体匹配等）
#[derive(Component)]
pub struct ScreenEffectMarker;

/// 屏幕预警标记（低血量或虚弱）
#[derive(Component)]
pub struct ScreenWarning;