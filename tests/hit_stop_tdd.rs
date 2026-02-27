use bevy::prelude::*;
use bevy_card_battler::components::hit_stop::{HitStopEvent, HitStopState};
use bevy_card_battler::systems::hit_stop::{handle_hit_stop_events, update_hit_stop_timer};
use std::time::Duration;

#[test]
fn test_hit_stop_pure_logic() {
    let mut world = World::new();
    
    // 1. 初始化资源
    world.insert_resource(Time::<Real>::default());
    world.insert_resource(Time::<Virtual>::default());
    world.insert_resource(HitStopState::new());
    world.insert_resource(Events::<HitStopEvent>::default());

    // 2. 模拟触发事件
    world.resource_mut::<Events<HitStopEvent>>().send(HitStopEvent {
        duration: 0.1,
        speed: 0.1,
    });

    // 3. 手动执行 handle 系统
    let mut schedule = Schedule::default();
    schedule.add_systems(handle_hit_stop_events);
    schedule.run(&mut world);

    {
        let time = world.get_resource::<Time<Virtual>>().unwrap();
        assert_eq!(time.relative_speed(), 0.1, "Speed must drop to 0.1");
    }

    // 4. 模拟时间步进：经过了 0.15s
    {
        let mut time_real = world.get_resource_mut::<Time<Real>>().unwrap();
        time_real.advance_by(Duration::from_millis(150));
    }

    // 5. 手动执行 update 计时系统
    let mut schedule_timer = Schedule::default();
    schedule_timer.add_systems(update_hit_stop_timer);
    schedule_timer.run(&mut world);

    {
        let time = world.get_resource::<Time<Virtual>>().unwrap();
        assert_eq!(time.relative_speed(), 1.0, "Speed must be restored to 1.0");
    }
}
