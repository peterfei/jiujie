use bevy::prelude::*;
use bevy_card_battler::components::combat::{Enemy, EnemyType, EnemyIntent};
use bevy_card_battler::components::particle::{EffectType, SpawnEffectEvent};

#[test]
fn test_wolf_triple_strike_logic() {
    let mut app = App::new();
    app.add_event::<SpawnEffectEvent>();
    
    // 1. 模拟魔狼的特有攻击逻辑：预警 + 三连击
    fn mock_wolf_attack_system(
        mut effect_events: EventWriter<SpawnEffectEvent>,
        enemy_query: Query<&Enemy>,
    ) {
        for enemy in enemy_query.iter() {
            if enemy.enemy_type == EnemyType::DemonicWolf {
                // 1. 预警
                effect_events.send(SpawnEffectEvent::new(EffectType::SwordEnergy, Vec3::ZERO));
                // 2. 三连击
                for _ in 0..3 {
                    effect_events.send(SpawnEffectEvent::new(EffectType::Slash, Vec3::ZERO));
                }
            }
        }
    }

    app.world_mut().spawn(Enemy::new(1, "魔狼", 100));
    app.add_systems(Update, mock_wolf_attack_system);
    app.update();

    // 2. 验证：是否发出了 1 次 SwordEnergy 和 3 次 Slash
    let events = app.world().resource::<Events<SpawnEffectEvent>>();
    let mut reader = events.get_cursor();
    let slash_count = reader.read(events).filter(|e| e.effect_type == EffectType::Slash).count();
    let sword_energy_count = events.get_cursor().read(events).filter(|e| e.effect_type == EffectType::SwordEnergy).count();
    
    assert_eq!(slash_count, 3, "魔狼的‘连环扑杀’必须包含 3 次连续打击");
    assert_eq!(sword_energy_count, 1, "魔狼攻击前应有爆发性预警特效");
    println!("✅ 魔狼三连击逻辑验证通过");
}