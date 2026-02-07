use bevy::prelude::*;
use bevy_card_battler::components::sprite::{CharacterSprite, AnimationState, PlayerSpriteMarker, PlayerAnimationConfig, PhysicalImpact, ActionType, Combatant3d, CharacterAnimationEvent};
use bevy_card_battler::systems::sprite::{update_physical_impacts, trigger_hit_feedback};

#[test]
#[ignore]
fn test_cultivator_combo_overshoot_check() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<CharacterAnimationEvent>();
    app.add_event::<bevy_card_battler::components::particle::SpawnEffectEvent>();
    app.add_event::<bevy_card_battler::components::ScreenEffectEvent>();

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
        // 假设目标在正前方 5 米
        PhysicalImpact { 
            home_position: Vec3::new(-5.0, 0.8, 0.0), 
            target_vector: Vec3::X,
            target_offset_dist: 5.0,
            ..default() 
        },
        Combatant3d { facing_right: true, base_rotation: 0.0, model_offset: 0.0 },
    )).id();

    app.add_systems(Update, (trigger_hit_feedback, update_physical_impacts).chain());
    app.insert_resource(Time::<Real>::default());

    // --- 模拟进入连斩阶段 (Stage 1) ---
    {
        let mut impact = app.world_mut().get_mut::<PhysicalImpact>(player_entity).unwrap();
        impact.action_type = ActionType::CultivatorCombo;
        impact.action_stage = 1;
        impact.action_timer = 0.6;
        impact.current_offset = Vec3::new(4.2, 0.0, 0.0); // 已经在怪面前
    }

    let start_offset_x = app.world().get::<PhysicalImpact>(player_entity).unwrap().current_offset.x;

    // 运行连斩阶段 (0.6s)
    for _ in 0..20 {
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_millis(32));
        app.update();
    }

    let final_impact = app.world().get::<PhysicalImpact>(player_entity).unwrap();
    let move_delta = final_impact.current_offset.x - start_offset_x;

    println!("连斩阶段产生的位移: {:.2}", move_delta);

    // 断言：位移不能过大。如果飞出屏幕，位移通常会超过 3-4 米
    // 在怪面前的微调位移应该控制在 1 米以内
    assert!(move_delta.abs() < 1.5, "BUG: 连斩阶段位移过大 ({:.2})，导致角色飞出屏幕！", move_delta);
}
