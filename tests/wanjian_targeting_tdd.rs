use bevy::prelude::*;
use bevy_card_battler::components::particle::{EffectType, Particle};
use bevy_card_battler::components::sprite::EnemySpriteMarker;
use bevy_card_battler::systems::vfx_orchestrator::{update_vfx_logic};

#[test]
fn test_wanjian_3d_hit_accuracy() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<bevy_card_battler::components::screen_effect::ScreenEffectEvent>();
    app.add_event::<bevy_card_battler::components::particle::SpawnEffectEvent>();
    
    // 1. 生成远处的目标怪 (x=20.0)
    let target_pos_3d = Vec3::new(20.0, 0.0, 0.0);
    app.world_mut().spawn((
        Transform::from_translation(target_pos_3d),
        EnemySpriteMarker { id: 1 },
    ));

    // 2. 生成万剑粒子，设为即将命中的时间点
    let mut p = EffectType::WanJian.config().spawn_particle(Vec3::ZERO, EffectType::WanJian);
    p.target_entity = Some(Entity::from_raw(0)); // 故意留空，触发智能寻敌
    p.elapsed = 1.9; // 几乎满寿命 (lifetime=2.0)
    p.lifetime = 2.0;
    
    let sword = app.world_mut().spawn((
        p,
        Transform::default(),
        Visibility::Visible,
    )).id();

    // 3. 运行逻辑
    app.add_systems(Update, update_vfx_logic);
    app.update();

    // 4. 获取 3D Transform 结果
    let final_transform = app.world().get::<Transform>(sword).unwrap();
    
    // --- TDD 断言 ---
    // 目标在 20.0，如果比例正确，最终 Transform.x 应该接近 20.0
    // 目前代码由于比例问题，可能会输出 0.2
    let dist = final_transform.translation.x - 20.0;
    assert!(dist.abs() < 2.0, "Wanjian missed the target! Expected near 20.0, got {}", final_transform.translation.x);
}
