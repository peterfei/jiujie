//! GPU 粒子系统 (极致稳定版 - 彻底消除白块与借用冲突)

use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use crate::components::particle::{EffectType, SpawnEffectEvent};
use crate::states::GameState;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct GpuParticleAssets {
    pub effects: HashMap<EffectType, Handle<EffectAsset>>,
}

#[derive(Component)]
pub struct DespawnTimer(pub Timer);

pub struct GpuParticlePlugin;

impl Plugin for GpuParticlePlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<HanabiPlugin>() {
            app.add_plugins(HanabiPlugin);
        }
        app.init_resource::<GpuParticleAssets>();
        app.add_systems(OnEnter(GameState::Combat), setup_gpu_effects);
        app.add_systems(Update, (
            handle_gpu_effect_events,
            update_despawn_timers,
        ).run_if(in_state(GameState::Combat)));
    }
}

fn update_despawn_timers(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut DespawnTimer)>) {
    for (entity, mut timer) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() { commands.entity(entity).despawn_recursive(); }
    }
}

fn setup_gpu_effects(
    mut gpu_assets: ResMut<GpuParticleAssets>,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    gpu_assets.effects.clear();

    // --- 辅助函数：创建稳健的基础特效 ---
    let mut create_basic_effect = |name: &str, capacity: u32, spawner: SpawnerSettings, color: Vec4, size: f32| {
        let mut writer = ExprWriter::new();
        let init_pos = SetPositionSphereModifier { center: writer.lit(Vec3::ZERO).expr(), radius: writer.lit(0.05).expr(), dimension: ShapeDimension::Volume };
        let init_vel = SetVelocitySphereModifier { center: writer.lit(Vec3::ZERO).expr(), speed: writer.lit(2.0).expr() };
        let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, writer.lit(1.0).expr());
        
        let mut gradient = Gradient::new();
        gradient.add_key(0.0, color);
        gradient.add_key(1.0, Vec4::new(color.x, color.y, color.z, 0.0));

        effects.add(
            EffectAsset::new(capacity, spawner, writer.finish())
                .with_name(name)
                .init(init_pos)
                .init(init_vel)
                .init(init_lifetime)
                .render(ColorOverLifetimeModifier { gradient, blend: ColorBlendMode::Overwrite, mask: ColorBlendMask::RGBA })
                .render(SizeOverLifetimeModifier { gradient: Gradient::constant(Vec3::splat(size)), screen_space_size: false })
        )
    };

    // 1. 先生成核心特效句柄
    let ink = create_basic_effect("InkExplosion", 1024, SpawnerSettings::once(30.0.into()), Vec4::new(0.0, 0.0, 0.0, 1.0), 0.05);
    let spark = create_basic_effect("ImpactSpark", 512, SpawnerSettings::once(20.0.into()), Vec4::new(1.0, 0.8, 0.2, 1.0), 0.03);
    let sword = create_basic_effect("SwordEnergy", 1024, SpawnerSettings::once(50.0.into()), Vec4::new(0.4, 0.8, 1.0, 1.0), 0.04);
    let thunder = create_basic_effect("ThunderClap", 1024, SpawnerSettings::once(40.0.into()), Vec4::new(0.8, 0.9, 1.0, 1.0), 0.06);
    let fire = create_basic_effect("Fire", 512, SpawnerSettings::rate(10.0.into()), Vec4::new(1.0, 0.4, 0.0, 1.0), 0.05);
    let ice = create_basic_effect("Ice", 512, SpawnerSettings::rate(10.0.into()), Vec4::new(0.5, 0.8, 1.0, 1.0), 0.05);

    // 2. 批量插入映射关系，避免借用冲突
    gpu_assets.effects.insert(EffectType::InkExplosion, ink.clone());
    gpu_assets.effects.insert(EffectType::DemonAura, ink);
    
    gpu_assets.effects.insert(EffectType::ImpactSpark, spark.clone());
    gpu_assets.effects.insert(EffectType::Hit, spark);

    gpu_assets.effects.insert(EffectType::SwordEnergy, sword);
    gpu_assets.effects.insert(EffectType::ThunderClap, thunder);
    gpu_assets.effects.insert(EffectType::Fire, fire);
    gpu_assets.effects.insert(EffectType::Ice, ice);
}

fn handle_gpu_effect_events(
    mut commands: Commands,
    gpu_assets: Res<GpuParticleAssets>,
    mut events: EventReader<SpawnEffectEvent>,
) {
    for event in events.read() {
        let effect_type = match event.effect_type {
            EffectType::Lightning => EffectType::ThunderClap,
            EffectType::Slash => EffectType::SwordEnergy,
            _ => event.effect_type,
        };

        if let Some(effect_handle) = gpu_assets.effects.get(&effect_type) {
            commands.spawn((
                ParticleEffect::new(effect_handle.clone()),
                Transform::from_translation(event.position),
                GlobalTransform::default(),
                DespawnTimer(Timer::from_seconds(2.0, TimerMode::Once)),
            ));
        }
    }
}

pub struct GpuEffectFactory { writer: ExprWriter, name: String, capacity: u32, spawner: SpawnerSettings }
impl GpuEffectFactory {
    pub fn new(writer: ExprWriter) -> Self { Self { writer, name: "Effect".to_string(), capacity: 1024, spawner: SpawnerSettings::once(1.0.into()) } }
    pub fn writer_mut(&mut self) -> &mut ExprWriter { &mut self.writer }
    pub fn with_name(mut self, name: &str) -> Self { self.name = name.to_string(); self }
    pub fn build(self) -> EffectAsset { EffectAsset::new(self.capacity, self.spawner, self.writer.finish()).with_name(self.name) }
}
