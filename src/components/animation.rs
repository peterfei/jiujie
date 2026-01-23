//! 动画组件定义
//!
//! 提供敌人攻击动画效果，包括：
//! - 移动动画（敌人冲刺）
//! - 震动动画（玩家受击）
//! - 浮动伤害数字
//! - 缓动函数

use bevy::prelude::*;

/// 移动动画组件
///
/// 用于UI元素的平滑移动动画，如敌人攻击时的冲刺效果
#[derive(Component)]
pub struct MovementAnimation {
    /// 起始位置
    pub start: Vec2,
    /// 目标位置
    pub target: Vec2,
    /// 当前进度 (0.0 到 1.0)
    pub progress: f32,
    /// 动画持续时间（秒）
    pub duration: f32,
    /// 已经过的时间
    pub elapsed: f32,
    /// 是否在到达目标后返回起始位置
    pub return_on_complete: bool,
    /// 返回阶段的起始位置（用于return_on_complete）
    pub return_start: Vec2,
    /// 是否处于返回阶段
    pub is_returning: bool,
    /// 缓动函数
    pub easing: EasingFunction,
}

impl MovementAnimation {
    /// 创建一个新的移动动画
    pub fn new(target: Vec2, duration: f32) -> Self {
        Self {
            start: Vec2::ZERO,
            target,
            progress: 0.0,
            duration,
            elapsed: 0.0,
            return_on_complete: false,
            return_start: Vec2::ZERO,
            is_returning: false,
            easing: EasingFunction::EaseOutQuad,
        }
    }

    /// 创建一个往返移动动画
    pub fn with_return(start: Vec2, target: Vec2, duration: f32) -> Self {
        Self {
            start,
            target,
            progress: 0.0,
            duration,
            elapsed: 0.0,
            return_on_complete: true,
            return_start: Vec2::ZERO, // 将在动画开始时设置
            is_returning: false,
            easing: EasingFunction::EaseOutQuad,
        }
    }
}

/// 震动动画组件
///
/// 用于UI元素的震动效果，如玩家受击时的屏幕震动
#[derive(Component)]
pub struct ShakeAnimation {
    /// 原始位置
    pub original: Vec2,
    /// 震动强度
    pub intensity: f32,
    /// 震动持续时间（秒）
    pub duration: f32,
    /// 已经过的时间
    pub elapsed: f32,
    /// 当前震动偏移
    pub current_offset: Vec2,
    /// 震动频率
    pub frequency: f32,
}

impl ShakeAnimation {
    /// 创建一个新的震动动画
    pub fn new(intensity: f32, duration: f32) -> Self {
        Self {
            original: Vec2::ZERO,
            intensity,
            duration,
            elapsed: 0.0,
            current_offset: Vec2::ZERO,
            frequency: 20.0,
        }
    }
}

/// 浮动伤害数字组件
///
/// 用于显示伤害数字的上浮和淡出效果
#[derive(Component)]
pub struct FloatingDamageText {
    /// 浮动速度（像素/秒）
    pub float_speed: f32,
    /// 总持续时间（秒）
    pub duration: f32,
    /// 已经过的时间
    pub elapsed: f32,
    /// 起始透明度
    pub start_alpha: f32,
    /// 结束透明度
    pub end_alpha: f32,
    /// 初始位置
    pub start_position: Vec2,
}

impl FloatingDamageText {
    /// 创建一个新的浮动伤害数字
    pub fn new(float_speed: f32, duration: f32) -> Self {
        Self {
            float_speed,
            duration,
            elapsed: 0.0,
            start_alpha: 1.0,
            end_alpha: 0.0,
            start_position: Vec2::ZERO,
        }
    }
}

/// 敌人UI标记组件
///
/// 用于标识敌人相关的UI元素，方便查找和操作
#[derive(Component)]
pub struct EnemyUiMarker;

/// 玩家UI标记组件
///
/// 用于标识玩家相关的UI元素，方便查找和操作
#[derive(Component)]
pub struct PlayerUiMarker;

/// 缓动函数枚举
///
/// 提供各种常见的缓动函数，用于动画插值
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EasingFunction {
    /// 线性插值
    Linear,
    /// 二次方缓入
    EaseInQuad,
    /// 二次方缓出
    EaseOutQuad,
    /// 二次方缓入缓出
    EaseInOutQuad,
    /// 三次方缓入
    EaseInCubic,
    /// 三次方缓出
    EaseOutCubic,
    /// 三次方缓入缓出
    EaseInOutCubic,
    /// 弹跳效果
    EaseOutBounce,
}

impl EasingFunction {
    /// 应用缓动函数到进度值 t (0.0 到 1.0)
    #[must_use]
    pub fn apply(self, t: f32) -> f32 {
        debug_assert!(t >= 0.0 && t <= 1.0, "缓动函数输入必须在 [0, 1] 范围内");
        match self {
            EasingFunction::Linear => t,
            EasingFunction::EaseInQuad => t * t,
            EasingFunction::EaseOutQuad => {
                let t = 1.0 - t;
                1.0 - t * t
            }
            EasingFunction::EaseInOutQuad => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    let t = 2.0 * t - 1.0;
                    1.0 - 0.5 * t * t
                }
            }
            EasingFunction::EaseInCubic => t * t * t,
            EasingFunction::EaseOutCubic => {
                let t = 1.0 - t;
                1.0 - t * t * t
            }
            EasingFunction::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    let t = 2.0 * t - 2.0;
                    1.0 + 0.5 * t * t * t
                }
            }
            EasingFunction::EaseOutBounce => {
                let n1 = 7.5625;
                let d1 = 2.75;

                if t < 1.0 / d1 {
                    n1 * t * t
                } else if t < 2.0 / d1 {
                    let t = t - 1.5 / d1;
                    n1 * t * t + 0.75
                } else if t < 2.5 / d1 {
                    let t = t - 2.25 / d1;
                    n1 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / d1;
                    n1 * t * t + 0.984375
                }
            }
        }
    }
}

/// 敌人攻击事件
///
/// 当敌人攻击玩家时发送此事件，触发相关动画效果
#[derive(Event, Debug)]
pub struct EnemyAttackEvent {
    /// 造成的伤害值
    pub damage: i32,
    /// 是否破甲（护甲被完全击破）
    pub block_broken: bool,
}

impl EnemyAttackEvent {
    /// 创建一个新的攻击事件
    #[must_use]
    pub const fn new(damage: i32, block_broken: bool) -> Self {
        Self {
            damage,
            block_broken,
        }
    }
}
