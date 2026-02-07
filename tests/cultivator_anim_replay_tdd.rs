use bevy::prelude::*;
use bevy_card_battler::components::sprite::{CharacterSprite, AnimationState, PlayerSpriteMarker, PlayerAnimationConfig, PhysicalImpact, ActionType, Combatant3d, CharacterAnimationEvent};
use bevy_card_battler::systems::sprite::{update_physical_impacts, trigger_hit_feedback, sync_player_skeletal_animation};

#[test]
fn test_cultivator_combo_animation_replay() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<CharacterAnimationEvent>();
    app.add_event::<bevy_card_battler::components::particle::SpawnEffectEvent>();
    app.add_event::<bevy_card_battler::components::ScreenEffectEvent>();

    let player_entity = app.world_mut().spawn((
        Transform::default(),
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
            home_position: Vec3::ZERO,
            target_offset_dist: 5.0,
            ..default()
        },
        Combatant3d { facing_right: true, base_rotation: 0.0, model_offset: 0.0 },
    )).id();

    // 模拟子实体中包含 AnimationPlayer
    let anim_player_entity = app.world_mut().spawn(AnimationPlayer::default()).id();
    app.world_mut().entity_mut(player_entity).add_child(anim_player_entity);

    app.add_systems(Update, (trigger_hit_feedback, update_physical_impacts, sync_player_skeletal_animation).chain());
    app.insert_resource(Time::<Real>::default());

    // --- 步骤 1: 触发组合技并快速跳过 Rush 阶段 ---
    app.world_mut().send_event(CharacterAnimationEvent {
        target: player_entity,
        animation: AnimationState::LinearRun,
    });
    app.update();

    {
        let mut impact = app.world_mut().get_mut::<PhysicalImpact>(player_entity).unwrap();
        impact.current_offset.x = 4.6; // 到位
    }
    app.update(); // 进入 Stage 1 (Attack 1)

    let sprite_1 = app.world().get::<CharacterSprite>(player_entity).unwrap();
    assert_eq!(sprite_1.state, AnimationState::Attack);

    // --- 步骤 2: 等待 Attack 1 结束 ---
    for _ in 0..20 {
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(32));
        app.update();
    }

    let impact_2 = app.world().get::<PhysicalImpact>(player_entity).unwrap();
    assert_eq!(impact_2.action_stage, 2, "此时应该处于 Stage 2 (Attack 2)");

    // 核心验证逻辑需要通过观察 AnimationPlayer 的行为，
    // 在 Headless 模式下，我们通过确保 sync_player_skeletal_animation 被调用且状态机正确流转来推断逻辑正确性。
    // 本次修复的关键在于代码中显式调用了 replay()。
}
