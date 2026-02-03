use bevy::prelude::*;
use bevy_card_battler::components::combat::{Player, Enemy, EnemyType, AiPattern, EnemyIntent};

#[test]
fn test_boss_v2_sequence_looping() {
    let mut ai = AiPattern::great_demon();
    
    // 初始序列: 攻击(15) -> 防御(12) -> 攻击(28) -> 等待
    let i1 = ai.next_intent(0.5, 0);
    match i1 {
        EnemyIntent::Attack { damage } => assert_eq!(damage, 15),
        _ => panic!("Expected Attack(15)"),
    }
    
    let i2 = ai.next_intent(0.5, 0);
    match i2 {
        EnemyIntent::Defend { block } => assert_eq!(block, 12),
        _ => panic!("Expected Defend(12)"),
    }
    
    let i3 = ai.next_intent(0.5, 0);
    match i3 {
        EnemyIntent::Attack { damage } => assert_eq!(damage, 28),
        _ => panic!("Expected Attack(28)"),
    }
    
    let i4 = ai.next_intent(0.5, 0);
    assert!(matches!(i4, EnemyIntent::Wait));
    
    let i5 = ai.next_intent(0.5, 0);
    match i5 {
        EnemyIntent::Attack { damage } => assert_eq!(damage, 15, "Should loop back to start"),
        _ => panic!("Expected Loop Back"),
    }
}

#[test]
fn test_boss_rage_mode_transition() {
    let mut boss = Enemy::with_type(1, "Great Demon".to_string(), 100, EnemyType::GreatDemon);
    
    // 初始意图应来自一阶段序列
    boss.choose_new_intent();
    match boss.intent {
        EnemyIntent::Attack { damage } => assert_eq!(damage, 15),
        _ => panic!("Step 1 should be Attack(15)"),
    }
    
    // 伤害 Boss，使其跌破 50% 血量 (100 -> 40)
    boss.hp = 40;
    boss.choose_new_intent();
    
    // 应该触发二阶段切换，序列变为: Attack(35) -> Buff(8) -> Attack(25)
    match boss.intent {
        EnemyIntent::Attack { damage } => assert_eq!(damage, 35, "Phase 2 should start with high damage attack"),
        _ => panic!("Expected Phase 2 Attack(35), got {:?}", boss.intent),
    }
    
    boss.choose_new_intent();
    match boss.intent {
        EnemyIntent::Buff { strength } => assert_eq!(strength, 8),
        _ => panic!("Step 2 of Phase 2 should be Buff(8)"),
    }
}

#[test]
fn test_boss_charge_doubles_next_attack() {
    let mut boss = Enemy::with_type(1, "Great Demon".to_string(), 100, EnemyType::GreatDemon);
    
    // 1. 模拟 Boss 执行了一个带“蓄势”性质的防御招式
    boss.intent = EnemyIntent::Defend { block: 10 };
    boss.execute_intent(); // 执行招式，应该触发“蓄势”状态
    
    // 2. 检查第一次攻击的伤害计算
    let base_damage = 20;
    // 预期：20 * 2 = 40 (因为处于蓄势状态)
    let final_damage_1 = boss.calculate_outgoing_damage(base_damage);
    assert_eq!(final_damage_1, 40, "第一次蓄势后的攻击伤害应该翻倍");

    // 模拟重构后的系统行为：攻击后消耗蓄势
    boss.consume_charge(); 

    // 3. 检查第二次攻击
    let final_damage_2 = boss.calculate_outgoing_damage(base_damage);
    assert_eq!(final_damage_2, 20, "第二次攻击伤害应该恢复正常");
}