use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use bevy_card_battler::components::particle::{EffectType, SpawnEffectEvent, Particle};
use bevy_card_battler::systems::gpu_particle::{GpuParticleAssets};
use bevy_card_battler::systems::vfx_orchestrator::{ParticleAssets, handle_vfx_events};
use bevy_card_battler::states::GameState;

#[test]
fn test_vfx_migration_logic_pure() {
    let mut world = World::new();
    
    // 1. 准备资源
    let time: Time = Time::default();
    world.insert_resource(time);
    world.insert_resource(State::new(GameState::Combat));
    
    // 注入 Assets 存储
    world.insert_resource(Assets::<EffectAsset>::default());
    world.insert_resource(Assets::<Image>::default());
    world.insert_resource(Assets::<Mesh>::default());
    world.insert_resource(Assets::<StandardMaterial>::default());
    
    // 注入 Mock 的 ParticleAssets
    let mut images = world.get_resource_mut::<Assets<Image>>().unwrap();
    let dummy_image = images.add(Image::default());
    let mut textures = std::collections::HashMap::new();
    textures.insert(EffectType::CloudMist, dummy_image.clone());
    world.insert_resource(ParticleAssets {
        textures,
        default_texture: dummy_image,
    });

    // 注入 GpuParticleAssets
    world.insert_resource(GpuParticleAssets {
        effects: std::collections::HashMap::new(),
    });

    // 2. 准备事件系统
    world.insert_resource(Events::<SpawnEffectEvent>::default());
    world.insert_resource(Events::<bevy_card_battler::components::screen_effect::ScreenEffectEvent>::default());

    // 3. 运行逻辑验证
    let mut schedule = Schedule::default();
    schedule.add_systems(handle_vfx_events);

    // --- GPU 类型验证 (CloudMist / MovementTrail) ---
    world.resource_mut::<Events<SpawnEffectEvent>>().send(SpawnEffectEvent::new(EffectType::CloudMist, Vec3::ZERO));
    world.resource_mut::<Events<SpawnEffectEvent>>().send(SpawnEffectEvent::new(EffectType::MovementTrail, Vec3::ZERO));
    
    schedule.run(&mut world);

    let cpu_count = world.query::<&Particle>().iter(&world).count();
    assert_eq!(cpu_count, 0, "GPU-native effects MUST be skipped by orchestrator");

    // --- Orchestrator 类型验证 (WanJian) ---
    world.resource_mut::<Events<SpawnEffectEvent>>().send(SpawnEffectEvent::new(EffectType::WanJian, Vec3::ZERO).burst(5));
    schedule.run(&mut world);

    let wanjian_count = world.query::<&Particle>().iter(&world).count();
    assert!(wanjian_count >= 5, "WanJian MUST spawn CPU entities");
}
