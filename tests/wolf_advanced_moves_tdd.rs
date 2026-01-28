use bevy::prelude::*;
use bevy_card_battler::components::combat::{Enemy, EnemyType};
use bevy_card_battler::components::sprite::{PhysicalImpact, ActionType, BreathAnimation};
use bevy_card_battler::components::particle::{SpawnEffectEvent, EffectType};

#[test]
fn test_wolf_staggered_hits() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<Time>();
    app.add_event::<SpawnEffectEvent>();
    
    // 1. 设置一个正在执行贪狼撕咬的角色
    let mut impact = PhysicalImpact::default();
    impact.action_type = ActionType::WolfBite;
    impact.action_timer = 1.0; // 总时长 1.0
    impact.action_stage = 0;
    
    let ent = app.world_mut().spawn((
        impact,
        BreathAnimation::default(),
        Transform::default(),
    )).id();

    // 2. 模拟系统运行
    use bevy::ecs::system::RunSystemOnce;
    
    // 阶段 1: 进度 0.4 (超过 0.3)
    {
        let mut time = app.world_mut().resource_mut::<Time>();
        // 模拟流逝 0.4s
        time.advance_by(std::time::Duration::from_millis(400));
    }
    let _ = app.world_mut().run_system_once(bevy_card_battler::systems::sprite::update_physical_impacts);
    
    // 验证触发了第 1 段攻击
    let events = app.world().resource::<Events<SpawnEffectEvent>>();
    let mut reader = events.get_cursor();
    let hit_1 = reader.read(events).any(|e| e.effect_type == EffectType::Slash);
    assert!(hit_1, "进度 0.4 时应触发第一段斩击");
    
    // 验证 stage 已增加
    let impact_after = app.world().get::<PhysicalImpact>(ent).unwrap();
    assert_eq!(impact_after.action_stage, 1);

    // 阶段 2: 进度 0.7 (超过 0.6)
    {
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_millis(300));
    }
    let _ = app.world_mut().run_system_once(bevy_card_battler::systems::sprite::update_physical_impacts);
    
    // 验证触发了第 2 段攻击
    let events = app.world().resource::<Events<SpawnEffectEvent>>();
    let hit_2 = reader.read(events).any(|e| e.effect_type == EffectType::Slash);
    assert!(hit_2, "进度 0.7 时应触发第二段斩击");
    
    println!("✅ 魔狼多段打击同步逻辑验证通过");
}
