//! 屏幕特效组件
//!
//! 提供屏幕震动、闪光等全局视觉特效

use bevy::prelude::*;

/// 屏幕震动组件
#[derive(Component)]
pub struct CameraShake {
    /// 震动强度 (0.0 - 1.0)
    pub trauma: f32,
    /// 震动衰减速率
    pub decay: f32,
    /// 当前震动偏移
    pub offset: Vec2,
}

impl CameraShake {
    /// 创建新的震动效果
    pub fn new(trauma: f32) -> Self {
        Self {
            trauma: trauma.clamp(0.0, 1.0),
            decay: 5.0,
            offset: Vec2::ZERO,
        }
    }

    /// 设置衰减速率
    pub fn with_decay(mut self, decay: f32) -> Self {
        self.decay = decay;
        self
    }

    /// 是否已完成
    pub fn is_finished(&self) -> bool {
        self.trauma <= 0.0
    }
}

/// 屏幕闪光组件
#[derive(Component)]
pub struct ScreenFlash {
    /// 闪光颜色
    pub color: Color,
    /// 持续时间（秒）
    pub duration: f32,
    /// 已经过时间
    pub elapsed: f32,
    /// 初始透明度
    pub start_alpha: f32,
    /// 结束透明度
    pub end_alpha: f32,
}

impl ScreenFlash {
    /// 创建新的闪光效果
    pub fn new(color: Color, duration: f32) -> Self {
        Self {
            color,
            duration,
            elapsed: 0.0,
            start_alpha: 1.0,
            end_alpha: 0.0,
        }
    }

    /// 设置透明度范围
    pub fn with_alpha(mut self, start: f32, end: f32) -> Self {
        self.start_alpha = start;
        self.end_alpha = end;
        self
    }

    /// 计算当前透明度
    pub fn current_alpha(&self) -> f32 {
        let t = (self.elapsed / self.duration).min(1.0);
        self.start_alpha + (self.end_alpha - self.start_alpha) * t
    }

    /// 是否已完成
    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.duration
    }
}

/// 屏幕特效事件
#[derive(Event, Debug)]
pub enum ScreenEffectEvent {
    /// 震动屏幕
    Shake {
        /// 震动强度 (0.0 - 1.0)
        trauma: f32,
        /// 衰减速率
        decay: f32,
    },
    /// 闪光
    Flash {
        /// 闪光颜色
        color: Color,
        /// 持续时间（秒）
        duration: f32,
    },
}

impl ScreenEffectEvent {
    /// 创建震动事件
    pub fn shake(trauma: f32) -> Self {
        Self::Shake {
            trauma,
            decay: 5.0,
        }
    }

    /// 创建闪光事件
    pub fn flash(color: Color, duration: f32) -> Self {
        Self::Flash { color, duration }
    }

    /// 创建强震动事件（如Boss攻击）
    pub fn heavy_shake() -> Self {
        Self::shake(0.8)
    }

    /// 创建轻震动事件（如普通攻击）
    pub fn light_shake() -> Self {
        Self::shake(0.3)
    }

    /// 创建红色闪光（如受击）
    pub fn red_flash(duration: f32) -> Self {
        Self::flash(Color::srgba(1.0, 0.0, 0.0, 0.5), duration)
    }

    /// 创建白色闪光（如治疗）
    pub fn white_flash(duration: f32) -> Self {
        Self::flash(Color::srgba(1.0, 1.0, 1.0, 0.8), duration)
    }
}

/// 屏幕特效标记组件
#[derive(Component)]
pub struct ScreenEffectMarker;
