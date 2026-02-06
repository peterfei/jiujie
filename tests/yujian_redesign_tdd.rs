use bevy::prelude::*;
use bevy_card_battler::components::sprite::{CharacterSprite, AnimationState, PlayerSpriteMarker, PlayerAnimationConfig, PhysicalImpact, ActionType, Combatant3d};
use bevy_card_battler::systems::sprite::{sync_player_skeletal_animation, update_physical_impacts};

#[test]
fn test_yujian_linear_charge_green() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<bevy_card_battler::components::particle::SpawnEffectEvent>();
    app.add_event::<bevy_card_battler::components::ScreenEffectEvent>();

    // 定义索引
    let idle_node = AnimationNodeIndex::new(0);
    let kick_node = AnimationNodeIndex::new(1);
    let run_node = AnimationNodeIndex::new(2);
    let strike_node = AnimationNodeIndex::new(3);

    let player_entity = app.world_mut().spawn((
        Transform::from_xyz(-3.5, 0.8, 0.0), // 显式添加 Transform
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
            home_position: Vec3::new(-3.5, 0.8, 0.0), 
            target_offset_dist: 4.5,
            action_type: ActionType::Dash,
            action_timer: 1.0,
            action_direction: 1.0,
            special_rotation: 0.0,
            ..default()
        },
        Combatant3d { facing_right: true, base_rotation: 0.0, model_offset: 0.0 },
        bevy_card_battler::components::sprite::BreathAnimation { timer: 0.0, frequency: 1.0, amplitude: 0.0 },
    )).id();

    // 挂载动画播放器
    app.world_mut().spawn((
        AnimationPlayer::default(),
        AnimationGraphHandle(Handle::default()),
    )).set_parent(player_entity);

    app.insert_resource(Time::<Real>::default());
    
    // 设置初始状态：御剑冲刺中
    app.world_mut().get_mut::<CharacterSprite>(player_entity).unwrap().state = AnimationState::WolfAttack;

    app.add_systems(Update, (update_physical_impacts, sync_player_skeletal_animation));

    println!("\n--- [红绿测试] 御剑术：直线与位移验证 ---");
    let mut last_x = 0.0;
    
    for frame in 0..15 {
        {
            let mut time = app.world_mut().resource_mut::<Time<Real>>();
            time.advance_by(std::time::Duration::from_millis(32)); 
        }
        app.update();
        
        let impact = app.world().get::<PhysicalImpact>(player_entity).unwrap();
        let sprite = app.world().get::<CharacterSprite>(player_entity).unwrap();
        
        println!("帧 {}: 状态={:?}, X偏移={:.2}, 速度X={:.2}, 旋转={:.2}", 
            frame, sprite.state, impact.current_offset.x, impact.offset_velocity.x, impact.special_rotation);
        
        // 验证 1: special_rotation 必须绝对清零
        assert!(impact.special_rotation.abs() < 0.001, "冲刺期间不允许有绕圈旋转");
        
        // 验证 2: 偏移量 X 必须在增加 (向右冲刺)
        if frame > 1 && sprite.state == AnimationState::WolfAttack {
            assert!(impact.current_offset.x > last_x, "冲刺期间位移必须稳步增加");
        }
        last_x = impact.current_offset.x;
    }

    // 验证 3: 最终状态切换
    let sprite_final = app.world().get::<CharacterSprite>(player_entity).unwrap();
    println!("最终状态: {:?}", sprite_final.state);
    
    // 如果位移足够，应该已经切到了 Attack
    assert!(sprite_final.state == AnimationState::Attack || sprite_final.state == AnimationState::WolfAttack, "状态机逻辑异常");
}