use bevy::prelude::*;
use bevy_card_battler::components::sprite::{CharacterSprite, AnimationState, CharacterAnimationEvent, CharacterAssets};
use bevy_card_battler::systems::sprite::handle_animation_events;

#[test]
fn test_linear_run_state_persistence() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<CharacterAnimationEvent>();
    
    // 模拟角色资源
    app.insert_resource(CharacterAssets::default());

    let player_entity = app.world_mut().spawn((
        CharacterSprite::new(Handle::default(), Vec2::ONE),
    )).id();

    app.add_systems(Update, handle_animation_events);

    // --- 动作：发送 LinearRun 事件 ---
    app.world_mut().send_event(CharacterAnimationEvent {
        target: player_entity,
        animation: AnimationState::LinearRun,
    });

    // 运行一帧处理事件
    app.update();

    let sprite = app.world().get::<CharacterSprite>(player_entity).unwrap();
    
    // 核心断言：状态必须维持为 LinearRun
    println!("事件处理后动画状态: {:?}", sprite.state);
    assert_eq!(sprite.state, AnimationState::LinearRun, "BUG: LinearRun 状态被改写，导致骨骼动画无法正确匹配！");
    assert!(sprite.looping, "跑动动画必须是循环的");
}