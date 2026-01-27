use bevy::prelude::*;
use bevy_card_battler::components::particle::{EffectType, Particle, SpawnEffectEvent};
use bevy_card_battler::states::GameState;
use bevy_card_battler::systems::particle::{ParticlePlugin, ParticleAssets};
use std::collections::HashMap;

#[test]
fn test_wanjian_only_attacks_single_target_currently() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<GameState>();
    app.insert_state(GameState::Combat);
    
    // 手动添加 ParticlePlugin 中需要的资源，跳过自动加载
    app.add_event::<SpawnEffectEvent>();
    app.insert_resource(ParticleAssets {
        textures: HashMap::new(),
        default_texture: Handle::default(),
    });
    
    // 只添加更新系统，避免 Startup 系统崩溃
    app.add_systems(
        Update,
        (
            bevy_card_battler::systems::particle::handle_effect_events,
            bevy_card_battler::systems::particle::update_emitters,
            bevy_card_battler::systems::particle::update_particles,
        ).run_if(in_state(GameState::Combat))
    );

    // 1. 模拟两个敌人的位置
    let enemy1_pos = Vec2::new(200.0, 0.0);
    let enemy2_pos = Vec2::new(300.0, 100.0);

    // 2. 发送万剑归宗事件，目前只能传入一个 target
    app.world_mut().send_event(
        SpawnEffectEvent::new(EffectType::WanJian, Vec3::ZERO)
            .burst(10)
            .with_target(enemy1_pos)
    );

    // 更新一帧以处理事件并生成粒子
    app.update();

    // 3. 检查所有生成的粒子
    let mut query = app.world_mut().query::<&Particle>();
    let particles: Vec<&Particle> = query.iter(app.world()).collect();
    
    assert_eq!(particles.len(), 10);
    
    // 验证：所有粒子的 target 都是 enemy1_pos
    for p in particles {
        assert_eq!(p.target, Some(enemy1_pos), "目前粒子只能锁定同一个目标");
        assert_ne!(p.target, Some(enemy2_pos), "目前粒子无法感知第二个目标");
    }

    println!("✅ 确认现状：万剑归宗目前确实只能锁定单一目标");
}
