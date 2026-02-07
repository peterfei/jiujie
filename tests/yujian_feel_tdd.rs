use bevy::prelude::*;
use bevy_card_battler::components::sprite::{CharacterSprite, AnimationState, PlayerSpriteMarker, PlayerAnimationConfig, PhysicalImpact, ActionType, Combatant3d, CharacterAnimationEvent, BreathAnimation};
use bevy_card_battler::systems::sprite::{update_physical_impacts, trigger_hit_feedback};

#[test]
fn test_yujian_run_feel_improvement() {
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

    // --- 启动御剑术 (LinearRun) ---
    println!("
--- [TDD] 启动御剑术冲刺 ---");
    app.world_mut().send_event(CharacterAnimationEvent {
        target: player_entity,
        animation: AnimationState::LinearRun,
    });

    // 预热一帧以应用状态
    app.update();

    let mut z_offsets = Vec::new();

    // 模拟跑步过程 (约 0.5 秒)
    for i in 0..15 {
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_millis(32));
        app.update();

        let impact = app.world().get::<PhysicalImpact>(player_entity).unwrap();
        // 计算实际渲染位置的 Z 轴偏移 (相对于 home_position)
        let transform = app.world().get::<Transform>(player_entity).unwrap();
        let current_z_offset = transform.translation.z - impact.home_position.z;
        z_offsets.push(current_z_offset);
        
        println!("Frame {}: ActionTimer={:.2}, Z-Offset={:.4}", i, impact.action_timer, current_z_offset);
    }

    // --- 断言分析 ---
    // 1. 验证是否产生了 Z 轴位移 (左右摆动)
    let max_z = z_offsets.iter().map(|z| z.abs()).fold(0.0f32, |a, b| a.max(b));
    let has_z_movement = max_z > 0.001;
    
    if !has_z_movement {
        println!("FAILURE: 跑步过程 Z 轴完全静止，缺乏动感。");
    } else {
        println!("SUCCESS: 检测到 Z 轴动态摆动，最大幅度: {:.4}", max_z);
    }

    // 强制断言：我们希望这里失败，直到我们实现了该功能
    assert!(has_z_movement, "御剑术冲刺应该包含 Z 轴的自然摆动，以模拟跑步时的重心转移");
}
