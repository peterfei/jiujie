use bevy::prelude::*;
use bevy_card_battler::components::sprite::{CharacterSprite, AnimationState, PlayerSpriteMarker, PlayerAnimationConfig, PhysicalImpact, ActionType, Combatant3d, CharacterAnimationEvent, EnemySpriteMarker, BreathAnimation};
use bevy_card_battler::systems::sprite::{update_physical_impacts, trigger_hit_feedback, update_combatant_orientation, handle_animation_events};

#[test]
#[ignore]
fn test_cultivator_full_combo_integration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<CharacterAnimationEvent>();
    app.add_event::<bevy_card_battler::components::particle::SpawnEffectEvent>();
    app.add_event::<bevy_card_battler::components::ScreenEffectEvent>();

    // 1. 模拟真实修行者实体
    let player_entity = app.world_mut().spawn((
        Transform::from_xyz(-5.0, 0.8, 0.0),
        PlayerSpriteMarker,
        CharacterSprite::new(Handle::default(), Vec2::new(100.0, 100.0)),
        PlayerAnimationConfig {
            graph: Handle::default(),
            idle_node: AnimationNodeIndex::new(0),
            run_node: AnimationNodeIndex::new(1),
            kick_node: AnimationNodeIndex::new(2),
            strike_node: AnimationNodeIndex::new(3),
        },
        PhysicalImpact { home_position: Vec3::new(-5.0, 0.8, 0.0), ..default() },
        Combatant3d { facing_right: true, base_rotation: 0.0, model_offset: 0.0 },
        BreathAnimation::default(),
    )).id();

    // 2. 模拟真实敌人
    app.world_mut().spawn((
        Transform::from_xyz(2.0, 0.8, 0.0),
        EnemySpriteMarker { id: 1 },
        Combatant3d { facing_right: false, base_rotation: 0.0, model_offset: 0.0 },
    ));

    // 3. 加入可公开访问的系统
    app.add_systems(Update, (
        update_combatant_orientation, 
        trigger_hit_feedback, 
        handle_animation_events,
        update_physical_impacts,
    ).chain());
    
    app.insert_resource(Time::<Real>::default());

    println!("
--- [集成测试] 启动御剑术 (LinearRun) ---");
    app.world_mut().send_event(CharacterAnimationEvent {
        target: player_entity,
        animation: AnimationState::LinearRun,
    });

    // 运行 120 帧 (约 4 秒)，记录关键数据
    let mut stages = Vec::new();
    let mut anim_states = Vec::new();
    let mut z_offsets = Vec::new();

    for i in 0..120 {
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_millis(32));
        app.update();
        
        let impact = app.world().get::<PhysicalImpact>(player_entity).unwrap();
        let sprite = app.world().get::<CharacterSprite>(player_entity).unwrap();
        let transform = app.world().get::<Transform>(player_entity).unwrap();
        
        stages.push(impact.action_stage);
        anim_states.push(sprite.state);
        z_offsets.push(transform.translation.z);

        if i % 20 == 0 {
            println!("Frame {}: Stage={}, Anim={:?}, Pos={:?}", i, impact.action_stage, sprite.state, transform.translation);
        }
    }

    // --- 分析断言 ---
    
    // 1. 验证动画连贯性：跑动过程中动画不能切回 Idle
    let rush_anim_ok = anim_states.iter().take(20).all(|&s| s == AnimationState::Attack || s == AnimationState::LinearRun);
    println!("跑动动画连贯性: {}", if rush_anim_ok { "PASS" } else { "FAIL (动画过早切回 Idle)" });
    
    // 2. 验证 Z 轴动态：必须有明显的正负摆动 (大于 0.1)
    let max_z = z_offsets.iter().map(|z| z.abs()).fold(0.0f32, |a, b| a.max(b));
    println!("Z轴最大摆动幅度: {:.4}", max_z);
    
    // 3. 验证阶段流转：必须经历了 Stage 0, 1, 2, 3
    let reached_stage_3 = stages.contains(&3);
    println!("是否到达返回阶段 (Stage 3): {}", reached_stage_3);

    assert!(rush_anim_ok, "跑动动画不连贯，导致视觉僵硬");
    assert!(max_z > 0.1, "Z 轴摆动幅度不足，跑动感不明显");
    assert!(reached_stage_3, "状态机未能完成连斩流转");
}
