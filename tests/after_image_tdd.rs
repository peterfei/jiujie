use bevy::prelude::*;
use bevy_card_battler::components::after_image::{AfterImageConfig, GhostInstance, TrailSource};
use bevy_card_battler::systems::after_image::{LastPosition, spawn_after_images, sync_trail_emitters};
use bevy_card_battler::systems::gpu_particle::GpuParticleAssets;
use std::time::Duration;

#[test]
fn test_after_image_snapshot_logic() {
    let mut world = World::new();
    
    // 1. 准备资源
    let mut time: Time = Time::default();
    time.advance_by(Duration::from_millis(100));
    world.insert_resource(time);
    
    // 准备角色
    let character = world.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        AfterImageConfig {
            speed_threshold: 10.0,
            snapshot_interval: 0.05,
            timer: Timer::from_seconds(0.05, TimerMode::Repeating),
            ..default()
        },
        LastPosition(Vec3::ZERO),
    )).id();

    // 2. 模拟高速位移
    world.entity_mut(character).get_mut::<Transform>().unwrap().translation.x = 5.0;
    
    // 3. 手动运行系统
    let mut schedule = Schedule::default();
    schedule.add_systems(spawn_after_images);
    schedule.run(&mut world);

    // 4. 验证残影产生
    let ghost_count = world.query::<&GhostInstance>().iter(&world).count();
    assert!(ghost_count > 0, "High speed MUST trigger snapshot. Current count: {}", ghost_count);

    // 5. 验证拖尾激活逻辑
    // 手动建立层级关系
    let trail_entity = world.spawn((
        TrailSource,
        Visibility::Hidden,
    )).id();
    world.entity_mut(trail_entity).set_parent(character);
    
    // 注入 GpuParticleAssets Mock
    world.insert_resource(GpuParticleAssets {
        effects: std::collections::HashMap::new(),
    });

    // 运行同步系统
    let mut schedule_trail = Schedule::default();
    schedule_trail.add_systems(sync_trail_emitters);
    
    // 运行多次以确保系统链路触发
    schedule_trail.run(&mut world);
    
    let visibility = world.get::<Visibility>(trail_entity).unwrap();
    assert_eq!(*visibility, Visibility::Visible, "TrailSource MUST be visible when is_active is true");
}
