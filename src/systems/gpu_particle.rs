//! GPU 粒子 system (基于 bevy_hanabi)

use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use crate::components::particle::{EffectType, SpawnEffectEvent};
use crate::states::GameState;
use std::collections::HashMap;

#[derive(Resource)]
pub struct GpuParticleAssets {
    pub effects: HashMap<EffectType, Handle<EffectAsset>>,
    pub curl_noise: Handle<Image>,
}

#[derive(Component)]
pub struct DespawnTimer(pub Timer);

pub struct GpuParticlePlugin;

impl Plugin for GpuParticlePlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<HanabiPlugin>() {
            app.add_plugins(HanabiPlugin);
        }
        
        app.add_systems(OnEnter(GameState::Combat), setup_gpu_effects);
        app.add_systems(OnEnter(GameState::MainMenu), setup_gpu_effects);
        
        app.add_systems(
            Update,
            (
                handle_gpu_effect_events,
                update_despawn_timers,
            ).run_if(in_state(GameState::Combat).or(in_state(GameState::MainMenu))),
        );
    }
}

fn update_despawn_timers(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DespawnTimer)>,
) {
    for (entity, mut timer) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn setup_gpu_effects(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
    asset_server: Res<AssetServer>,
) {
    let curl_noise_handle = asset_server.load("textures/vfx/noise/curl_noise.png");

    let mut gpu_assets = GpuParticleAssets {
        effects: HashMap::new(),
        curl_noise: curl_noise_handle,
    };

    // --- 1. 墨色爆发 (DemonAura / InkExplosion) ---
    let mut factory = GpuEffectFactory::new(ExprWriter::new())
        .with_name("InkExplosion")
        .with_capacity(4096)
        .with_spawner(SpawnerSettings::once(80.0.into()))
        .with_curl_noise(1.5, gpu_assets.curl_noise.clone());

    let (init_pos_ink, init_vel_ink, init_lifetime_ink) = {
        let w = factory.writer_mut();
        (
            SetPositionSphereModifier { center: w.lit(Vec3::ZERO).expr(), radius: w.lit(0.1).expr(), dimension: ShapeDimension::Volume },
            SetVelocitySphereModifier { center: w.lit(Vec3::ZERO).expr(), speed: w.lit(4.0).expr() },
            SetAttributeModifier::new(Attribute::LIFETIME, w.lit(1.5).expr())
        )
    };

    let mut color_gradient_ink = Gradient::new();
    color_gradient_ink.add_key(0.0, Vec4::new(0.0, 0.0, 0.0, 1.0));
    color_gradient_ink.add_key(0.7, Vec4::new(0.1, 0.1, 0.1, 0.8));
    color_gradient_ink.add_key(1.0, Vec4::new(0.2, 0.2, 0.2, 0.0));

    let effect_ink = effects.add(
        factory.build()
            .init(init_pos_ink)
            .init(init_vel_ink)
            .init(init_lifetime_ink)
            .render(ColorOverLifetimeModifier { gradient: color_gradient_ink, blend: ColorBlendMode::Overwrite, mask: ColorBlendMask::RGBA })
            .render(SizeOverLifetimeModifier { gradient: Gradient::constant(Vec3::splat(0.3)), screen_space_size: false }),
    );
    gpu_assets.effects.insert(EffectType::DemonAura, effect_ink.clone());
    gpu_assets.effects.insert(EffectType::InkExplosion, effect_ink);

    // --- 4. 冲击火花 (ImpactSpark) ---
    let mut factory_spark = GpuEffectFactory::new(ExprWriter::new())
        .with_name("ImpactSpark")
        .with_capacity(1024)
        .with_spawner(SpawnerSettings::once(40.0.into()))
        .with_collision(true);

    let (init_pos_spark, init_vel_spark, init_lifetime_spark, update_accel_spark) = {
        let w = factory_spark.writer_mut();
        (
            SetPositionSphereModifier { center: w.lit(Vec3::ZERO).expr(), radius: w.lit(0.05).expr(), dimension: ShapeDimension::Volume },
            SetVelocitySphereModifier { center: w.lit(Vec3::ZERO).expr(), speed: w.lit(6.0).expr() },
            SetAttributeModifier::new(Attribute::LIFETIME, w.lit(0.3).expr()),
            AccelModifier::new(w.lit(Vec3::new(0.0, -9.8, 0.0)).expr())
        )
    };

    let mut color_gradient_spark = Gradient::new();
    color_gradient_spark.add_key(0.0, Vec4::new(1.0, 0.8, 0.2, 1.0));
    color_gradient_spark.add_key(1.0, Vec4::new(1.0, 0.2, 0.0, 0.0));

    let effect_spark = effects.add(
        factory_spark.build()
            .init(init_pos_spark)
            .init(init_vel_spark)
            .init(init_lifetime_spark)
            .update(update_accel_spark)
            .render(ColorOverLifetimeModifier { gradient: color_gradient_spark, blend: ColorBlendMode::Overwrite, mask: ColorBlendMask::RGBA })
            .render(SizeOverLifetimeModifier { gradient: Gradient::constant(Vec3::splat(0.05)), screen_space_size: false }),
    );
    gpu_assets.effects.insert(EffectType::ImpactSpark, effect_spark);

    // --- 5. 剑气震荡 (SwordEnergy) ---
    let mut factory_sword = GpuEffectFactory::new(ExprWriter::new())
        .with_name("SwordEnergy")
        .with_capacity(2048)
        .with_spawner(SpawnerSettings::once(100.0.into()))
        .with_ribbon_trail(true);

    let (init_pos_sword, init_vel_sword, init_lifetime_sword) = {
        let w = factory_sword.writer_mut();
        (
            SetPositionSphereModifier { center: w.lit(Vec3::ZERO).expr(), radius: w.lit(0.5).expr(), dimension: ShapeDimension::Surface },
            SetVelocitySphereModifier { center: w.lit(Vec3::ZERO).expr(), speed: w.lit(4.0).expr() },
            SetAttributeModifier::new(Attribute::LIFETIME, w.lit(0.5).expr())
        )
    };

    let mut color_gradient_sword = Gradient::new();
    color_gradient_sword.add_key(0.0, Vec4::new(0.4, 0.8, 1.0, 1.0));
    color_gradient_sword.add_key(1.0, Vec4::new(0.1, 0.2, 0.6, 0.0));

    let effect_sword = effects.add(
        factory_sword.build()
            .init(init_pos_sword)
            .init(init_vel_sword)
            .init(init_lifetime_sword)
            .render(ColorOverLifetimeModifier { gradient: color_gradient_sword, blend: ColorBlendMode::Overwrite, mask: ColorBlendMask::RGBA })
            .render(SizeOverLifetimeModifier { gradient: Gradient::constant(Vec3::splat(0.1)), screen_space_size: false }),
    );
    gpu_assets.effects.insert(EffectType::SwordEnergy, effect_sword);

    // --- 12. 雷霆爆发 (ThunderClap) ---
    let mut factory_thunder = GpuEffectFactory::new(ExprWriter::new())
        .with_name("ThunderClap")
        .with_capacity(1024)
        .with_spawner(SpawnerSettings::once(50.0.into()))
        .with_curl_noise(10.0, gpu_assets.curl_noise.clone());

    let (init_pos_thunder, init_vel_thunder, init_lifetime_thunder) = {
        let w = factory_thunder.writer_mut();
        (
            SetPositionSphereModifier { center: w.lit(Vec3::ZERO).expr(), radius: w.lit(0.1).expr(), dimension: ShapeDimension::Volume },
            SetVelocitySphereModifier { center: w.lit(Vec3::ZERO).expr(), speed: w.lit(12.0).expr() },
            SetAttributeModifier::new(Attribute::LIFETIME, w.lit(0.25).expr())
        )
    };

    let mut color_gradient_thunder = Gradient::new();
    color_gradient_thunder.add_key(0.0, Vec4::new(1.0, 1.0, 1.0, 1.0));
    color_gradient_thunder.add_key(0.2, Vec4::new(0.2, 0.6, 1.0, 1.0));
    color_gradient_thunder.add_key(1.0, Vec4::new(0.0, 0.0, 0.5, 0.0));

    let effect_thunder = effects.add(
        factory_thunder.build()
            .init(init_pos_thunder)
            .init(init_vel_thunder)
            .init(init_lifetime_thunder)
            .render(ColorOverLifetimeModifier { gradient: color_gradient_thunder, blend: ColorBlendMode::Overwrite, mask: ColorBlendMask::RGBA })
            .render(SizeOverLifetimeModifier { gradient: Gradient::constant(Vec3::splat(0.15)), screen_space_size: false }),
    );
    gpu_assets.effects.insert(EffectType::ThunderClap, effect_thunder);

    // --- 修复 Fire/Ice 特效：确保粒子布局非空 ---
    let mut writer_fire = ExprWriter::new();
    let init_pos_fire = SetPositionSphereModifier { center: writer_fire.lit(Vec3::ZERO).expr(), radius: writer_fire.lit(0.2).expr(), dimension: ShapeDimension::Volume };
    let init_lifetime_fire = SetAttributeModifier::new(Attribute::LIFETIME, writer_fire.lit(1.0).expr());
    let effect_fire = effects.add(
        EffectAsset::new(1024, SpawnerSettings::rate(20.0.into()), writer_fire.finish())
            .with_name("Fire")
            .init(init_pos_fire)
            .init(init_lifetime_fire)
    );
    gpu_assets.effects.insert(EffectType::Fire, effect_fire);

    let mut writer_ice = ExprWriter::new();
    let init_pos_ice = SetPositionSphereModifier { center: writer_ice.lit(Vec3::ZERO).expr(), radius: writer_ice.lit(0.2).expr(), dimension: ShapeDimension::Volume };
    let init_lifetime_ice = SetAttributeModifier::new(Attribute::LIFETIME, writer_ice.lit(1.0).expr());
    let effect_ice = effects.add(
        EffectAsset::new(1024, SpawnerSettings::rate(20.0.into()), writer_ice.finish())
            .with_name("Ice")
            .init(init_pos_ice)
            .init(init_lifetime_ice)
    );
    gpu_assets.effects.insert(EffectType::Ice, effect_ice);

    commands.insert_resource(gpu_assets);
}

fn handle_gpu_effect_events(
    mut commands: Commands,
    gpu_assets: Res<GpuParticleAssets>,
    mut events: EventReader<SpawnEffectEvent>,
) {
    for event in events.read() {
        let effect_type = match event.effect_type {
            EffectType::Lightning => EffectType::ThunderClap,
            EffectType::Slash => {
                // 如果 VoidSlash 还没做完，先用 ThunderClap 演示，避免崩溃
                if gpu_assets.effects.contains_key(&EffectType::VoidSlash) { EffectType::VoidSlash } else { EffectType::ThunderClap }
            },
            _ => event.effect_type,
        };

        if let Some(effect_handle) = gpu_assets.effects.get(&effect_type) {
            let duration = 2.0;
            commands.spawn((
                ParticleEffect::new(effect_handle.clone()),
                Transform::from_translation(event.position),
                GlobalTransform::default(),
                DespawnTimer(Timer::from_seconds(duration, TimerMode::Once)),
            ));
        }
    }
}

pub struct GpuEffectFactory {
    writer: ExprWriter,
    name: String,
    capacity: u32,
    spawner: SpawnerSettings,
    use_ribbon: bool,
    use_collision: bool,
    curl_noise_strength: f32,
    curl_noise_texture: Option<Handle<Image>>,
}

impl GpuEffectFactory {
    pub fn new(writer: ExprWriter) -> Self {
        Self {
            writer,
            name: "CinematicEffect".to_string(),
            capacity: 2048,
            spawner: SpawnerSettings::once(1.0.into()),
            use_ribbon: false,
            use_collision: false,
            curl_noise_strength: 0.0,
            curl_noise_texture: None,
        }
    }

    pub fn writer_mut(&mut self) -> &mut ExprWriter {
        &mut self.writer
    }

    pub fn with_collision(mut self, enable: bool) -> Self {
        self.use_collision = enable;
        self
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn with_capacity(mut self, capacity: u32) -> Self {
        self.capacity = capacity;
        self
    }

    pub fn with_spawner(mut self, spawner: SpawnerSettings) -> Self {
        self.spawner = spawner;
        self
    }

    pub fn with_curl_noise(mut self, strength: f32, texture: Handle<Image>) -> Self {
        self.curl_noise_strength = strength;
        self.curl_noise_texture = Some(texture);
        self
    }

    pub fn with_ribbon_trail(mut self, enable: bool) -> Self {
        self.use_ribbon = enable;
        self
    }

    pub fn build(mut self) -> EffectAsset {
        let mut asset = EffectAsset::new(self.capacity, self.spawner, self.writer.finish())
            .with_name(self.name);

        if self.curl_noise_strength > 0.0 {
            let writer = ExprWriter::new();
            let pos = writer.attr(Attribute::POSITION);
            let noise = (pos.clone().x().sin() + pos.y().cos()) * writer.lit(self.curl_noise_strength);
            let accel = writer.lit(Vec3::new(0.2, 1.0, 0.2)) * noise;
            asset = asset.update(AccelModifier::new(accel.expr()));
        }

        if self.use_collision {
            let writer = ExprWriter::new();
            let pos = writer.attr(Attribute::POSITION);
            let is_below_ground = (writer.lit(0.0) - pos.y()).max(writer.lit(0.0));
            let bounce = writer.lit(Vec3::new(0.0, 500.0, 0.0)) * is_below_ground;
            asset = asset.update(AccelModifier::new(bounce.expr()));
            asset = asset.update(LinearDragModifier::new(writer.lit(2.0).expr()));
        }

        asset
    }
}
