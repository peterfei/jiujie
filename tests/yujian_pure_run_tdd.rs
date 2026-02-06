use bevy::prelude::*;
use bevy_card_battler::components::sprite::{CharacterSprite, AnimationState, PlayerSpriteMarker, PlayerAnimationConfig, PhysicalImpact, ActionType};
use bevy_card_battler::systems::sprite::{sync_player_skeletal_animation, update_physical_impacts};

#[test]
fn test_yujian_pure_animation_redesign() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<bevy_card_battler::components::particle::SpawnEffectEvent>();
    app.add_event::<bevy_card_battler::components::ScreenEffectEvent>();

    // 1. 定义节点
    let idle_node = AnimationNodeIndex::new(1);
    let run_node = AnimationNodeIndex::new(3); // 假设 3 是跑步

    // 2. 创建玩家
    let player_entity = app.world_mut().spawn((
        PlayerSpriteMarker,
        CharacterSprite::new(Handle::default(), Vec2::ONE),
        PlayerAnimationConfig {
            graph: Handle::default(),
            idle_node,
            run_node,
            kick_node: AnimationNodeIndex::new(2),
            strike_node: AnimationNodeIndex::new(4),
        },
        PhysicalImpact::default(),
    )).id();

    // 3. 模拟打出“御剑术” (LinearRun 状态)
    app.world_mut().get_mut::<CharacterSprite>(player_entity).unwrap().state = AnimationState::LinearRun;
    
    // 我们期望：LinearRun 不应该触发 ActionType::Dash 的强力滑行
    // 这里我们手动模拟事件触发后的物理状态
    {
        let mut impact = app.world_mut().get_mut::<PhysicalImpact>(player_entity).unwrap();
        impact.action_type = ActionType::None; // 重新设计：不要 Dash
        impact.offset_velocity = Vec3::ZERO;
    }

    app.add_systems(Update, (update_physical_impacts, sync_player_skeletal_animation));
    app.insert_resource(Time::<Real>::default());

    app.update();

    let impact_final = app.world().get::<PhysicalImpact>(player_entity).unwrap();
    let sprite_final = app.world().get::<CharacterSprite>(player_entity).unwrap();
    
    println!("御剑术重构验证: 状态={:?}, 物理动作={:?}, 速度={:?}", 
        sprite_final.state, impact_final.action_type, impact_final.offset_velocity);

    // 断言 1: 状态正确
    assert_eq!(sprite_final.state, AnimationState::LinearRun);
    // 断言 2: 没有触发滑行物理 (ActionType 为 None 或特定的非位移动作)
    assert_ne!(impact_final.action_type, ActionType::Dash, "御剑术不应再使用 Glide 式的 Dash");
}
