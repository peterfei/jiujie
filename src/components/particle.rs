//! 粒子特效组件
//!
//! 用于战斗中的视觉特效，如火焰、冰霜、闪电等

use bevy::prelude::*;
use serde::{Serialize, Deserialize};

/// 粒子组件
#[derive(Component)]
pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub lifetime: f32,
    pub elapsed: f32,
    pub start_size: f32,
    pub end_size: f32,
    pub start_color: Color,
    pub end_color: Color,
    pub rotation_speed: f32,
    pub rotation: f32,
    pub gravity: Vec2,
    /// 目标位置 (用于导引)
    pub target: Option<Vec2>,
    /// 目标实体
    pub target_entity: Option<Entity>,
    /// 所有存活敌人的位置和实体（用于动态重定向）
    pub target_group: Option<Vec<(Entity, Vec2)>>,
    /// 当前粒子在目标组中的索引（用于循环分配）
    pub target_index: Option<usize>,
    /// 第四相位锁定位置 (用于固定贝塞尔曲线起点)
    pub lock_pos: Option<Vec2>,
    /// 初始位置
    pub start_pos: Vec2,
    /// 随机种子
    pub seed: f32,
    /// 特效类型
    pub effect_type: EffectType,
}

impl Particle {
    pub fn new(lifetime: f32) -> Self {
        Self {
            position: Vec2::ZERO, velocity: Vec2::ZERO, lifetime, elapsed: 0.0,
            start_size: 10.0, end_size: 0.0, start_color: Color::WHITE, end_color: Color::srgba(0.0, 0.0, 0.0, 0.0),
            rotation_speed: 0.0, rotation: 0.0, gravity: Vec2::ZERO,
            target: None, target_entity: None, target_group: None, target_index: None, lock_pos: None,
            start_pos: Vec2::ZERO, seed: rand::random::<f32>(),
            effect_type: EffectType::Hit,
        }
    }

    pub fn with_type(mut self, effect_type: EffectType) -> Self {
        self.effect_type = effect_type;
        self
    }

    pub fn current_size(&self) -> f32 {
        let t = (self.elapsed / self.lifetime).min(1.0);
        self.start_size + (self.end_size - self.start_size) * t
    }

    pub fn current_color(&self) -> Color {
        let t = (self.elapsed / self.lifetime).min(1.0);
        lerp_color(&self.start_color, &self.end_color, t)
    }

    pub fn is_dead(&self) -> bool { self.elapsed >= self.lifetime }
}

fn lerp_color(a: &Color, b: &Color, t: f32) -> Color {
    let a_rgba: Srgba = (*a).into();
    let b_rgba: Srgba = (*b).into();
    Color::srgba(
        a_rgba.red + (b_rgba.red - a_rgba.red) * t,
        a_rgba.green + (b_rgba.green - a_rgba.green) * t,
        a_rgba.blue + (b_rgba.blue - a_rgba.blue) * t,
        a_rgba.alpha + (b_rgba.alpha - a_rgba.alpha) * t,
    )
}

/// 粒子发射器组件
#[derive(Component)]
pub struct ParticleEmitter {
    pub rate: f32,
    pub timer: f32,
    pub max_particles: usize,
    pub emitted_count: usize,
    pub looping: bool,
    pub duration: f32,
    pub elapsed: f32,
    pub config: EmitterConfig,
    pub effect_type: EffectType, // 新增：记录生成的粒子类型
}

impl ParticleEmitter {
    pub fn new(rate: f32, config: EmitterConfig) -> Self {
        Self {
            rate, timer: 0.0, max_particles: 100, emitted_count: 0,
            looping: true, duration: 0.0, elapsed: 0.0, config, effect_type: EffectType::Hit,
        }
    }

    pub fn with_type(mut self, effect_type: EffectType) -> Self {
        self.effect_type = effect_type;
        self
    }

    pub fn once(mut self, count: usize) -> Self {
        self.max_particles = count;
        self.looping = false;
        self.duration = 0.1; // 爆发模式通常很短
        self
    }

    pub fn with_duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }
}

/// 发射器配置
#[derive(Clone)]
pub struct EmitterConfig {
    pub lifetime: (f32, f32),
    pub size: (f32, f32),
    pub start_color: Color,
    pub end_color: Color,
    pub speed: (f32, f32),
    pub angle: (f32, f32),
    pub gravity: Vec2,
    pub rotation_speed: (f32, f32),
    pub shape: ParticleShape,
}

impl EmitterConfig {
    pub fn fire() -> Self {
        Self {
            lifetime: (0.5, 1.0), size: (20.0, 50.0), start_color: Color::srgb(1.0, 0.8, 0.2),
            end_color: Color::srgba(1.0, 0.3, 0.0, 0.0), speed: (50.0, 120.0),
            angle: (-std::f32::consts::PI / 3.0, -std::f32::consts::PI * 2.0 / 3.0),
            gravity: Vec2::new(0.0, -80.0), rotation_speed: (-5.0, 5.0), shape: ParticleShape::Circle,
        }
    }

    pub fn ice() -> Self {
        Self {
            lifetime: (0.4, 0.8), size: (5.0, 15.0), start_color: Color::srgb(0.8, 0.95, 1.0),
            end_color: Color::srgba(0.5, 0.8, 1.0, 0.0), speed: (30.0, 80.0),
            angle: (0.0, std::f32::consts::PI * 2.0), gravity: Vec2::new(0.0, -30.0),
            rotation_speed: (-3.0, 3.0), shape: ParticleShape::Square,
        }
    }

    pub fn poison() -> Self {
        Self {
            lifetime: (0.8, 1.5), size: (10.0, 25.0), start_color: Color::srgb(0.2, 0.8, 0.2),
            end_color: Color::srgba(0.1, 0.4, 0.1, 0.0), speed: (20.0, 50.0),
            angle: (0.0, std::f32::consts::PI * 2.0), gravity: Vec2::new(0.0, 10.0),
            rotation_speed: (-2.0, 2.0), shape: ParticleShape::Circle,
        }
    }

    pub fn lightning() -> Self {
        Self {
            lifetime: (0.1, 0.3), size: (3.0, 8.0), start_color: Color::srgb(0.8, 0.8, 1.0),
            end_color: Color::srgba(0.5, 0.5, 1.0, 0.0), speed: (100.0, 200.0),
            angle: (0.0, std::f32::consts::PI * 2.0), gravity: Vec2::ZERO,
            rotation_speed: (-10.0, 10.0), shape: ParticleShape::Line,
        }
    }

    pub fn heal() -> Self {
        Self {
            lifetime: (0.5, 1.0), size: (5.0, 12.0), start_color: Color::srgb(0.4, 1.0, 0.4),
            end_color: Color::srgba(0.2, 0.8, 0.2, 0.0), speed: (30.0, 60.0),
            angle: (0.0, std::f32::consts::PI * 2.0), gravity: Vec2::new(0.0, 50.0),
            rotation_speed: (-2.0, 2.0), shape: ParticleShape::Star,
        }
    }

    pub fn hit() -> Self {
        Self {
            lifetime: (0.3, 0.6), size: (5.0, 15.0), start_color: Color::srgb(1.0, 1.0, 1.0),
            end_color: Color::srgba(1.0, 0.2, 0.2, 0.0), speed: (80.0, 150.0),
            angle: (0.0, std::f32::consts::PI * 2.0), gravity: Vec2::ZERO,
            rotation_speed: (-5.0, 5.0), shape: ParticleShape::Circle,
        }
    }

    pub fn coin() -> Self {
        Self {
            lifetime: (0.8, 1.5), size: (10.0, 20.0), start_color: Color::srgb(1.0, 0.84, 0.0),
            end_color: Color::srgba(1.0, 0.5, 0.0, 0.0), speed: (40.0, 100.0),
            angle: (-std::f32::consts::PI / 4.0, -std::f32::consts::PI * 3.0 / 4.0),
            gravity: Vec2::new(0.0, -150.0), rotation_speed: (-3.0, 3.0), shape: ParticleShape::Star,
        }
    }

    pub fn victory() -> Self {
        Self {
            lifetime: (2.0, 3.5), size: (8.0, 20.0), start_color: Color::srgb(1.0, 0.9, 0.3),
            end_color: Color::srgba(1.0, 0.5, 0.0, 0.0), speed: (100.0, 300.0),
            angle: (0.0, std::f32::consts::PI * 2.0), gravity: Vec2::new(0.0, -100.0),
            rotation_speed: (-5.0, 5.0), shape: ParticleShape::Star,
        }
    }

    pub fn mana_flow() -> Self {
        Self {
            lifetime: (1.0, 2.0), size: (15.0, 35.0), start_color: Color::srgba(0.2, 0.7, 1.0, 0.9),
            end_color: Color::srgba(0.0, 0.2, 0.5, 0.0), speed: (40.0, 80.0),
            angle: (std::f32::consts::PI * 0.4, std::f32::consts::PI * 0.6),
            gravity: Vec2::new(0.0, 20.0), rotation_speed: (-2.0, 2.0), shape: ParticleShape::Circle,
        }
    }

    pub fn ambient_spirit() -> Self {
        Self {
            lifetime: (3.0, 5.0), size: (5.0, 15.0), start_color: Color::srgba(0.5, 1.0, 0.8, 0.4),
            end_color: Color::srgba(0.1, 0.4, 0.3, 0.0), speed: (10.0, 40.0),
            angle: (0.0, std::f32::consts::PI * 2.0), gravity: Vec2::new(0.0, 5.0),
            rotation_speed: (-1.0, 1.0), shape: ParticleShape::Circle,
        }
    }

    pub fn sword_energy() -> Self {
        Self {
            lifetime: (0.4, 0.6), size: (10.0, 25.0), start_color: Color::srgba(1.0, 0.3, 0.1, 0.9),
            end_color: Color::srgba(0.8, 0.1, 0.0, 0.0), speed: (200.0, 450.0),
            angle: (0.0, std::f32::consts::PI * 2.0), gravity: Vec2::ZERO,
            rotation_speed: (20.0, 40.0), shape: ParticleShape::Line,
        }
    }

    pub fn demon_aura() -> Self {
        Self {
            lifetime: (0.8, 1.2), size: (15.0, 30.0), start_color: Color::srgba(0.3, 0.0, 0.5, 0.8),
            end_color: Color::srgba(0.0, 0.0, 0.0, 0.0), speed: (50.0, 100.0),
            angle: (std::f32::consts::PI * 0.4, std::f32::consts::PI * 0.6),
            gravity: Vec2::new(0.0, 10.0), rotation_speed: (-2.0, 2.0), shape: ParticleShape::Circle,
        }
    }

    pub fn web_shot() -> Self {
        Self {
            lifetime: (0.8, 1.2), size: (15.0, 25.0), start_color: Color::srgba(0.9, 0.9, 1.0, 0.9),
            end_color: Color::srgba(0.7, 0.7, 0.8, 0.0), speed: (250.0, 400.0),
            angle: (std::f32::consts::PI * 0.9, std::f32::consts::PI * 1.1),
            gravity: Vec2::new(0.0, -20.0), rotation_speed: (0.0, 0.0), shape: ParticleShape::Line,
        }
    }

    pub fn wan_jian() -> Self {
        Self {
            lifetime: (1.5, 2.5), size: (12.0, 22.0), start_color: Color::srgba(1.8, 1.4, 0.3, 1.0),
            end_color: Color::srgba(1.0, 0.2, 0.0, 0.0), speed: (0.0, 0.0),
            angle: (0.0, 0.0), gravity: Vec2::ZERO, rotation_speed: (0.0, 0.0), shape: ParticleShape::Line,
        }
    }

    /// 撞击火花 - 高亮度的微小爆炸效果
    pub fn impact_spark() -> Self {
        Self {
            lifetime: (0.15, 0.35), size: (2.0, 6.0), start_color: Color::srgba(1.0, 1.0, 0.9, 1.0),
            end_color: Color::srgba(1.0, 0.6, 0.2, 0.0), speed: (150.0, 350.0),
            angle: (0.0, std::f32::consts::PI * 2.0), gravity: Vec2::new(0.0, -100.0),
            rotation_speed: (-15.0, 15.0), shape: ParticleShape::Star,
        }
    }

    /// 斩击剑气 - 极速的线性爆发
    pub fn slash() -> Self {
        Self {
            lifetime: (0.6, 0.8), size: (15.0, 35.0), start_color: Color::srgba(1.0, 0.2, 0.2, 1.0),
            end_color: Color::srgba(0.8, 0.0, 0.0, 0.0), speed: (300.0, 500.0),
            angle: (-std::f32::consts::PI / 4.0, std::f32::consts::PI / 4.0), // 稍宽一点
            gravity: Vec2::ZERO, rotation_speed: (0.0, 0.0), shape: ParticleShape::Line,
        }
    }

    /// 灵能护盾 - 持续的环绕光晕
    pub fn shield() -> Self {
        Self {
            lifetime: (1.0, 1.5), size: (10.0, 25.0), start_color: Color::srgba(0.2, 0.8, 1.0, 0.6),
            end_color: Color::srgba(0.0, 0.4, 0.8, 0.0), speed: (10.0, 30.0),
            angle: (0.0, std::f32::consts::PI * 2.0), gravity: Vec2::ZERO,
            rotation_speed: (-1.0, 1.0), shape: ParticleShape::Circle,
        }
    }

    /// 灵山云雾 - 水墨写意风格 (类三国志11)
    pub fn cloud_mist() -> Self {
        Self {
            lifetime: (12.0, 18.0),
            size: (800.0, 1500.0), 
            start_color: Color::srgba(0.06, 0.06, 0.06, 0.0), 
            end_color: Color::srgba(0.02, 0.02, 0.02, 0.0),
            // [史诗级改进] 初始向上初速度
            speed: (25.0, 55.0),
            angle: (std::f32::consts::PI * 0.45, std::f32::consts::PI * 0.55), // 集中向上方发射
            // [史诗级改进] 向上升腾的浮力感
            gravity: Vec2::new(0.0, 15.0), 
            rotation_speed: (-0.05, 0.05),
            shape: ParticleShape::Circle,
        }
    }

    pub fn silk_trail() -> Self {
        Self {
            lifetime: (1.5, 2.5),
            size: (10.0, 20.0),
            start_color: Color::srgba(0.9, 0.9, 1.0, 0.6),
            end_color: Color::srgba(0.7, 0.7, 0.8, 0.0),
            speed: (2.0, 10.0),
            angle: (0.0, std::f32::consts::PI * 2.0),
            gravity: Vec2::ZERO,
            rotation_speed: (-0.2, 0.2),
            shape: ParticleShape::Circle,
        }
    }

    pub fn wolf_slash() -> Self {
        Self {
            lifetime: (0.4, 0.6),
            size: (30.0, 50.0),
            start_color: Color::srgba(1.2, 0.1, 0.1, 0.9), // 绯红色 (带超亮自发光感)
            end_color: Color::srgba(0.4, 0.0, 0.0, 0.0),   // 深红渐变消失
            speed: (5.0, 15.0),
            angle: (0.0, 0.3), // 集中方向
            gravity: Vec2::ZERO,
            rotation_speed: (0.0, 0.0),
            shape: ParticleShape::Line,
        }
    }

    pub fn spawn_particle(&self, position: Vec3, effect_type: EffectType) -> Particle {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let lifetime = self.lifetime.0 + rng.gen::<f32>() * (self.lifetime.1 - self.lifetime.0);
        let size = self.size.0 + rng.gen::<f32>() * (self.size.1 - self.size.0);
        let speed = self.speed.0 + rng.gen::<f32>() * (self.speed.1 - self.speed.0);
        let angle = self.angle.0 + rng.gen::<f32>() * (self.angle.1 - self.angle.0);
        let rotation_speed = self.rotation_speed.0 + rng.gen::<f32>() * (self.rotation_speed.1 - self.rotation_speed.0);
        let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;
        let mut p = Particle::new(lifetime).with_type(effect_type);
        p.position = position.truncate(); p.start_pos = position.truncate(); p.velocity = velocity;
        p.start_size = size; p.end_size = size * 0.3; p.start_color = self.start_color; p.end_color = self.end_color;
        p.rotation_speed = rotation_speed; p.gravity = self.gravity;
        p
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleShape { Circle, Square, Line, Triangle, Star }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectType { Fire, Ice, Poison, Hit, Lightning, Victory, ManaFlow, Heal, Coin, AmbientSpirit, SwordEnergy, DemonAura, WebShot, WanJian, ImpactSpark, Slash, Shield, CloudMist, SilkTrail, WolfSlash }

impl EffectType {
    pub fn config(&self) -> EmitterConfig {
        match self {
            EffectType::Fire => EmitterConfig::fire(),
            EffectType::Ice => EmitterConfig::ice(),
            EffectType::Poison => EmitterConfig::poison(),
            EffectType::Lightning => EmitterConfig::lightning(),
            EffectType::Heal => EmitterConfig::heal(),
            EffectType::Hit => EmitterConfig::hit(),
            EffectType::Coin => EmitterConfig::coin(),
            EffectType::Victory => EmitterConfig::victory(),
            EffectType::ManaFlow => EmitterConfig::mana_flow(),
            EffectType::AmbientSpirit => EmitterConfig::ambient_spirit(),
            EffectType::SwordEnergy => EmitterConfig::sword_energy(),
            EffectType::DemonAura => EmitterConfig::demon_aura(),
            EffectType::WebShot => EmitterConfig::web_shot(),
            EffectType::WanJian => EmitterConfig::wan_jian(),
            EffectType::ImpactSpark => EmitterConfig::impact_spark(),
            EffectType::Slash => EmitterConfig::slash(),
            EffectType::Shield => EmitterConfig::shield(),
            EffectType::CloudMist => EmitterConfig::cloud_mist(),
            EffectType::SilkTrail => EmitterConfig::silk_trail(),
            EffectType::WolfSlash => EmitterConfig::wolf_slash(),
        }
    }
}

#[derive(Event, Debug, Clone)]
pub struct SpawnEffectEvent {
    pub effect_type: EffectType,
    pub position: Vec3,
    pub count: u32,
    /// [新增] 覆盖初速度：用于生成式渲染，使用 Vec2 匹配粒子系统
    pub velocity_override: Option<Vec2>,
    /// 万剑归宗等系统需要的锁定目标坐标
    pub target_pos: Option<Vec2>,
    /// 万剑归宗等系统需要的锁定目标实体
    pub target_entity: Option<Entity>,
    /// 万剑归宗需要的全目标列表 (用于避障)
    pub target_group: Vec<(Entity, Vec2)>,
    /// 万剑归宗需要的个体索引
    pub target_index: usize,
}

impl SpawnEffectEvent {
    pub fn new(effect_type: EffectType, position: Vec3) -> Self {
        Self { 
            effect_type, 
            position, 
            count: 1, 
            velocity_override: None,
            target_pos: None,
            target_entity: None,
            target_group: Vec::new(),
            target_index: 0,
        }
    }
    
    pub fn burst(mut self, count: u32) -> Self {
        self.count = count;
        self
    }

    pub fn with_target(mut self, pos: Vec2) -> Self {
        self.target_pos = Some(pos);
        self
    }

    pub fn with_target_entity(mut self, entity: Entity) -> Self {
        self.target_entity = Some(entity);
        self
    }

    pub fn with_target_group(mut self, group: Vec<(Entity, Vec2)>) -> Self {
        self.target_group = group;
        self
    }

    pub fn with_target_index(mut self, index: usize) -> Self {
        self.target_index = index;
        self
    }
}

#[derive(Component)]
pub struct ParticleMarker;
#[derive(Component)]
pub struct EmitterMarker;

#[derive(Component)]
pub struct EnemyDeathAnimation { pub progress: f32, pub duration: f32, pub elapsed: f32 }
impl EnemyDeathAnimation { pub fn new(duration: f32) -> Self { Self { progress: 0.0, duration, elapsed: 0.0 } } }

#[derive(Event, Debug)]
pub struct VictoryEvent;

/// 闪电分段组件
#[derive(Component)]
pub struct LightningBolt {
    pub points: Vec<Vec3>,
    pub ttl: f32,
    pub max_ttl: f32,
    pub alpha: f32,
    pub is_light: bool, // 是否是光源实体
}

/// 环境残痕组件
#[derive(Component)]
pub struct Decal {
    pub ttl: f32,
    pub max_ttl: f32,
}

impl Decal {
    pub fn new(duration: f32) -> Self {
        Self { ttl: duration, max_ttl: duration }
    }
}

impl LightningBolt {
    pub fn new(points: Vec<Vec3>, ttl: f32, is_light: bool) -> Self {
        Self { points, ttl, max_ttl: ttl, alpha: 1.0, is_light }
    }
}
