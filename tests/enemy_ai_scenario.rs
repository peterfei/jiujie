//! 敌人AI自动化测试场景还原脚本
//!
//! 验证不同敌人类型的AI行为：
//! 1. 哥布林 - 高攻击概率（70%）
//! 2. 骷髅 - 均衡型（50%攻击，30%防御）
//! 3. 史莱姆 - 高防御概率（50%）
//! 4. Boss - 强力型（高伤害范围）

use crate::test_utils::*;
use bevy_card_battler::components::{Enemy, EnemyType, EnemyIntent};
use rand::SeedableRng;

// ============================================================================
// 场景1：哥布林AI测试
// ============================================================================

#[test]
fn scenario_ai_001_goblin_prefers_attack() {
    let mut app = create_test_app();
    let _enemy_entity = setup_combat_scene(&mut app);
    advance_frames(&mut app, 1);

    let world = app.world_mut();
    let mut enemy = world.query::<&mut Enemy>().get_single_mut(world).unwrap();

    // 哥布林应该有高攻击概率（70%）
    assert_eq!(enemy.enemy_type, EnemyType::Goblin);
    assert_eq!(enemy.ai_pattern.attack_chance, 0.7);
    assert_eq!(enemy.ai_pattern.defend_chance, 0.1);
    assert_eq!(enemy.ai_pattern.buff_chance, 0.2);

    println!("✓ 场景1：哥布林AI配置正确 (攻击70%, 防御10%, 强化20%)");
}

#[test]
fn scenario_ai_002_goblin_damage_range() {
    let mut app = create_test_app();
    let _enemy_entity = setup_combat_scene(&mut app);
    advance_frames(&mut app, 1);

    let world = app.world_mut();
    let enemy = world.query::<&Enemy>().get_single(world).unwrap();

    // 哥布林伤害范围 8-12
    assert_eq!(enemy.ai_pattern.damage_range, (8, 12));
    assert_eq!(enemy.ai_pattern.block_range, (3, 5));
    assert_eq!(enemy.ai_pattern.buff_range, (1, 2));

    println!("✓ 场景2：哥布林伤害范围正确 (8-12, 防御3-5, 强化1-2)");
}

#[test]
fn scenario_ai_003_goblin_intent_selection() {
    // 测试哥布林多次选择意图，应该主要是攻击
    let mut attack_count = 0;
    let mut defend_count = 0;
    let mut buff_count = 0;

    for _seed in 0..100 {
        let mut enemy = Enemy::new(0, "哥布林", 30);
        enemy.choose_new_intent();

        match enemy.intent {
            EnemyIntent::Attack { .. } => attack_count += 1,
            EnemyIntent::Defend { .. } => defend_count += 1,
            EnemyIntent::Buff { .. } => buff_count += 1,
            EnemyIntent::Wait => {}
        }
    }

    // 攻击应该占大多数（约70%）
    let attack_ratio = attack_count as f32 / 100.0;
    assert!(attack_ratio > 0.5, "哥布林攻击概率应该 > 50%，实际: {:.2}", attack_ratio);

    println!("✓ 场景3：哥布林意图分布 - 攻击: {}%, 防御: {}%, 强化: {}%",
        attack_count, defend_count, buff_count);
}

// ============================================================================
// 场景2：骷髅AI测试
// ============================================================================

#[test]
fn scenario_ai_101_skeleton_balanced_pattern() {
    let enemy = Enemy::with_type(0, "骷髅", 40, EnemyType::Skeleton);

    assert_eq!(enemy.enemy_type, EnemyType::Skeleton);
    assert_eq!(enemy.ai_pattern.attack_chance, 0.5);
    assert_eq!(enemy.ai_pattern.defend_chance, 0.3);
    assert_eq!(enemy.ai_pattern.buff_chance, 0.2);

    println!("✓ 场景101：骷髅AI配置正确 (攻击50%, 防御30%, 强化20%)");
}

#[test]
fn scenario_ai_102_skeleton_moderate_stats() {
    let enemy = Enemy::with_type(0, "骷髅", 40, EnemyType::Skeleton);

    assert_eq!(enemy.ai_pattern.damage_range, (6, 10));
    assert_eq!(enemy.ai_pattern.block_range, (5, 8));
    assert_eq!(enemy.ai_pattern.buff_range, (2, 3));

    println!("✓ 场景102：骷髅属性中等 (伤害6-10, 防御5-8, 强化2-3)");
}

// ============================================================================
// 场景3：史莱姆AI测试
// ============================================================================

#[test]
fn scenario_ai_201_slime_defensive_pattern() {
    let enemy = Enemy::with_type(0, "史莱姆", 50, EnemyType::Slime);

    assert_eq!(enemy.enemy_type, EnemyType::Slime);
    assert_eq!(enemy.ai_pattern.attack_chance, 0.3);
    assert_eq!(enemy.ai_pattern.defend_chance, 0.5);
    assert_eq!(enemy.ai_pattern.buff_chance, 0.2);

    println!("✓ 场景201：史莱姆AI配置正确 (攻击30%, 防御50%, 强化20%)");
}

#[test]
fn scenario_ai_202_slime_high_block() {
    let enemy = Enemy::with_type(0, "史莱姆", 50, EnemyType::Slime);

    assert_eq!(enemy.ai_pattern.damage_range, (4, 7));
    assert_eq!(enemy.ai_pattern.block_range, (8, 12));  // 高防御

    println!("✓ 场景202：史莱姆高防御 (伤害4-7, 防御8-12)");
}

// ============================================================================
// 场景4：Boss AI测试
// ============================================================================

#[test]
fn scenario_ai_301_boss_powerful_pattern() {
    let enemy = Enemy::with_type(0, "Boss", 100, EnemyType::Boss);

    assert_eq!(enemy.enemy_type, EnemyType::Boss);
    assert_eq!(enemy.ai_pattern.attack_chance, 0.6);
    assert_eq!(enemy.ai_pattern.defend_chance, 0.2);
    assert_eq!(enemy.ai_pattern.buff_chance, 0.2);

    println!("✓ 场景301：Boss AI配置正确 (攻击60%, 防御20%, 强化20%)");
}

#[test]
fn scenario_ai_302_boss_high_damage() {
    let enemy = Enemy::with_type(0, "Boss", 100, EnemyType::Boss);

    assert_eq!(enemy.ai_pattern.damage_range, (12, 18));  // 高伤害
    assert_eq!(enemy.ai_pattern.block_range, (6, 10));
    assert_eq!(enemy.ai_pattern.buff_range, (3, 5));  // 高强化

    println!("✓ 场景302：Boss高属性 (伤害12-18, 防御6-10, 强化3-5)");
}

// ============================================================================
// 场景5：意图执行测试
// ============================================================================

#[test]
fn scenario_ai_401_execute_attack_intent() {
    let mut enemy = Enemy::new(0, "哥布林", 30);
    enemy.intent = EnemyIntent::Attack { damage: 10 };

    let executed = enemy.execute_intent();
    match executed {
        EnemyIntent::Attack { damage } => {
            assert_eq!(damage, 10);
        }
        _ => panic!("应该执行攻击意图"),
    }

    println!("✓ 场景401：攻击意图执行正确");
}

#[test]
fn scenario_ai_402_execute_defend_intent() {
    let mut enemy = Enemy::new(0, "哥布林", 30);
    enemy.intent = EnemyIntent::Defend { block: 5 };

    let executed = enemy.execute_intent();
    match executed {
        EnemyIntent::Defend { block } => {
            assert_eq!(block, 5);
            assert_eq!(enemy.block, 5);  // 敌人应该获得护甲
        }
        _ => panic!("应该执行防御意图"),
    }

    println!("✓ 场景402：防御意图执行正确，护甲+5");
}

#[test]
fn scenario_ai_403_execute_buff_intent() {
    let mut enemy = Enemy::new(0, "哥布林", 30);
    enemy.intent = EnemyIntent::Buff { strength: 2 };

    let executed = enemy.execute_intent();
    match executed {
        EnemyIntent::Buff { strength } => {
            assert_eq!(strength, 2);
            assert_eq!(enemy.strength, 2);  // 敌人应该获得攻击力
        }
        _ => panic!("应该执行强化意图"),
    }

    println!("✓ 场景403：强化意图执行正确，攻击力+2");
}

#[test]
fn scenario_ai_404_buff_increases_damage() {
    let mut enemy = Enemy::new(0, "哥布林", 30);
    enemy.strength = 3;  // 先获得3点攻击力
    enemy.intent = EnemyIntent::Attack { damage: 8 };

    enemy.execute_intent();

    // 下次选择意图时应该考虑攻击力加成
    enemy.choose_new_intent();
    match enemy.intent {
        EnemyIntent::Attack { damage } => {
            // 伤害应该包含基础伤害 + strength
            assert!(damage >= 8 + 3, "伤害应该包含strength加成，实际: {}", damage);
        }
        _ => {}
    }

    println!("✓ 场景404：强化加成影响伤害计算");
}

// ============================================================================
// 场景6：回合转换测试
// ============================================================================

#[test]
fn scenario_ai_501_enemy_start_turn_clears_block() {
    let mut enemy = Enemy::new(0, "哥布林", 30);
    enemy.block = 10;

    enemy.start_turn();

    assert_eq!(enemy.block, 0, "回合开始时护甲应该清零");
    assert!(!matches!(enemy.intent, EnemyIntent::Wait), "应该选择新意图");

    println!("✓ 场景501：回合开始时护甲清零");
}

#[test]
fn scenario_ai_502_enemy_start_turn_chooses_new_intent() {
    let mut enemy = Enemy::new(0, "哥布林", 30);
    enemy.intent = EnemyIntent::Attack { damage: 10 };

    enemy.start_turn();

    // 意图应该改变（除非运气相同）
    let old_intent = enemy.intent;
    enemy.start_turn();

    // 新回合应该重新选择意图
    assert!(!matches!(enemy.intent, EnemyIntent::Wait), "应该总是有有效意图");

    println!("✓ 场景502：回合开始时选择新意图");
}

// ============================================================================
// 场景7：多类型敌人对比
// ============================================================================

#[test]
fn scenario_ai_601_all_enemy_types_valid() {
    let goblin = Enemy::with_type(0, "哥布林", 30, EnemyType::Goblin);
    let skeleton = Enemy::with_type(1, "骷髅", 40, EnemyType::Skeleton);
    let slime = Enemy::with_type(2, "史莱姆", 50, EnemyType::Slime);
    let boss = Enemy::with_type(3, "Boss", 100, EnemyType::Boss);

    // 验证概率总和为1.0
    let goblin_sum = goblin.ai_pattern.attack_chance + goblin.ai_pattern.defend_chance + goblin.ai_pattern.buff_chance;
    assert!((goblin_sum - 1.0).abs() < 0.01, "哥布林概率总和应为1.0，实际: {:.2}", goblin_sum);

    let skeleton_sum = skeleton.ai_pattern.attack_chance + skeleton.ai_pattern.defend_chance + skeleton.ai_pattern.buff_chance;
    assert!((skeleton_sum - 1.0).abs() < 0.01, "骷髅概率总和应为1.0，实际: {:.2}", skeleton_sum);

    let slime_sum = slime.ai_pattern.attack_chance + slime.ai_pattern.defend_chance + slime.ai_pattern.buff_chance;
    assert!((slime_sum - 1.0).abs() < 0.01, "史莱姆概率总和应为1.0，实际: {:.2}", slime_sum);

    let boss_sum = boss.ai_pattern.attack_chance + boss.ai_pattern.defend_chance + boss.ai_pattern.buff_chance;
    assert!((boss_sum - 1.0).abs() < 0.01, "Boss概率总和应为1.0，实际: {:.2}", boss_sum);

    println!("✓ 场景601：所有敌人类型的AI概率配置有效");
}

#[test]
fn scenario_ai_602_enemy_types_have_distinct_patterns() {
    let goblin = Enemy::with_type(0, "哥布林", 30, EnemyType::Goblin);
    let skeleton = Enemy::with_type(1, "骷髅", 40, EnemyType::Skeleton);
    let slime = Enemy::with_type(2, "史莱姆", 50, EnemyType::Slime);
    let boss = Enemy::with_type(3, "Boss", 100, EnemyType::Boss);

    // 哥布林攻击概率最高
    assert!(goblin.ai_pattern.attack_chance > skeleton.ai_pattern.attack_chance);
    assert!(goblin.ai_pattern.attack_chance > slime.ai_pattern.attack_chance);

    // 史莱姆防御概率最高
    assert!(slime.ai_pattern.defend_chance > goblin.ai_pattern.defend_chance);
    assert!(slime.ai_pattern.defend_chance > skeleton.ai_pattern.defend_chance);

    // Boss伤害最高
    assert!(boss.ai_pattern.damage_range.0 > goblin.ai_pattern.damage_range.0);
    assert!(boss.ai_pattern.damage_range.1 > goblin.ai_pattern.damage_range.1);

    println!("✓ 场景602：敌人类型有明显的AI模式差异");
}
