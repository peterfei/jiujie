use bevy::prelude::*;
use bevy_card_battler::components::after_image::{AfterImageConfig, GhostInstance};
use bevy_card_battler::systems::after_image::{LastPosition, spawn_after_images};
use std::time::Duration;

#[test]
fn test_forced_snapshot_without_movement() {
    let mut world = World::new();
    
    // 1. 准备资源
    let time: Time = Time::default();
    world.insert_resource(time);
    
    // 准备静止的角色
    let character = world.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        AfterImageConfig {
            speed_threshold: 10.0,
            force_snapshot: true, // 强制标记！
            ..default()
        },
        LastPosition(Vec3::ZERO),
    )).id();

    // 2. 运行系统逻辑
    let mut schedule = Schedule::default();
    schedule.add_systems(spawn_after_images);
    schedule.run(&mut world);

    // 3. 验证即使速度为 0，也产生了残影
    let ghost_count = world.query::<&GhostInstance>().iter(&world).count();
    assert!(ghost_count > 0, "Force snapshot MUST trigger even if speed is zero");
    
    // 验证标记被清除（防止连发）
    let config = world.get::<AfterImageConfig>(character).unwrap();
    assert_eq!(config.force_snapshot, false, "Force snapshot flag must be cleared after use");
}
