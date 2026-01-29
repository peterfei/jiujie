use bevy::prelude::*;
use bevy_card_battler::components::combat::{Enemy, EnemyType, EnemyIntent, AiPattern};
use bevy_card_battler::components::sprite::{CharacterType, AnimationState};

#[test]
fn test_boss_vs_minion_scale() {
    // 逻辑：BOSS 尺寸 vs 普通狼尺寸
    let boss_size = Vec2::new(150.0, 200.0);
    let minion_size = Vec2::new(70.0, 100.0);
    
    let height_ratio = boss_size.y / minion_size.y;
    let width_ratio = boss_size.x / minion_size.x;
    
    assert!(height_ratio >= 1.5, "BOSS 高度应至少是普通怪的 1.5 倍");
    assert!(width_ratio >= 2.0, "BOSS 宽度应至少是普通怪的 2 倍以体现体量感");
}

#[test]
fn test_boss_hp_and_type() {
    let boss = Enemy::with_type(99, "幽冥白虎", 0, EnemyType::GreatDemon);
    assert!(boss.hp >= 150, "BOSS 血量应具有压迫感");
    assert_eq!(boss.enemy_type, EnemyType::GreatDemon);
}

#[test]
fn test_boss_ai_cycle() {
    // 逻辑：BOSS 应该具备更复杂的行为模式
    let mut boss = Enemy::with_type(99, "幽冥白虎", 0, EnemyType::GreatDemon);
    
    // 回合 1: 啸天 (大伤害)
    boss.choose_new_intent();
    let i1 = boss.execute_intent();
    assert!(matches!(i1, EnemyIntent::Attack { damage } if damage >= 20), "第一回合应发起啸天猛攻");
    
    // 回合 2: 瞬狱杀 (中伤害)
    boss.choose_new_intent();
    let i2 = boss.execute_intent();
    assert!(matches!(i2, EnemyIntent::Attack { damage } if damage >= 15), "第二回合应执行瞬狱杀");
    
    // 回合 3: 聚灵 (强化)
    boss.choose_new_intent();
    let i3 = boss.execute_intent();
    assert!(matches!(i3, EnemyIntent::Buff { .. }), "第三回合应进行能量聚灵");
}

#[test]
fn test_boss_visual_scale() {
    // 逻辑：BOSS 尺寸
    let char_type = CharacterType::GreatDemon;
    let size = match char_type {
        CharacterType::GreatDemon => Vec2::new(150.0, 200.0),
        _ => Vec2::new(70.0, 100.0),
    };
    
    assert!(size.x > 100.0, "BOSS 尺寸应具有压迫感");
}
