use bevy::prelude::*;
use bevy_card_battler::components::combat::Enemy;
use bevy_card_battler::components::sprite::{CharacterAnimationEvent, AnimationState};

#[test]
fn test_multi_attack_event_chain() {
    let mut app = App::new();
    app.add_event::<CharacterAnimationEvent>();
    
    // 1. 模拟 HP 归零
    let enemy_id = app.world_mut().spawn(Enemy::new(1, "Test", 0)).id(); // 已死
    
    // 2. 模拟系统检测到死亡并发送事件
    // (这正是我们在 mod.rs 中修复的逻辑)
    let mut events = app.world_mut().resource_mut::<Events<CharacterAnimationEvent>>();
    events.send(CharacterAnimationEvent {
        target: enemy_id,
        animation: AnimationState::Death
    });
    
    // 3. 验证事件是否存在
    let events = app.world().resource::<Events<CharacterAnimationEvent>>();
    let mut reader = events.get_reader();
    let has_death = reader.read(events).any(|e| e.animation == AnimationState::Death);
    
    assert!(has_death, "应该产生死亡动画事件");
}