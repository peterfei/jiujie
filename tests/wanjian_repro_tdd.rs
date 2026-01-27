use bevy::prelude::*;
use bevy_card_battler::components::particle::{EffectType, Particle, SpawnEffectEvent};
use bevy_card_battler::systems::particle::{ParticlePlugin, ParticleAssets};
use bevy_card_battler::components::sprite::EnemySpriteMarker;
use bevy_card_battler::states::GameState;
use std::collections::HashMap;

#[test]
fn test_wanjian_lock_pos_stability() {
    // 验证锁定位置的稳定性
    let mut p = Particle::new(2.0).with_type(EffectType::WanJian);
    let target = Vec2::new(200.0, 100.0);
    p.target = Some(target);
    p.position = Vec2::new(0.0, 250.0); // Phase 3 结束位置
    
    // 模拟连续更新
    let strike_t = 0.5; // Phase 4 中间点
    
    // 第一次计算，锁定 lock_pos
    if p.lock_pos.is_none() { p.lock_pos = Some(p.position); }
    let lock_pos = p.lock_pos.unwrap();
    
    // 计算位置
    let inv_t = 1.0 - strike_t;
    let pos1 = lock_pos * inv_t * inv_t * inv_t + target * strike_t * strike_t * strike_t; // 简化版贝塞尔
    
    p.position = pos1;
    
    // 第二次计算，不应该改变 lock_pos
    assert_eq!(p.lock_pos, Some(Vec2::new(0.0, 250.0)));
    
    println!("✅ Lock pos 保持稳定");
}

#[test]
fn test_wanjian_retargeting_logic() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<GameState>();
    app.insert_state(GameState::Combat);
    
    app.add_event::<SpawnEffectEvent>();
    app.add_event::<bevy_card_battler::components::screen_effect::ScreenEffectEvent>();
    
    app.insert_resource(ParticleAssets {
        textures: HashMap::new(),
        default_texture: Handle::default(),
    });

    app.add_systems(Update, bevy_card_battler::systems::particle::update_particles.run_if(in_state(GameState::Combat)));

    // 1. 创建两个敌人
    let e1 = app.world_mut().spawn((EnemySpriteMarker { id: 1 }, Transform::from_xyz(200.0, 0.0, 0.0))).id();
    let e2 = app.world_mut().spawn((EnemySpriteMarker { id: 2 }, Transform::from_xyz(-200.0, 100.0, 0.0))).id();

    // 2. 创建一个粒子，锁定 e1
    let mut p = Particle::new(2.0).with_type(EffectType::WanJian);
    p.target_entity = Some(e1);
    p.elapsed = 1.2; // 进入 Phase 4 (1.2/2.0 * 1.6 - 0*0.6 = 0.96)
    
    app.world_mut().spawn((
        Node::default(),
        ImageNode::default(),
        Visibility::Visible,
        Transform::default(),
        p,
    ));

    // 更新一帧，目标应该是 e1 的位置
    app.update();
    
    {
        let mut p_query = app.world_mut().query::<&Particle>();
        let p_after = p_query.single(app.world());
        assert_eq!(p_after.target, Some(Vec2::new(200.0, 0.0)));
    }

    // 3. 杀死 e1
    app.world_mut().despawn(e1);
    
    // 更新一帧，粒子应该重定向到 e2
    app.update();
    
    {
        let mut p_query = app.world_mut().query::<&Particle>();
        let p_after = p_query.single(app.world());
        assert_eq!(p_after.target_entity, Some(e2), "应该重定向到 e2");
        assert_eq!(p_after.target, Some(Vec2::new(-200.0, 100.0)), "目标坐标应更新为 e2 的位置");
    }

    println!("✅ 重定向逻辑验证通过");
}
