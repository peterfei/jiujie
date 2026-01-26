//! Sprite 角色组件
//!
//! 用于管理战斗中的角色精灵显示

use bevy::prelude::*;

/// 角色精灵组件
#[derive(Component)]
pub struct CharacterSprite {
    /// 精灵图句柄
    pub texture: Handle<Image>,
    /// 当前动画帧
    pub current_frame: usize,
    /// 动画帧总数
    pub total_frames: usize,
    /// 每帧持续时间（秒）
    pub frame_duration: f32,
    /// 已播放时间
    pub elapsed: f32,
    /// 是否循环播放
    pub looping: bool,
    /// 当前动画状态
    pub state: AnimationState,
    /// 精灵尺寸
    pub size: Vec2,
    /// Z轴层级
    pub z_index: f32,
}

impl CharacterSprite {
    /// 创建一个新的角色精灵
    pub fn new(texture: Handle<Image>, size: Vec2) -> Self {
        Self {
            texture,
            current_frame: 0,
            total_frames: 1,
            frame_duration: 0.1,
            elapsed: 0.0,
            looping: true,
            state: AnimationState::Idle,
            size,
            z_index: 10.0,
        }
    }

    /// 设置为攻击动画
    pub fn set_attack(&mut self, frames: usize, duration: f32) {
        self.state = AnimationState::Attack;
        self.total_frames = frames;
        self.frame_duration = duration / frames as f32;
        self.current_frame = 0;
        self.elapsed = 0.0;
        self.looping = false;
    }

    /// 设置为受击动画
    pub fn set_hit(&mut self, frames: usize, duration: f32) {
        self.state = AnimationState::Hit;
        self.total_frames = frames;
        self.frame_duration = duration / frames as f32;
        self.current_frame = 0;
        self.elapsed = 0.0;
        self.looping = false;
    }

    /// 设置为死亡动画
    pub fn set_death(&mut self, frames: usize, duration: f32) {
        self.state = AnimationState::Death;
        self.total_frames = frames;
        self.frame_duration = duration / frames as f32;
        self.current_frame = 0;
        self.elapsed = 0.0;
        self.looping = false;
    }

    /// 重置为待机动画
    pub fn set_idle(&mut self) {
        self.state = AnimationState::Idle;
        self.total_frames = 1;
        self.current_frame = 0;
        self.elapsed = 0.0;
        self.looping = true;
    }

    /// 是否动画已完成
    pub fn is_finished(&self) -> bool {
        if self.looping {
            return false;
        }
        self.current_frame >= self.total_frames - 1
    }
}

/// 动画状态
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AnimationState {
    /// 待机
    Idle,
    /// 攻击
    Attack,
    /// 受击
    Hit,
    /// 死亡
    Death,
}

/// 角色类型
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CharacterType {
    /// 玩家
    Player,
    /// 普通敌人
    NormalEnemy,
    /// 精英敌人
    EliteEnemy,
    /// Boss
    Boss,
}

/// 角色资源配置
#[derive(Resource)]
pub struct CharacterAssets {
    /// 玩家待机图
    pub player_idle: Handle<Image>,
    /// 玩家攻击图
    pub player_attack: Handle<Image>,
    /// 普通敌人图
    pub normal_enemy: Handle<Image>,
    /// 精英敌人图
    pub elite_enemy: Handle<Image>,
    /// Boss图
    pub boss: Handle<Image>,
}

impl CharacterAssets {
    /// 从资源服务器加载（暂时使用颜色占位）
    pub fn load(_asset_server: &AssetServer) -> Self {
        // TODO: 后续加载真实图片资源
        Self {
            player_idle: Handle::default(),
            player_attack: Handle::default(),
            normal_enemy: Handle::default(),
            elite_enemy: Handle::default(),
            boss: Handle::default(),
        }
    }
}

/// 角色动画事件
#[derive(Event, Debug)]
pub struct CharacterAnimationEvent {
    pub target: Entity,
    pub animation: AnimationState,
}

/// 3D 战斗角色标记组件 (2.5D 纸片人模式)
#[derive(Component)]
pub struct Combatant3d {
    /// 角色面向 (true 为面向右侧)
    pub facing_right: bool,
}

/// 呼吸动画组件（用于 3D 空间中的上下浮动感）
#[derive(Component)]
pub struct BreathAnimation {
    /// 动画计时
    pub timer: f32,
    /// 浮动频率
    pub frequency: f32,
    /// 浮动幅度
    pub amplitude: f32,
}

impl Default for BreathAnimation {
    fn default() -> Self {
        Self {
            timer: 0.0,
            frequency: 1.0,
            amplitude: 0.02,
        }
    }
}

/// 精灵图标记组件
#[derive(Component)]
pub struct SpriteMarker;

/// 玩家精灵标记
#[derive(Component)]
pub struct PlayerSpriteMarker;

/// 敌人精灵标记
#[derive(Component)]
pub struct EnemySpriteMarker;
