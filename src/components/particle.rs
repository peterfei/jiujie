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
            target: None, start_pos: Vec2::ZERO, seed: rand::random::<f32>(),
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
            lifetime: (0.3, 0.5), size: (3.0, 10.0), start_color: Color::srgba(1.0, 0.3, 0.1, 0.9),
            end_color: Color::srgba(0.8, 0.1, 0.0, 0.0), speed: (100.0, 250.0),
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
            lifetime: (0.4, 0.6), size: (3.0, 8.0), start_color: Color::srgba(0.9, 0.9, 1.0, 0.9),
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ParticleShape { Circle, Square, Line, Triangle, Star }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectType { Fire, Ice, Hit, Lightning, Victory, ManaFlow, Heal, Coin, AmbientSpirit, SwordEnergy, DemonAura, WebShot, WanJian }

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
            EffectType::SwordEnergy => EmitterConfig::sword_energy(),
            EffectType::DemonAura => EmitterConfig::demon_aura(),
            EffectType::WebShot => EmitterConfig::web_shot(),
            EffectType::WanJian => EmitterConfig::wan_jian(),
        }
    }
}

#[derive(Event, Debug)]
pub struct SpawnEffectEvent { 
    pub effect_type: EffectType, 
    pub position: Vec3, 
    pub burst: bool, 
    pub count: usize,
    pub target: Option<Vec2>, // 新增：目标位置
}

impl SpawnEffectEvent {
    pub fn new(effect_type: EffectType, position: Vec3) -> Self { 
        Self { effect_type, position, burst: true, count: 20, target: None } 
    }
    pub fn burst(mut self, count: usize) -> Self { self.burst = true; self.count = count; self }
    pub fn with_target(mut self, target: Vec2) -> Self { self.target = Some(target); self }
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
