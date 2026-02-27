use bevy::prelude::*;
use bevy_card_battler::components::after_image::{AfterImageConfig, GhostInstance};
use bevy_card_battler::components::hit_stop::HitStopEvent;
use bevy_card_battler::systems::hit_stop::handle_hit_stop_events;
use bevy_card_battler::systems::after_image::{spawn_after_images, LastPosition};

#[test]
fn test_hit_stop_triggers_after_image_snapshot() {
    let mut world = World::new();
    
    // 1. 初始化资源与事件
    let time: Time = Time::default();
    world.insert_resource(time);
    world.insert_resource(Time::<Virtual>::default());
    world.insert_resource(bevy_card_battler::components::hit_stop::HitStopState::new());
    world.insert_resource(Events::<HitStopEvent>::default());
    world.insert_resource(Events::<bevy_card_battler::components::screen_effect::ScreenEffectEvent>::default());

    // 2. 准备带配置的角色实体
    let character = world.spawn((
        Transform::default(),
        AfterImageConfig::default(),
        LastPosition::default(),
    )).id();

    // 3. 模拟发送顿帧事件
    world.resource_mut::<Events<HitStopEvent>>().send(HitStopEvent {
        duration: 0.3,
        speed: 0.01,
    });

    // 4. 按顺序运行两个关键系统 (模拟同一帧内的调度)
    let mut schedule = Schedule::default();
    schedule.add_systems((
        handle_hit_stop_events,
        spawn_after_images,
    ).chain()); // 强制先后顺序
    
    schedule.run(&mut world);

    // 5. 验证是否产生了残影实体
    let ghost_count = world.query::<&GhostInstance>().iter(&world).count();
    assert!(ghost_count > 0, "HitStop MUST trigger at least one after-image snapshot in the SAME frame");
}
