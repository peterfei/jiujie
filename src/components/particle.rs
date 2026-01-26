//! 粒子特效组件
//!
//! 用于战斗中的视觉特效，如火焰、冰霜、闪电等

use bevy::prelude::*;

/// 粒子组件
#[derive(Component)]
pub struct Particle {
    /// 逻辑位置
    pub position: Vec2,
    /// 速度
    pub velocity: Vec2,
    /// 生命周期（秒）
    pub lifetime: f32,
    /// 已存活时间
    pub elapsed: f32,
    /// 初始大小
    pub start_size: f32,
    /// 结束大小
    pub end_size: f32,
    /// 初始颜色
    pub start_color: Color,
    /// 结束颜色
    pub end_color: Color,
    /// 旋转速度
    pub rotation_speed: f32,
    /// 当前旋转角度
    pub rotation: f32,
    /// 重力
    pub gravity: Vec2,
}

impl Particle {
    /// 创建一个新粒子
    pub fn new(lifetime: f32) -> Self {
        Self {
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            lifetime,
            elapsed: 0.0,
            start_size: 10.0,
            end_size: 0.0,
            start_color: Color::WHITE,
            end_color: Color::srgba(0.0, 0.0, 0.0, 0.0),
            rotation_speed: 0.0,
            rotation: 0.0,
            gravity: Vec2::ZERO,
        }
    }

    /// 计算当前大小
    pub fn current_size(&self) -> f32 {
        let t = (self.elapsed / self.lifetime).min(1.0);
        self.start_size + (self.end_size - self.start_size) * t
    }

    /// 计算当前颜色
    pub fn current_color(&self) -> Color {
        let t = (self.elapsed / self.lifetime).min(1.0);
        lerp_color(&self.start_color, &self.end_color, t)
    }

    /// 是否已死亡
    pub fn is_dead(&self) -> bool {
        self.elapsed >= self.lifetime
    }
}

/// 颜色插值
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
    /// 每秒发射粒子数
    pub rate: f32,
    /// 发射计时器
    pub timer: f32,
    /// 粒子总数限制
    pub max_particles: usize,
    /// 已发射粒子数
    pub emitted_count: usize,
    /// 是否循环发射
    pub looping: bool,
    /// 发射持续时间（0表示无限）
    pub duration: f32,
    /// 已运行时间
    pub elapsed: f32,
    /// 发射器配置
    pub config: EmitterConfig,
}

impl ParticleEmitter {
    /// 创建新的粒子发射器
    pub fn new(rate: f32, config: EmitterConfig) -> Self {
        Self {
            rate,
            timer: 0.0,
            max_particles: 100,
            emitted_count: 0,
            looping: true,
            duration: 0.0,
            elapsed: 0.0,
            config,
        }
    }

    /// 设置一次性爆发
    pub fn once(mut self, count: usize) -> Self {
        self.looping = false;
        self.max_particles = count;
        self.rate = count as f32;
        self.duration = 0.1;
        self
    }

    /// 设置持续时间
    pub fn with_duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }
}

/// 发射器配置
#[derive(Clone)]
pub struct EmitterConfig {
    /// 粒子生命周期范围
    pub lifetime: (f32, f32),
    /// 粒子大小范围
    pub size: (f32, f32),
    /// 粒子初始颜色
    pub start_color: Color,
    /// 粒子结束颜色
    pub end_color: Color,
    /// 发射速度范围
    pub speed: (f32, f32),
    /// 发射角度范围（弧度）
    pub angle: (f32, f32),
    /// 重力
    pub gravity: Vec2,
    /// 旋转速度范围
    pub rotation_speed: (f32, f32),
    /// 粒子形状
    pub shape: ParticleShape,
}

impl EmitterConfig {
    pub fn fire() -> Self {
        Self {
            lifetime: (0.5, 1.0),
            size: (20.0, 50.0),
            start_color: Color::srgb(1.0, 0.8, 0.2),
            end_color: Color::srgba(1.0, 0.3, 0.0, 0.0),
            speed: (50.0, 120.0),
            angle: (-std::f32::consts::PI / 3.0, -std::f32::consts::PI * 2.0 / 3.0),
            gravity: Vec2::new(0.0, -80.0),
            rotation_speed: (-5.0, 5.0),
            shape: ParticleShape::Circle,
        }
    }

    pub fn ice() -> Self {
        Self {
            lifetime: (0.4, 0.8),
            size: (5.0, 15.0),
            start_color: Color::srgb(0.8, 0.95, 1.0),
            end_color: Color::srgba(0.5, 0.8, 1.0, 0.0),
            speed: (30.0, 80.0),
            angle: (0.0, std::f32::consts::PI * 2.0),
            gravity: Vec2::new(0.0, -30.0),
            rotation_speed: (-3.0, 3.0),
            shape: ParticleShape::Square,
        }
    }

    pub fn lightning() -> Self {
        Self {
            lifetime: (0.1, 0.3),
            size: (3.0, 8.0),
            start_color: Color::srgb(0.8, 0.8, 1.0),
            end_color: Color::srgba(0.5, 0.5, 1.0, 0.0),
            speed: (100.0, 200.0),
            angle: (0.0, std::f32::consts::PI * 2.0),
            gravity: Vec2::ZERO,
            rotation_speed: (-10.0, 10.0),
            shape: ParticleShape::Line,
        }
    }

    pub fn heal() -> Self {
        Self {
            lifetime: (0.5, 1.0),
            size: (5.0, 12.0),
            start_color: Color::srgb(0.4, 1.0, 0.4),
            end_color: Color::srgba(0.2, 0.8, 0.2, 0.0),
            speed: (30.0, 60.0),
            angle: (0.0, std::f32::consts::PI * 2.0),
            gravity: Vec2::new(0.0, 50.0),
            rotation_speed: (-2.0, 2.0),
            shape: ParticleShape::Star,
        }
    }

    pub fn hit() -> Self {
        Self {
            lifetime: (0.3, 0.6),
            size: (5.0, 15.0),
            start_color: Color::srgb(1.0, 1.0, 1.0),
            end_color: Color::srgba(1.0, 0.2, 0.2, 0.0),
            speed: (80.0, 150.0),
            angle: (0.0, std::f32::consts::PI * 2.0),
            gravity: Vec2::ZERO,
            rotation_speed: (-5.0, 5.0),
            shape: ParticleShape::Circle,
        }
    }

    pub fn coin() -> Self {
        Self {
            lifetime: (0.8, 1.5),
            size: (10.0, 20.0),
            start_color: Color::srgb(1.0, 0.84, 0.0),
            end_color: Color::srgba(1.0, 0.5, 0.0, 0.0),
            speed: (40.0, 100.0),
            angle: (-std::f32::consts::PI / 4.0, -std::f32::consts::PI * 3.0 / 4.0),
            gravity: Vec2::new(0.0, -150.0),
            rotation_speed: (-3.0, 3.0),
            shape: ParticleShape::Star,
        }
    }

    pub fn victory() -> Self {
        Self {
            lifetime: (2.0, 3.5),
            size: (8.0, 20.0),
            start_color: Color::srgb(1.0, 0.9, 0.3),
            end_color: Color::srgba(1.0, 0.5, 0.0, 0.0),
            speed: (100.0, 300.0),
            angle: (0.0, std::f32::consts::PI * 2.0),
            gravity: Vec2::new(0.0, -100.0),
            rotation_speed: (-5.0, 5.0),
            shape: ParticleShape::Star,
        }
    }

    pub fn mana_flow() -> Self {
        Self {
            lifetime: (1.0, 2.0),
            size: (15.0, 35.0), // 显著增大粒子
            start_color: Color::srgba(0.2, 0.7, 1.0, 0.9), // 极其明亮的青蓝色
            end_color: Color::srgba(0.0, 0.2, 0.5, 0.0),
            speed: (40.0, 80.0), // 提高流动速度
            angle: (std::f32::consts::PI * 0.4, std::f32::consts::PI * 0.6), // 向上喷涌
            gravity: Vec2::new(0.0, 20.0), 
            rotation_speed: (-2.0, 2.0),
            shape: ParticleShape::Circle,
        }
    }

    pub fn ambient_spirit() -> Self {
        Self {
            lifetime: (3.0, 5.0),
            size: (5.0, 15.0),
            start_color: Color::srgba(0.5, 1.0, 0.8, 0.4),
            end_color: Color::srgba(0.1, 0.4, 0.3, 0.0),
            speed: (10.0, 40.0),
            angle: (0.0, std::f32::consts::PI * 2.0),
            gravity: Vec2::new(0.0, 5.0),
            rotation_speed: (-1.0, 1.0),
            shape: ParticleShape::Circle,
        }
    }

    pub fn spawn_particle(&self, position: Vec3) -> Particle {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let lifetime = self.lifetime.0 + rng.gen::<f32>() * (self.lifetime.1 - self.lifetime.0);
        let size = self.size.0 + rng.gen::<f32>() * (self.size.1 - self.size.0);
        let speed = self.speed.0 + rng.gen::<f32>() * (self.speed.1 - self.speed.0);
        let angle = self.angle.0 + rng.gen::<f32>() * (self.angle.1 - self.angle.0);
        let rotation_speed = self.rotation_speed.0 + rng.gen::<f32>() * (self.rotation_speed.1 - self.rotation_speed.0);
        let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;
        Particle {
            position: position.truncate(),
            velocity, lifetime, elapsed: 0.0, start_size: size, end_size: size * 0.3,
            start_color: self.start_color, end_color: self.end_color, rotation_speed, rotation: 0.0, gravity: self.gravity,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ParticleShape { Circle, Square, Line, Triangle, Star }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EffectType { Fire, Ice, Hit, Lightning, Victory, ManaFlow, Heal, Coin, AmbientSpirit }

impl EffectType {
    pub fn config(&self) -> EmitterConfig {
        match self {
            EffectType::Fire => EmitterConfig::fire(),
            EffectType::Ice => EmitterConfig::ice(),
            EffectType::Lightning => EmitterConfig::lightning(),
            EffectType::Heal => EmitterConfig::heal(),
            EffectType::Hit => EmitterConfig::hit(),
            EffectType::Coin => EmitterConfig::coin(),
            EffectType::Victory => EmitterConfig::victory(),
            EffectType::ManaFlow => EmitterConfig::mana_flow(),
            EffectType::AmbientSpirit => EmitterConfig::ambient_spirit(),
        }
    }
}

#[derive(Event, Debug)]
pub struct SpawnEffectEvent { pub effect_type: EffectType, pub position: Vec3, pub burst: bool, pub count: usize }

impl SpawnEffectEvent {
    pub fn new(effect_type: EffectType, position: Vec3) -> Self { Self { effect_type, position, burst: true, count: 20 } }
    pub fn burst(mut self, count: usize) -> Self { self.burst = true; self.count = count; self }
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