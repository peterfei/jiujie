use bevy::prelude::*;
use bevy_card_battler::components::sprite::{CharacterSprite, AnimationState, PlayerSpriteMarker, PlayerAnimationConfig, PhysicalImpact, ActionType, Combatant3d, CharacterAnimationEvent, EnemySpriteMarker};
use bevy_card_battler::systems::sprite::{update_physical_impacts, trigger_hit_feedback, update_combatant_orientation};

#[test]
fn test_cultivator_diagonal_rush_pathing() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<CharacterAnimationEvent>();
    app.add_event::<bevy_card_battler::components::particle::SpawnEffectEvent>();
    app.add_event::<bevy_card_battler::components::ScreenEffectEvent>();

    // 1. 玩家在 (-5, 0, 0)
    let player_entity = app.world_mut().spawn((
        Transform::from_xyz(-5.0, 0.8, 0.0),
        PlayerSpriteMarker,
        CharacterSprite::new(Handle::default(), Vec2::ONE),
        PlayerAnimationConfig {
            graph: Handle::default(),
            idle_node: AnimationNodeIndex::new(0),
            run_node: AnimationNodeIndex::new(1),
            kick_node: AnimationNodeIndex::new(2),
            strike_node: AnimationNodeIndex::new(3),
        },
        PhysicalImpact { home_position: Vec3::new(-5.0, 0.8, 0.0), ..default() },
        Combatant3d { facing_right: true, base_rotation: 0.0, model_offset: 0.0 },
    )).id();

    // 2. 怪物在 (0, 0, 5) -> 斜对角
    app.world_mut().spawn((
        Transform::from_xyz(0.0, 0.8, 5.0),
        EnemySpriteMarker { id: 1 },
        Combatant3d { facing_right: false, base_rotation: 0.0, model_offset: 0.0 },
    ));

    app.add_systems(Update, (
        update_combatant_orientation, 
        trigger_hit_feedback, 
        update_physical_impacts
    ).chain());
    
    app.insert_resource(Time::<Real>::default());

    // --- 启动组合技 ---
    println!("
--- [TDD] 测试斜对角突袭路径 ---");
    app.world_mut().send_event(CharacterAnimationEvent {
        target: player_entity,
        animation: AnimationState::LinearRun,
    });

    // 运行一段时间
    for i in 0..10 {
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_millis(32));
        app.update();
        
        let transform = app.world().get::<Transform>(player_entity).unwrap();
        println!("Frame {}: Translation={:?}", i, transform.translation);
    }

    let final_transform = app.world().get::<Transform>(player_entity).unwrap();
    
    // 核心断言：Z 轴应该产生位移！
    // 如果 Z 轴一直是 0，说明是在沿 X 轴漂移，而不是冲向怪物。
    assert!(final_transform.translation.z.abs() > 0.1, "BUG: 突袭路径没有朝向怪物（Z轴无位移），导致漂移感！");
}
