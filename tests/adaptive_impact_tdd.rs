use bevy::prelude::*;
use bevy::ecs::system::RunSystemOnce;
use bevy_card_battler::components::screen_effect::{CameraShake, ScreenEffectEvent};
use bevy_card_battler::systems::screen_effect::handle_screen_effects;

#[test]
fn test_impact_feedback_directional_offset() {
    let mut app = App::new();
    
    // 1. 设置测试相机
    let camera_id = app.world_mut().spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 10.0),
    )).id();
    
    app.add_event::<ScreenEffectEvent>();

    // 2. 发送一个从右侧而来的冲击力事件
    // 冲击力方向为 (-1.0, 0.0)，即向左冲击
    let impulse = Vec2::new(-5.0, 0.0);
    app.world_mut().resource_mut::<Events<ScreenEffectEvent>>().send(
        ScreenEffectEvent::Impact { impulse, duration: 0.5 }
    );

    // 3. 运行处理系统
    let _ = app.world_mut().run_system_once(handle_screen_effects);

    // 4. 验证：相机是否挂载了包含冲击力信息的 CameraShake
    let shake = app.world().get::<CameraShake>(camera_id).expect("相机应挂载 CameraShake");
    
    // 注意：这里需要我们在 CameraShake 结构体中新增字段来存储冲量
    // 预期：在接下来的 update 系统中，相机的偏移量应该偏向左侧 (-x)
}
