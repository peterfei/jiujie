use bevy::prelude::*;
use bevy_card_battler::components::after_image::{AfterImageConfig, GhostInstance};
use bevy_card_battler::systems::after_image::{LastPosition, spawn_after_images, update_ghosts};
use std::time::Duration;

#[test]
fn test_after_image_expansion_logic() {
    let mut world = World::new();
    
    // 1. 准备资源
    let time: Time = Time::default();
    world.insert_resource(time);
    
    // 准备角色，初始缩放为 1.0
    let character = world.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::ONE),
        AfterImageConfig {
            force_snapshot: true,
            ghost_ttl: 0.8,
            ..default()
        },
        LastPosition(Vec3::ZERO),
    )).id();

    // 2. 运行生成系统
    let mut schedule_gen = Schedule::default();
    schedule_gen.add_systems(spawn_after_images);
    schedule_gen.run(&mut world);

    // 3. 验证初始放大 (1.15x)
    let (ghost_entity, ghost_transform) = world.query::<(Entity, &Transform)>()
        .iter(&world)
        .find(|(e, _)| *e != character)
        .expect("Ghost must be spawned");
    
    assert!(ghost_transform.scale.x > 1.1, "Ghost should be initially larger than character. Got: {}", ghost_transform.scale.x);

    // 4. 运行更新系统并模拟时间流逝
    let initial_scale = ghost_transform.scale.x;
    {
        let mut time_res = world.get_resource_mut::<Time>().unwrap();
        time_res.advance_by(Duration::from_millis(100)); // 经过 0.1s
    }
    
    let mut schedule_update = Schedule::default();
    schedule_update.add_systems(update_ghosts);
    schedule_update.run(&mut world);

    // 5. 验证持续膨胀
    let expanded_transform = world.get::<Transform>(ghost_entity).unwrap();
    assert!(expanded_transform.scale.x > initial_scale, "Ghost should continue expanding over time");
}
