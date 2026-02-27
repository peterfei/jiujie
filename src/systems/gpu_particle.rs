//! GPU 粒子系统 (基于 bevy_hanabi)

use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use crate::components::particle::{EffectType, SpawnEffectEvent};
use crate::states::GameState;
use std::collections::HashMap;

#[derive(Resource)]
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
) {
    let mut gpu_assets = GpuParticleAssets {
        effects: HashMap::new(),
    };

    // --- 1. 墨色爆发 (DemonAura / Armor Break) ---
    let mut writer = ExprWriter::new();
    let mut color_gradient_ink = Gradient::new();
    color_gradient_ink.add_key(0.0, Vec4::new(0.0, 0.0, 0.0, 1.0));
    color_gradient_ink.add_key(0.7, Vec4::new(0.1, 0.1, 0.1, 0.6));
    color_gradient_ink.add_key(1.0, Vec4::new(0.2, 0.2, 0.2, 0.0));

    let mut size_gradient_ink = Gradient::new();
    size_gradient_ink.add_key(0.0, Vec3::splat(0.1));
    size_gradient_ink.add_key(0.2, Vec3::splat(0.6));
    size_gradient_ink.add_key(1.0, Vec3::splat(1.2));

    let init_pos_ink = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.1).expr(),
        dimension: ShapeDimension::Volume,
    };
    let init_vel_ink = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(3.0).expr(),
    };
    let init_lifetime_ink = SetAttributeModifier::new(Attribute::LIFETIME, writer.lit(1.2).expr());

    let effect_ink = effects.add(
        EffectAsset::new(4096, SpawnerSettings::once(60.0.into()), writer.finish())
            .with_name("InkExplosion")
            .init(init_pos_ink)
            .init(init_vel_ink)
            .init(init_lifetime_ink)
            .render(ColorOverLifetimeModifier { 
                gradient: color_gradient_ink,
                blend: ColorBlendMode::Overwrite,
                mask: ColorBlendMask::RGBA,
            })
            .render(SizeOverLifetimeModifier { 
                gradient: size_gradient_ink, 
                screen_space_size: false 
            }),
    );
    gpu_assets.effects.insert(EffectType::DemonAura, effect_ink);

    // --- 2. 真火特效 (Fire) ---
    let mut writer = ExprWriter::new();
    let mut color_gradient_fire = Gradient::new();
    color_gradient_fire.add_key(0.0, Vec4::new(1.0, 0.9, 0.2, 1.0));
    color_gradient_fire.add_key(0.4, Vec4::new(1.0, 0.4, 0.0, 1.0));
    color_gradient_fire.add_key(1.0, Vec4::new(0.2, 0.0, 0.0, 0.0));

    let init_pos_fire = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.2).expr(),
        dimension: ShapeDimension::Volume,
    };
    let init_vel_fire = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(1.5).expr(),
    };
    let init_lifetime_fire = SetAttributeModifier::new(Attribute::LIFETIME, writer.lit(0.8).expr());
    let update_accel_fire = AccelModifier::new(writer.lit(Vec3::new(0.0, 2.0, 0.0)).expr());

    let effect_fire = effects.add(
        EffectAsset::new(2048, SpawnerSettings::rate(50.0.into()), writer.finish())
            .with_name("TrueFire")
            .init(init_pos_fire)
            .init(init_vel_fire)
            .init(init_lifetime_fire)
            .update(update_accel_fire)
            .render(ColorOverLifetimeModifier { 
                gradient: color_gradient_fire,
                blend: ColorBlendMode::Overwrite,
                mask: ColorBlendMask::RGBA,
            })
            .render(SizeOverLifetimeModifier { 
                gradient: Gradient::constant(Vec3::splat(0.15)),
                screen_space_size: false 
            }),
    );
    gpu_assets.effects.insert(EffectType::Fire, effect_fire);

    // --- 3. 寒霜特效 (Ice) ---
    let mut writer = ExprWriter::new();
    let mut color_gradient_ice = Gradient::new();
    color_gradient_ice.add_key(0.0, Vec4::new(0.8, 0.9, 1.0, 1.0));
    color_gradient_ice.add_key(1.0, Vec4::new(0.4, 0.6, 1.0, 0.0));

    let init_pos_ice = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.3).expr(),
        dimension: ShapeDimension::Surface,
    };
    let init_vel_ice = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(0.8).expr(),
    };
    let init_lifetime_ice = SetAttributeModifier::new(Attribute::LIFETIME, writer.lit(1.0).expr());

    let effect_ice = effects.add(
        EffectAsset::new(2048, SpawnerSettings::once(30.0.into()), writer.finish())
            .with_name("FrostBurst")
            .init(init_pos_ice)
            .init(init_vel_ice)
            .init(init_lifetime_ice)
            .render(ColorOverLifetimeModifier { 
                gradient: color_gradient_ice,
                blend: ColorBlendMode::Overwrite,
                mask: ColorBlendMask::RGBA,
            })
            .render(SizeOverLifetimeModifier { 
                gradient: Gradient::constant(Vec3::splat(0.1)),
                screen_space_size: false 
            }),
    );
    gpu_assets.effects.insert(EffectType::Ice, effect_ice);

    // --- 4. 冲击火花 (ImpactSpark) ---
    let mut writer = ExprWriter::new();
    let mut color_gradient_spark = Gradient::new();
    color_gradient_spark.add_key(0.0, Vec4::new(1.5, 1.5, 1.0, 1.0));
    color_gradient_spark.add_key(0.3, Vec4::new(1.0, 0.8, 0.2, 1.0));
    color_gradient_spark.add_key(1.0, Vec4::new(1.0, 0.2, 0.0, 0.0));

    let init_pos_spark = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.05).expr(),
        dimension: ShapeDimension::Volume,
    };
    let init_vel_spark = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(6.0).expr(),
    };
    let init_lifetime_spark = SetAttributeModifier::new(Attribute::LIFETIME, writer.lit(0.3).expr());
    let update_accel_spark = AccelModifier::new(writer.lit(Vec3::new(0.0, -9.8, 0.0)).expr());

    let effect_spark = effects.add(
        EffectAsset::new(1024, SpawnerSettings::once(40.0.into()), writer.finish())
            .with_name("ImpactSpark")
            .init(init_pos_spark)
            .init(init_vel_spark)
            .init(init_lifetime_spark)
            .update(update_accel_spark)
            .render(ColorOverLifetimeModifier { 
                gradient: color_gradient_spark,
                blend: ColorBlendMode::Overwrite,
                mask: ColorBlendMask::RGBA,
            })
            .render(SizeOverLifetimeModifier { 
                gradient: Gradient::constant(Vec3::splat(0.05)),
                screen_space_size: false 
            }),
    );
    gpu_assets.effects.insert(EffectType::ImpactSpark, effect_spark);

    // --- 5. 剑气震荡 (SwordEnergy) ---
    let mut writer = ExprWriter::new();
    let mut color_gradient_sword = Gradient::new();
    color_gradient_sword.add_key(0.0, Vec4::new(0.6, 0.9, 1.0, 1.0));
    color_gradient_sword.add_key(1.0, Vec4::new(0.2, 0.4, 0.8, 0.0));

    let init_pos_sword = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.5).expr(),
        dimension: ShapeDimension::Surface,
    };
    let init_vel_sword = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(4.0).expr(),
    };
    let init_lifetime_sword = SetAttributeModifier::new(Attribute::LIFETIME, writer.lit(0.5).expr());

    let effect_sword = effects.add(
        EffectAsset::new(2048, SpawnerSettings::once(100.0.into()), writer.finish())
            .with_name("SwordEnergy")
            .init(init_pos_sword)
            .init(init_vel_sword)
            .init(init_lifetime_sword)
            .render(ColorOverLifetimeModifier { 
                gradient: color_gradient_sword,
                blend: ColorBlendMode::Overwrite,
                mask: ColorBlendMask::RGBA,
            })
            .render(SizeOverLifetimeModifier { 
                gradient: Gradient::constant(Vec3::splat(0.12)),
                screen_space_size: false 
            }),
    );
    gpu_assets.effects.insert(EffectType::SwordEnergy, effect_sword);

    // --- 6. 灵山云雾 (CloudMist) ---
    let mut writer = ExprWriter::new();
    let mut color_gradient_mist = Gradient::new();
    color_gradient_mist.add_key(0.0, Vec4::new(0.05, 0.05, 0.05, 0.0));
    color_gradient_mist.add_key(0.2, Vec4::new(0.08, 0.08, 0.08, 0.25));
    color_gradient_mist.add_key(0.8, Vec4::new(0.04, 0.04, 0.04, 0.15));
    color_gradient_mist.add_key(1.0, Vec4::new(0.02, 0.02, 0.02, 0.0));

    let mut size_gradient_mist = Gradient::new();
    size_gradient_mist.add_key(0.0, Vec3::splat(2.0));
    size_gradient_mist.add_key(1.0, Vec3::splat(8.0));

    let init_pos_mist = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(5.0).expr(),
        dimension: ShapeDimension::Volume,
    };
    // 赋予一个缓慢向上的浮力
    let init_vel_mist = SetVelocitySphereModifier {
        center: writer.lit(Vec3::new(0.0, -1.0, 0.0)).expr(),
        speed: writer.lit(0.5).expr(),
    };
    let init_lifetime_mist = SetAttributeModifier::new(Attribute::LIFETIME, writer.lit(15.0).expr());
    
    let effect_mist = effects.add(
        EffectAsset::new(1024, SpawnerSettings::rate(5.0.into()), writer.finish())
            .with_name("CloudMist")
            .init(init_pos_mist)
            .init(init_vel_mist)
            .init(init_lifetime_mist)
            .render(ColorOverLifetimeModifier { 
                gradient: color_gradient_mist,
                blend: ColorBlendMode::Overwrite,
                mask: ColorBlendMask::RGBA,
            })
            .render(SizeOverLifetimeModifier { 
                gradient: size_gradient_mist, 
                screen_space_size: false 
            }),
    );
    gpu_assets.effects.insert(EffectType::CloudMist, effect_mist);

    commands.insert_resource(gpu_assets);
}

fn handle_gpu_effect_events(
    mut commands: Commands,
    gpu_assets: Res<GpuParticleAssets>,
    mut events: EventReader<SpawnEffectEvent>,
) {
    for event in events.read() {
        if let Some(effect_handle) = gpu_assets.effects.get(&event.effect_type) {
            // 根据特效类型决定销毁时间
            let duration = match event.effect_type {
                EffectType::DemonAura => 2.0,
                EffectType::Fire => 3.0,
                EffectType::Ice => 2.0,
                EffectType::ImpactSpark => 1.0,
                EffectType::SwordEnergy => 1.5,
                EffectType::CloudMist => 20.0, // 较长的环境背景
                _ => 2.0,
            };

            commands.spawn((
                ParticleEffect::new(effect_handle.clone()),
                Transform::from_translation(event.position),
                GlobalTransform::default(),
                DespawnTimer(Timer::from_seconds(duration, TimerMode::Once)),
            ));
        }
    }
}
