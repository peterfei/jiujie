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
    /// 御剑术 (270度回旋斩)
    ImperialSword,
    /// 妖物突袭 (沉重撞击)
    DemonAttack,
    /// 嗜血妖狼专属：奔袭撕咬
    WolfAttack,
    /// 剧毒蛛专属：爬行吐丝
    SpiderAttack,
    /// 怨灵专属：灵体突袭
    SpiritAttack,
    /// BOSS 专属：啸天 (全屏 AOE)
    BossRoar,
    /// BOSS 专属：瞬狱杀 (连击)
    BossFrenzy,
    /// 施展妖术 (蓄力/护盾/强化)
    DemonCast,
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
    /// 嗜血妖狼
    DemonicWolf,
    /// 剧毒蛛
    PoisonSpider,
    /// 怨灵
    CursedSpirit,
    /// 筑基大妖 (BOSS)
    GreatDemon,
}

/// 角色资源配置
#[derive(Resource)]
pub struct CharacterAssets {
    /// 玩家待机图
    pub player_idle: Handle<Image>,
    /// 玩家攻击图
    pub player_attack: Handle<Image>,
    /// 嗜血妖狼图
    pub wolf: Handle<Image>,
    /// 剧毒蛛图
    pub spider: Handle<Image>,
    /// 怨灵图
    pub spirit: Handle<Image>,
    /// BOSS图
    pub boss: Handle<Image>,
}

impl CharacterAssets {
    /// 从资源服务器加载
    pub fn load(asset_server: &AssetServer) -> Self {
        Self {
            player_idle: asset_server.load("textures/cards/attack.png"),
            player_attack: asset_server.load("textures/cards/attack.png"),
            wolf: asset_server.load("textures/enemies/wolf.png"),
            spider: asset_server.load("textures/enemies/spider.png"),
            spirit: asset_server.load("textures/enemies/spirit.png"),
            boss: asset_server.load("textures/enemies/boss.png"),
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

/// 动作类型（用于区分物理反馈行为）
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ActionType {
    #[default]
    None,
    WolfBite,
    SpiderWeb,
    DemonCast,
}

/// 物理冲击组件（用于立牌的倾斜和晃动效果）
#[derive(Component)]
pub struct PhysicalImpact {
    /// 角色初始位置 (回弹目标点)
    pub home_position: Vec3,
    /// 当前倾斜角度 (弧度)
    pub tilt_velocity: f32,
    /// 当前倾斜量
    pub tilt_amount: f32,
    /// 招式回旋角度 (用于特殊招式，如 270 度旋转)
    pub special_rotation: f32,
    /// 招式回旋速度
    pub special_rotation_velocity: f32,
    /// 动作计时 (用于多阶段招式)
    pub action_timer: f32,
    /// 当前执行的动作类型
    pub action_type: ActionType,
    /// 动作方向记录 (-1.0 代表向左，1.0 代表向右)
    pub action_direction: f32,
    /// 目标位移总距离 (动态计算，解决多敌人位置偏差)
    pub target_offset_dist: f32,
    /// 目标位置偏移
    pub offset_velocity: Vec3,
    /// 当前位置偏移
    pub current_offset: Vec3,
}

impl Default for PhysicalImpact {
    fn default() -> Self {
        Self {
            home_position: Vec3::ZERO,
            tilt_velocity: 0.0,
            tilt_amount: 0.0,
            special_rotation: 0.0,
            special_rotation_velocity: 0.0,
            action_timer: 0.0,
            action_type: ActionType::None,
            action_direction: 1.0,
            target_offset_dist: 0.0,
            offset_velocity: Vec3::ZERO,
            current_offset: Vec3::ZERO,
        }
    }
}

/// 3D 旋转组件
#[derive(Component)]
pub struct Rotating {
    pub speed: f32,
}

/// 法阵标记组件 (用于亮度脉动)
#[derive(Component)]
pub struct MagicSealMarker;

/// 精灵标记
#[derive(Component)]
pub struct SpriteMarker;

/// 玩家精灵标记
#[derive(Component)]
pub struct PlayerSpriteMarker;

/// 敌人精灵标记
#[derive(Component)]
pub struct EnemySpriteMarker {
    pub id: u32,
}

/// 法宝 3D 视觉标记
#[derive(Component)]
pub struct RelicVisualMarker {
    pub relic_id: crate::components::relic::RelicId,
    pub base_y: f32,
}

/// 残影组件 (Ghost Trail)
#[derive(Component)]
pub struct Ghost {
    /// 存活时间
    pub ttl: f32,
}
