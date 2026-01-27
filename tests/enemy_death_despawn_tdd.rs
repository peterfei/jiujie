use bevy::prelude::*;
use bevy_card_battler::components::combat::{Enemy, EnemyType};
use bevy_card_battler::components::sprite::{CharacterAnimationEvent, AnimationState, PhysicalImpact, EnemySpriteMarker, SpriteMarker};
use bevy_card_battler::systems::sprite::SpritePlugin;
use bevy_card_battler::states::GameState;

#[test]
fn test_enemy_despawns_after_death_animation() {
    let mut app = App::new();
    app.init_resource::<Time>();
    
    // 模拟敌人渲染实体
    let enemy_render_entity = app.world_mut().spawn((
        SpriteMarker,
        Sprite::default(),
        EnemySpriteMarker { id: 1 },
        PhysicalImpact::default(),
        Transform::default(),
        Visibility::Visible,
    )).id();

    // 1. 模拟修复后的受击反馈系统的插入行为
    // 我们直接往实体上插组件，模拟系统执行结果
    app.world_mut().entity_mut(enemy_render_entity).insert(
        bevy_card_battler::components::particle::EnemyDeathAnimation::new(0.5)
    );

    // 2. 模拟时间推进并显式运行销毁系统
    use bevy::ecs::system::RunSystemOnce;
    
    // 第一步：进度到达一半
    {
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_millis(250));
    }
    let _ = app.world_mut().run_system_once(bevy_card_battler::plugins::update_enemy_death_animation);
    
    // 验证还没销毁
    assert!(app.world().get_entity(enemy_render_entity).is_ok(), "Should NOT be despawned yet");

    // 第二步：进度到达终点
    {
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_millis(300));
    }
    let _ = app.world_mut().run_system_once(bevy_card_battler::plugins::update_enemy_death_animation);

    // 验证渲染实体已经销毁
    assert!(app.world().get_entity(enemy_render_entity).is_err(), "Enemy render entity should be despawned after death animation finishes");
}

