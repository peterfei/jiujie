use bevy::prelude::*;
use bevy_card_battler::components::particle::{EffectType, Particle};
use bevy_card_battler::components::sprite::EnemySpriteMarker;
use bevy_card_battler::systems::vfx_orchestrator::update_wanjian_target;

#[test]
fn test_wanjian_smart_retargeting_green() {
    let mut app = App::new();
    
    // 1. 生成两个实体
    let survivor_pos = Vec3::new(-88.0, 0.0, 0.0);
    let survivor_entity = app.world_mut().spawn((
        Transform::from_translation(survivor_pos),
        EnemySpriteMarker { id: 10 },
    )).id();

    // 2. 准备万剑粒子
    let mut p = EffectType::WanJian.config().spawn_particle(Vec3::ZERO, EffectType::WanJian);
    p.target_entity = Some(Entity::from_raw(12345)); // 模拟死亡目标
    
    // 3. 运行寻敌逻辑 (通过封装系统避开复杂的 Query 构造问题)
    app.add_systems(Update, move |mut query: Query<&mut Particle>, eq: Query<(Entity, &Transform), With<EnemySpriteMarker>>| {
        for mut p_in in query.iter_mut() {
            update_wanjian_target(&mut p_in, &eq);
        }
    });
    
    let sword = app.world_mut().spawn(p).id();
    app.update();

    // 4. 验证重定向是否成功
    let p_after = app.world().get::<Particle>(sword).expect("Particle must exist");
    
    assert!(p_after.target.is_some(), "Sword MUST find a target");
    assert_eq!(p_after.target.unwrap().x, -88.0, "Sword should retarget to the survivor at -88.0");
    assert_eq!(p_after.target_entity.unwrap(), survivor_entity, "Target entity ID should match survivor");
}
