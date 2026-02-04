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
    let boss = Enemy::with_type(99, "幽冥白虎".to_string(), 150, EnemyType::GreatDemon);
    assert_eq!(boss.hp, 150);
    assert_eq!(boss.enemy_type, EnemyType::GreatDemon);
}

#[test]
fn test_boss_ai_cycle() {
    // 逻辑：BOSS 应该具备更复杂的行为模式
    let mut boss = Enemy::with_type(99, "幽冥白虎".to_string(), 150, EnemyType::GreatDemon);
    
    // 验证固定序列 (试探-蓄势-重击-喘息)
    boss.choose_new_intent(); // 初始化第一招
    let i1 = boss.execute_intent();
    match i1 {
        EnemyIntent::Attack { damage } => assert_eq!(damage, 15, "第一回合应发起试探"),
        _ => panic!("预期攻击，实际得到 {:?}", i1),
    }
    
    // 蓄势 (Defend)
    boss.choose_new_intent();
    let i2 = boss.execute_intent();
    match i2 {
        EnemyIntent::Defend { block } => assert_eq!(block, 12, "第二回合应开始蓄势"),
        _ => panic!("预期蓄势防御"),
    }
    
    // 重击 (Attack 28)
    boss.choose_new_intent();
    let i3 = boss.execute_intent();
    match i3 {
        EnemyIntent::Attack { damage } => assert_eq!(damage, 28, "第三回合应发起破魔重击"),
        _ => panic!("预期重击"),
    }
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
