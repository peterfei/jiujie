//! Sprite 角色渲染与物理系统
//!
//! 实现 2.5D 纸片人渲染、物理冲击反馈、呼吸动画及残影特效

use bevy::prelude::*;
use serde::{Serialize, Deserialize};

/// 角色精灵组件
#[derive(Component)]
pub struct CharacterSprite {
    pub texture: Handle<Image>,
    pub size: Vec2,
    pub current_frame: usize,
    pub total_frames: usize,
    pub frame_duration: f32,
    pub elapsed: f32,
    pub state: AnimationState,
    pub looping: bool,
    /// 视觉染色 (覆盖默认白色)
    pub tint: Color,
}

impl CharacterSprite {
    pub fn new(texture: Handle<Image>, size: Vec2) -> Self {
        Self {
            texture, size, current_frame: 0, total_frames: 1,
            frame_duration: 0.1, elapsed: 0.0, state: AnimationState::Idle, looping: true,
            tint: Color::WHITE,
        }
    }

    pub fn with_tint(mut self, color: Color) -> Self {
        self.tint = color;
        self
    }

    pub fn set_idle(&mut self) {
        self.state = AnimationState::Idle;
        self.total_frames = 1;
        self.current_frame = 0;
        self.elapsed = 0.0;
        self.looping = true;
    }

    pub fn set_attack(&mut self, frames: usize, duration: f32) {
        self.state = AnimationState::Attack;
        self.total_frames = frames;
        self.frame_duration = duration / frames as f32;
        self.current_frame = 0;
        self.elapsed = 0.0;
        self.looping = false;
    }

    pub fn set_hit(&mut self, frames: usize, duration: f32) {
        self.state = AnimationState::Hit;
        self.total_frames = frames;
        self.frame_duration = duration / frames as f32;
        self.current_frame = 0;
        self.elapsed = 0.0;
        self.looping = false;
    }

    pub fn set_death(&mut self, frames: usize, duration: f32) {
        self.state = AnimationState::Death;
        self.total_frames = frames;
        self.frame_duration = duration / frames as f32;
        self.current_frame = 0;
        self.elapsed = 0.0;
        self.looping = false;
    }

    /// 标记是否为待机状态
    pub fn is_idle(&self) -> bool {
        self.state == AnimationState::Idle
    }

    pub fn reset_animation(&mut self) {
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
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum AnimationState {
    /// 待机
    Idle,
    /// 受到攻击
    Hit,
    /// 死亡
    Death,
    /// 普通攻击 (向前冲刺)
    Attack,
    /// 御剑术 (高速回旋并小幅位移)
    ImperialSword,
    /// 天象施法 (原地旋转特效)
    HeavenCast,
    /// 防御状态 (原地不动)
    Defense,
    /// 妖兽攻击
    DemonAttack,
    /// 嗜血妖狼专属：奔袭撕咬
    WolfAttack,
    /// [新增] 嗜血妖狼大招：天狼狂袭
    WolfHowl,
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
#[derive(Resource, Default, Clone)]
pub struct CharacterAssets {
    /// 玩家待机图
    pub player_idle: Handle<Image>,
    /// 玩家攻击图
    pub player_attack: Handle<Image>,
    /// 玩家防御图
    pub player_defense: Handle<Image>,
    /// 玩家祈祷图 (天象施法)
    pub player_prise: Handle<Image>,
    /// 嗜血妖狼图
    pub wolf: Handle<Image>,
    /// 剧毒蛛图
    pub spider: Handle<Image>,
    /// 怨灵图
    pub spirit: Handle<Image>,
    /// BOSS图
    pub boss: Handle<Image>,
    // --- 3D 模型资产 ---
    pub player_3d: Option<Handle<Scene>>,
    pub wolf_3d: Option<Handle<Scene>>,
    pub spider_3d: Option<Handle<Scene>>,
    pub spirit_3d: Option<Handle<Scene>>,
    pub boss_3d: Option<Handle<Scene>>,
}

impl CharacterAssets {
    /// 从资源服务器加载
    pub fn load(asset_server: &AssetServer) -> Self {
        Self {
            player_idle: asset_server.load("textures/cards/attack.png"),
            player_attack: asset_server.load("textures/cards/attack.png"),
            player_defense: asset_server.load("textures/cards/defense.png"),
            player_prise: asset_server.load("textures/cards/prise.png"),
            wolf: asset_server.load("textures/enemies/wolf.png"),

            spider: asset_server.load("textures/enemies/spider.png"),
            spirit: asset_server.load("textures/enemies/spirit.png"),
            boss: asset_server.load("textures/enemies/boss.png"),
            // 默认 3D 字段
            player_3d: None,
            wolf_3d: None,
            spider_3d: None,
            spirit_3d: None,
            boss_3d: None,
        }
    }
}

/// 角色动画事件
#[derive(Event, Debug)]
pub struct CharacterAnimationEvent {
    pub target: Entity,
    pub animation: AnimationState,
}

/// 标记战斗中的3D实体
#[derive(Component)]
pub struct Combatant3d {
    pub facing_right: bool,
    /// 基础 Y 轴旋转偏移 (弧度)，用于纠正不同模型的初始朝向
    pub base_rotation: f32,
}

/// 呼吸动画组件
#[derive(Component)]
pub struct BreathAnimation {
    pub timer: f32,
    pub frequency: f32,
    pub amplitude: f32,
}

impl Default for BreathAnimation {
    fn default() -> Self {
        Self { timer: 0.0, frequency: 3.5, amplitude: 0.05 }
    }
}

/// 物理冲击效果组件
#[derive(Component)]
pub struct PhysicalImpact {
    pub home_position: Vec3,
    pub current_offset: Vec3,
    pub offset_velocity: Vec3,
    pub tilt_amount: f32,
    pub tilt_velocity: f32,
    pub action_timer: f32,
    pub action_type: ActionType,
    pub action_direction: f32, // 1.0 向右, -1.0 向左
    pub target_offset_dist: f32,
    pub action_stage: u32,
    pub special_rotation: f32,
    pub special_rotation_velocity: f32,
    /// [新增] 丝迹生成计时器 (用于蜘蛛爬行)
    pub trail_timer: f32,
}

impl Default for PhysicalImpact {
    fn default() -> Self {
        Self {
            home_position: Vec3::ZERO, current_offset: Vec3::ZERO, offset_velocity: Vec3::ZERO,
            tilt_amount: 0.0, tilt_velocity: 0.0, action_timer: 0.0, action_type: ActionType::None,
            action_direction: 1.0, target_offset_dist: 0.0, action_stage: 0,
            special_rotation: 0.0, special_rotation_velocity: 0.0,
            trail_timer: 0.0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActionType { 
    None, 
    Dash, 
    WolfBite, 
    /// [新增] 狼类扑杀：抛物线跳跃攻击
    WolfPounce,
    /// [新增] 狼类大招：多段残影闪击
    SiriusFrenzy,
    SpiderWeb, 
    DemonCast, 
    Ascend,
    /// [新增] 蜘蛛急速爬行：向目标移动并伴随剧烈抖动与丝迹
    SkitterApproach,
    /// [新增] 怨灵大招：多重幻影分身冲击
    SpiritMultiShadow,
}

#[derive(Component)]
pub struct Rotating { pub speed: f32 }

#[derive(Component)]
pub struct Ghost { pub ttl: f32 }

/// 怨灵分身组件
#[derive(Component)]
pub struct SpiritClone {
    pub lifetime: f32,
    pub velocity: Vec3,
    /// [新增] 静止等待时间，结束后开始移动/爆发
    pub delay: f32,
    /// [新增] 随机种子，用于生成不同的运动偏移
    pub seed: f32,
}

/// [新增] 动态云组件
#[derive(Component)]
pub struct Cloud {
    pub scroll_speed: Vec2,
    pub amplitude: f32,
    pub frequency: f32,
    pub seed: f32,
}

/// 标记旋转法阵
#[derive(Component)]
pub struct MagicSealMarker;

/// 标记法宝视觉
#[derive(Component)]
pub struct RelicVisualMarker {
    pub relic_id: crate::components::relic::RelicId,
    pub base_y: f32,
}

/// 标记精灵实体
#[derive(Component)]
pub struct SpriteMarker;

/// 标记玩家精灵
#[derive(Component)]
pub struct PlayerSpriteMarker;

/// 标记敌人精灵
#[derive(Component)]
pub struct EnemySpriteMarker {
    pub id: u32,
}

/// [新增] 动态流雾组件
#[derive(Component)]
pub struct Mist {
    pub scroll_speed: Vec2,
    pub seed: f32,
}

/// [新增] 风场摇曳组件
#[derive(Component)]
pub struct WindSway {
    pub speed: f32,
    pub strength: f32,
    pub seed: f32,
}

/// 战斗摄像机控制器
#[derive(Component)]
pub struct CombatCamera {
    pub distance: f32,
    pub target: Vec3,
    pub rotation_y: f32,
    pub rotation_x: f32,
}

impl Default for CombatCamera {
    fn default() -> Self {
        Self {
            distance: 12.0,
            target: Vec3::new(0.0, 1.2, 0.0),
            rotation_y: 0.0,
            rotation_x: -0.3,
        }
    }
}

/// [新增] 动态水面组件
#[derive(Component)]
pub struct Water {
    pub flow_speed: Vec2,
    pub wave_strength: f32,
}