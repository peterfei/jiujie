use bevy::prelude::*;
use bevy_card_battler::components::sprite::{CharacterSprite, AnimationState, PlayerSpriteMarker, PlayerAnimationConfig, PhysicalImpact, ActionType, Combatant3d, CharacterAnimationEvent, EnemySpriteMarker, BreathAnimation};
use bevy_card_battler::systems::sprite::{update_physical_impacts, trigger_hit_feedback, update_combatant_orientation, handle_animation_events};

#[test]
#[ignore]
fn test_cultivator_overshoot_and_sway_ratio() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<CharacterAnimationEvent>();
    app.add_event::<bevy_card_battler::components::particle::SpawnEffectEvent>();
    app.add_event::<bevy_card_battler::components::ScreenEffectEvent>();

    // 1. 玩家在 (-10, 0, 0)
    let player_entity = app.world_mut().spawn((
        Transform::from_xyz(-10.0, 0.8, 0.0),
        PlayerSpriteMarker,
        CharacterSprite::new(Handle::default(), Vec2::ONE),
        PlayerAnimationConfig {
            graph: Handle::default(),
            idle_node: AnimationNodeIndex::new(0),
            run_node: AnimationNodeIndex::new(1),
            kick_node: AnimationNodeIndex::new(2),
            strike_node: AnimationNodeIndex::new(3),
        },
        PhysicalImpact { home_position: Vec3::new(-10.0, 0.8, 0.0), ..default() },
        Combatant3d { facing_right: true, base_rotation: 0.0, model_offset: 0.0 },
        BreathAnimation::default(),
    )).id();

    // 2. 怪物在 (0, 0, 0) -> 正前方 10 米
    app.world_mut().spawn((
        Transform::from_xyz(0.0, 0.8, 0.0),
        EnemySpriteMarker { id: 1 },
        Combatant3d { facing_right: false, base_rotation: 0.0, model_offset: 0.0 },
    ));

    app.add_systems(Update, (
        update_combatant_orientation, 
        trigger_hit_feedback, 
        handle_animation_events,
        update_physical_impacts,
    ).chain());
    
    app.insert_resource(Time::<Real>::default());

    println!("
--- [TDD] 启动组合技 ---");
    app.world_mut().send_event(CharacterAnimationEvent {
        target: player_entity,
        animation: AnimationState::LinearRun,
    });

    // 预热
    let mut time = app.world_mut().resource_mut::<Time>();
    time.advance_by(std::time::Duration::from_millis(16));
    app.update();

    // --- Phase 1: 跑动阶段监控 (验证侧向摆动比例) ---
    let mut max_sway = 0.0;
    let mut traveled_dist = 0.0;
    let start_pos = app.world().get::<Transform>(player_entity).unwrap().translation;

    // 模拟跑动 30 帧
    for _ in 0..30 {
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_millis(32));
        app.update();
        
        let transform = app.world().get::<Transform>(player_entity).unwrap();
        // 因为是从 (-10,0,0) 跑到 (0,0,0)，侧向就是 Z 轴
        if transform.translation.z.abs() > max_sway {
            max_sway = transform.translation.z.abs();
        }
    }
    let mid_pos = app.world().get::<Transform>(player_entity).unwrap().translation;
    traveled_dist = (mid_pos.x - start_pos.x).abs();

    println!("跑动距离: {:.2}, 最大侧向摆动: {:.4}", traveled_dist, max_sway);
    
    // 断言 1: 侧向摆动幅度必须足够大 (例如 > 0.5)，否则在大尺度跑动下肉眼不可见
    // 当前实现约 0.28，相对于 10米的距离太小
    if max_sway < 0.5 {
        println!("FAIL: 侧向摆动幅度过小，导致视觉上像直线。");
    }

    // --- Phase 2: 攻击过冲监控 ---
    // 手动瞬移到怪面前触发攻击
    if let Some(mut impact) = app.world_mut().get_mut::<PhysicalImpact>(player_entity) {
        impact.current_offset.x = 9.7; // 距离目标 (10.0) 剩 0.3
    }
    app.update(); // 触发切换到 Stage 1 (Attack)

    // 模拟攻击过程中的惯性位移
    let mut max_x = 0.0;
    for _ in 0..20 {
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_millis(32));
        app.update();
        
        let transform = app.world().get::<Transform>(player_entity).unwrap();
        // 目标是 0.0 (home -10 + 10)
        // 如果 x > 0.0，说明穿过了怪物
        if transform.translation.x > max_x {
            max_x = transform.translation.x;
        }
    }

    println!("攻击阶段最远位置 X: {:.2} (怪物在 0.0)", max_x);

    // 断言 2: 不应该穿过怪物 (允许极其微小的误差，但不能跑得比怪还远)
    // 如果 max_x > 0.5，说明明显穿模到了怪身后
    assert!(max_x < 0.5, "BUG REPRODUCED: 攻击惯性导致角色穿过怪物跑到了身后！");
    
    // 强制断言摆动幅度
    assert!(max_sway > 0.5, "BUG REPRODUCED: 跑动轨迹过于平直，缺乏动感。");
}
