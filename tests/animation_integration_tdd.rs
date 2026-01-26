use bevy::prelude::*;
use bevy_card_battler::components::sprite::{CharacterAnimationEvent, AnimationState, PhysicalImpact};
use bevy_card_battler::systems::sprite::SpritePlugin;
use bevy_card_battler::states::GameState;

use bevy::state::app::StatesPlugin;

#[test]
fn test_animation_event_triggers_physical_impact() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(HierarchyPlugin)
       .add_plugins(AssetPlugin::default())
       .add_plugins(StatesPlugin) // 必须添加
       .init_state::<GameState>();
    
    // 注册插件
    app.add_plugins(SpritePlugin);
    
    // 设置为战斗状态，否则系统不运行
    app.world_mut().insert_resource(NextState::Pending(GameState::Combat));
    app.update(); // 切换状态

    // 创建测试实体
    let entity = app.world_mut().spawn((
        Transform::default(),
        PhysicalImpact::default(),
    )).id();

    // 发送攻击事件
    app.world_mut().resource_mut::<Events<CharacterAnimationEvent>>().send(CharacterAnimationEvent {
        target: entity,
        animation: AnimationState::Attack,
    });

    // 运行一帧
    app.update();

    // 验证物理反馈
    let impact = app.world().get::<PhysicalImpact>(entity).unwrap();
    println!("Impact tilt_velocity: {}", impact.tilt_velocity);
    assert!(impact.tilt_velocity != 0.0, "发送 CharacterAnimationEvent 后，PhysicalImpact 的速度应该被激活");
}
