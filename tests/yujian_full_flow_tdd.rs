use bevy::prelude::*;
use bevy_card_battler::components::sprite::{CharacterSprite, AnimationState, PlayerSpriteMarker, PlayerAnimationConfig, PhysicalImpact, ActionType, Combatant3d};
use bevy_card_battler::systems::sprite::{sync_player_skeletal_animation, update_physical_impacts};

#[test]
fn test_yujian_full_flow_diagnostics() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<bevy_card_battler::components::particle::SpawnEffectEvent>();
    app.add_event::<bevy_card_battler::components::ScreenEffectEvent>();

    // 1. 定义期望索引 (根据用户反馈: 0:Wait, 1:Kick, 2:Run, 3:Strike)
    let idle_node = AnimationNodeIndex::new(0);
    let kick_node = AnimationNodeIndex::new(1);
    let run_node = AnimationNodeIndex::new(2);
    let strike_node = AnimationNodeIndex::new(3);

    // 2. 创建玩家
    let player_entity = app.world_mut().spawn((
        PlayerSpriteMarker,
        CharacterSprite::new(Handle::default(), Vec2::ONE),
        PlayerAnimationConfig {
            graph: Handle::default(),
            idle_node,
            kick_node,
            run_node,
            strike_node,
        },
        PhysicalImpact { 
            home_position: Vec3::new(-3.5, 0.0, 0.0), 
            target_offset_dist: 5.0,
            action_type: ActionType::Dash,
            action_timer: 1.0,
            action_direction: 1.0,
            ..default()
        },
        Combatant3d { facing_right: true, base_rotation: 0.0, model_offset: 0.0 },
        bevy_card_battler::components::sprite::BreathAnimation { timer: 0.0, frequency: 1.0, amplitude: 0.0 },
    )).id();

    let anim_player_entity = app.world_mut().spawn((
        AnimationPlayer::default(),
        AnimationGraphHandle(Handle::default()),
    )).set_parent(player_entity).id();

    app.insert_resource(Time::<Real>::default());
    app.update(); // 初始化

    // 3. 模拟打出“御剑术”
    {
        let mut sprite = app.world_mut().get_mut::<CharacterSprite>(player_entity).unwrap();
        sprite.state = AnimationState::WolfAttack;
    }

    app.add_systems(Update, (update_physical_impacts, sync_player_skeletal_animation));

    // 4. 连续跟踪 10 帧，模拟时间流逝
    println!("\n--- 御剑术冲刺阶段诊断 ---");
    for frame in 0..10 {
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_millis(16)); // 模拟 60fps
        }
        app.update();
        
        let player = app.world().get::<AnimationPlayer>(anim_player_entity).unwrap();
        let impact = app.world().get::<PhysicalImpact>(player_entity).unwrap();
        let sprite = app.world().get::<CharacterSprite>(player_entity).unwrap();
        
        println!("帧 {}: 状态={:?}, 播放节点={:?}, 偏移X={:.2}, 速度X={:.2}", 
            frame, sprite.state, player.playing_animations().next().map(|(id, _)| id), 
            impact.current_offset.x, impact.offset_velocity.x);
    }

    // 5. 最终验证：是否能在冲刺后正确切换到挥砍
    let sprite_final = app.world().get::<CharacterSprite>(player_entity).unwrap();
    println!("最终状态: {:?}", sprite_final.state);
}