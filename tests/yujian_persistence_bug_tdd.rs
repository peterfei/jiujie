use bevy::prelude::*;
use bevy_card_battler::components::sprite::{CharacterSprite, AnimationState, PlayerSpriteMarker, PlayerAnimationConfig, PhysicalImpact, ActionType, Combatant3d, CharacterAnimationEvent, BreathAnimation};
use bevy_card_battler::systems::sprite::{update_physical_impacts, trigger_hit_feedback};

#[test]
fn test_yujian_persistence_bug_repro() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<CharacterAnimationEvent>();
    app.add_event::<bevy_card_battler::components::particle::SpawnEffectEvent>();
    app.add_event::<bevy_card_battler::components::ScreenEffectEvent>();

    let player_entity = app.world_mut().spawn((
        Transform::from_xyz(-3.5, 0.8, 0.0),
        PlayerSpriteMarker,
        CharacterSprite::new(Handle::default(), Vec2::ONE),
        PlayerAnimationConfig {
            graph: Handle::default(),
            idle_node: AnimationNodeIndex::new(0),
            run_node: AnimationNodeIndex::new(1),
            kick_node: AnimationNodeIndex::new(2),
            strike_node: AnimationNodeIndex::new(3),
        },
        PhysicalImpact {
            home_position: Vec3::new(-3.5, 0.8, 0.0),
            target_offset_dist: 6.0,
            action_direction: 1.0, 
            ..default()
        },
        Combatant3d { facing_right: true, base_rotation: 0.0, model_offset: 0.0 },
        BreathAnimation { timer: 0.0, frequency: 1.0, amplitude: 0.0 },
    )).id();

    app.add_systems(Update, (trigger_hit_feedback, update_physical_impacts).chain());
    app.insert_resource(Time::<Real>::default());

    // --- 第一轮攻击 ---
    println!("\n--- [TDD] 执行第一轮御剑术 ---");
    app.world_mut().send_event(CharacterAnimationEvent {
        target: player_entity,
        animation: AnimationState::LinearRun,
    });

    for _ in 0..30 {
        // [关键修复] 直接手动设置 delta_time，确保 MinimalPlugins 下系统能读取到
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_millis(32));
        app.update();
    }
    
    let impact_1 = app.world().get::<PhysicalImpact>(player_entity).unwrap();
    println!("第一轮结束: X偏移={:.2}, ActionStage={}", impact_1.current_offset.x, impact_1.action_stage);
    assert!(impact_1.current_offset.x.abs() > 0.3, "第一轮应该产生位移");

    // 模拟动作结束回到原点并产生状态残留 (action_stage=1)
    {
        let mut impact = app.world_mut().get_mut::<PhysicalImpact>(player_entity).unwrap();
        impact.action_timer = 0.0;
        impact.current_offset = Vec3::ZERO;
        impact.offset_velocity = Vec3::ZERO;
        impact.action_stage = 1; 
    }
    app.update();

    // --- 第二轮攻击 ---
    println!("--- [TDD] 执行第二轮御剑术 ---");
    app.world_mut().send_event(CharacterAnimationEvent {
        target: player_entity,
        animation: AnimationState::LinearRun,
    });

    app.update(); // 触发 trigger_hit_feedback，期望重置 action_stage=0
    
    for _ in 0..30 {
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_millis(32));
        app.update();
    }

    let impact_2 = app.world().get::<PhysicalImpact>(player_entity).unwrap();
    println!("第二轮结束: X偏移={:.2}, ActionStage={}", impact_2.current_offset.x, impact_2.action_stage);

    // 核心断言：第二轮必须要有位移，证明 action_stage 被成功重置
    assert!(impact_2.current_offset.x.abs() > 0.3, "BUG：第二轮没有位移！确认 action_stage 重置逻辑失效。");
    println!("--- [TDD] 红绿测试通过：御剑术持久性 Bug 已修复 ---");
}