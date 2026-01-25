//! 敌人AI自动化测试场景还原脚本
//!
//! 验证不同敌人类型的AI行为：
//! 1. 嗜血妖狼 - 高攻击概率（70%）
//! 2. 巡逻阴兵 - 均衡型（50%攻击，30%防御）
//! 3. 地府幽火 - 高防御概率（50%）
//! 4. 筑基大妖 - 强力型（高伤害范围）

use crate::test_utils::*;
use bevy_card_battler::components::{Enemy, EnemyType, EnemyIntent};
use rand::SeedableRng;

// ============================================================================
// 场景1：嗜血妖狼AI测试
// ============================================================================

#[test]
fn scenario_ai_001_demonic_wolf_prefers_attack() {
    let mut app = create_test_app();
    let _enemy_entity = setup_combat_scene(&mut app);
    advance_frames(&mut app, 1);

    let world = app.world_mut();
    let mut enemy = world.query::<&mut Enemy>().get_single_mut(world).unwrap();

    // 嗜血妖狼应该有高攻击概率（70%）
    assert_eq!(enemy.enemy_type, EnemyType::DemonicWolf);
    assert_eq!(enemy.ai_pattern.attack_chance, 0.7);
    assert_eq!(enemy.ai_pattern.defend_chance, 0.1);
    assert_eq!(enemy.ai_pattern.buff_chance, 0.2);

    println!("✓ 场景1：嗜血妖狼AI配置正确 (攻击70%, 防御10%, 强化20%)");
}

#[test]
fn scenario_ai_002_demonic_wolf_damage_range() {
    let mut app = create_test_app();
    let _enemy_entity = setup_combat_scene(&mut app);
    advance_frames(&mut app, 1);

    let world = app.world_mut();
    let enemy = world.query::<&Enemy>().get_single(world).unwrap();

    // 嗜血妖狼伤害范围 8-12
    assert_eq!(enemy.ai_pattern.damage_range, (8, 12));
    assert_eq!(enemy.ai_pattern.block_range, (3, 5));
    assert_eq!(enemy.ai_pattern.buff_range, (1, 2));

    println!("✓ 场景2：嗜血妖狼伤害范围正确 (8-12, 防御3-5, 强化1-2)");
}

#[test]
fn scenario_ai_003_demonic_wolf_intent_selection() {
    // 测试嗜血妖狼多次选择意图，应该主要是攻击
    let mut attack_count = 0;
    let mut defend_count = 0;
    let mut buff_count = 0;

    for _seed in 0..100 {
        let mut enemy = Enemy::new(0, "嗜血妖狼", 30);
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
    assert!(attack_ratio > 0.5, "嗜血妖狼攻击概率应该 > 50%，实际: {:.2}", attack_ratio);

    println!("✓ 场景3：嗜血妖狼意图分布 - 攻击: {}%, 防御: {}%, 强化: {}%",
        attack_count, defend_count, buff_count);
}

// ============================================================================
// 场景2：巡逻阴兵AI测试
// ============================================================================

#[test]
fn scenario_ai_101_ghost_soldier_balanced_pattern() {
    let enemy = Enemy::with_type(0, "巡逻阴兵", 40, EnemyType::GhostSoldier);

    assert_eq!(enemy.enemy_type, EnemyType::GhostSoldier);
    assert_eq!(enemy.ai_pattern.attack_chance, 0.5);
    assert_eq!(enemy.ai_pattern.defend_chance, 0.3);
    assert_eq!(enemy.ai_pattern.buff_chance, 0.2);

    println!("✓ 场景101：巡逻阴兵AI配置正确 (攻击50%, 防御30%, 强化20%)");
}

#[test]
fn scenario_ai_102_ghost_soldier_moderate_stats() {
    let enemy = Enemy::with_type(0, "巡逻阴兵", 40, EnemyType::GhostSoldier);

    assert_eq!(enemy.ai_pattern.damage_range, (6, 10));
    assert_eq!(enemy.ai_pattern.block_range, (5, 8));
    assert_eq!(enemy.ai_pattern.buff_range, (2, 3));

    println!("✓ 场景102：巡逻阴兵属性中等 (伤害6-10, 防御5-8, 强化2-3)");
}

// ============================================================================
// 场景3：地府幽火AI测试
// ============================================================================

#[test]
fn scenario_ai_201_spirit_fire_defensive_pattern() {
    let enemy = Enemy::with_type(0, "地府幽火", 50, EnemyType::SpiritFire);

    assert_eq!(enemy.enemy_type, EnemyType::SpiritFire);
    assert_eq!(enemy.ai_pattern.attack_chance, 0.3);
    assert_eq!(enemy.ai_pattern.defend_chance, 0.5);
    assert_eq!(enemy.ai_pattern.buff_chance, 0.2);

    println!("✓ 场景201：地府幽火AI配置正确 (攻击30%, 防御50%, 强化20%)");
}

#[test]
fn scenario_ai_202_spirit_fire_high_block() {
    let enemy = Enemy::with_type(0, "地府幽火", 50, EnemyType::SpiritFire);

    assert_eq!(enemy.ai_pattern.damage_range, (4, 7));
    assert_eq!(enemy.ai_pattern.block_range, (8, 12));  // 高防御

    println!("✓ 场景202：地府幽火高防御 (伤害4-7, 防御8-12)");
}

// ============================================================================
// 场景4：筑基大妖 AI测试
// ============================================================================

#[test]
fn scenario_ai_301_great_demon_powerful_pattern() {
    let enemy = Enemy::with_type(0, "筑基大妖", 100, EnemyType::GreatDemon);

    assert_eq!(enemy.enemy_type, EnemyType::GreatDemon);
    assert_eq!(enemy.ai_pattern.attack_chance, 0.6);
    assert_eq!(enemy.ai_pattern.defend_chance, 0.2);
    assert_eq!(enemy.ai_pattern.buff_chance, 0.2);

    println!("✓ 场景301：筑基大妖 AI配置正确 (攻击60%, 防御20%, 强化20%)");
}

#[test]
fn scenario_ai_302_great_demon_high_damage() {
    let enemy = Enemy::with_type(0, "筑基大妖", 100, EnemyType::GreatDemon);

    assert_eq!(enemy.ai_pattern.damage_range, (12, 18));  // 高伤害
    assert_eq!(enemy.ai_pattern.block_range, (6, 10));
    assert_eq!(enemy.ai_pattern.buff_range, (3, 5));  // 高强化

    println!("✓ 场景302：筑基大妖高属性 (伤害12-18, 防御6-10, 强化3-5)");
}

// ============================================================================
// 场景5：意图执行测试
// ============================================================================

#[test]
fn scenario_ai_401_execute_attack_intent() {
    let mut enemy = Enemy::new(0, "嗜血妖狼", 30);
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
    let mut enemy = Enemy::new(0, "嗜血妖狼", 30);
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
    let mut enemy = Enemy::new(0, "嗜血妖狼", 30);
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
    let mut enemy = Enemy::new(0, "嗜血妖狼", 30);
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
    let mut enemy = Enemy::new(0, "嗜血妖狼", 30);
    enemy.block = 10;

    enemy.start_turn();

    assert_eq!(enemy.block, 0, "回合开始时护甲应该清零");
    assert!(!matches!(enemy.intent, EnemyIntent::Wait), "应该选择新意图");

    println!("✓ 场景501：回合开始时护甲清零");
}

#[test]
fn scenario_ai_502_enemy_start_turn_chooses_new_intent() {
    let mut enemy = Enemy::new(0, "嗜血妖狼", 30);
    enemy.intent = EnemyIntent::Attack { damage: 10 };

    enemy.start_turn();

    // 意图应该改变（除非运气相同）
    let _old_intent = enemy.intent;
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
    let demonic_wolf = Enemy::with_type(0, "嗜血妖狼", 30, EnemyType::DemonicWolf);
    let ghost_soldier = Enemy::with_type(1, "巡逻阴兵", 40, EnemyType::GhostSoldier);
    let spirit_fire = Enemy::with_type(2, "地府幽火", 50, EnemyType::SpiritFire);
    let great_demon = Enemy::with_type(3, "筑基大妖", 100, EnemyType::GreatDemon);

    // 验证概率总和为1.0
    let wolf_sum = demonic_wolf.ai_pattern.attack_chance + demonic_wolf.ai_pattern.defend_chance + demonic_wolf.ai_pattern.buff_chance;
    assert!((wolf_sum - 1.0).abs() < 0.01, "妖狼概率总和应为1.0，实际: {:.2}", wolf_sum);

    let ghost_sum = ghost_soldier.ai_pattern.attack_chance + ghost_soldier.ai_pattern.defend_chance + ghost_soldier.ai_pattern.buff_chance;
    assert!((ghost_sum - 1.0).abs() < 0.01, "阴兵概率总和应为1.0，实际: {:.2}", ghost_sum);

    let fire_sum = spirit_fire.ai_pattern.attack_chance + spirit_fire.ai_pattern.defend_chance + spirit_fire.ai_pattern.buff_chance;
    assert!((fire_sum - 1.0).abs() < 0.01, "幽火概率总和应为1.0，实际: {:.2}", fire_sum);

    let demon_sum = great_demon.ai_pattern.attack_chance + great_demon.ai_pattern.defend_chance + great_demon.ai_pattern.buff_chance;
    assert!((demon_sum - 1.0).abs() < 0.01, "筑基大妖概率总和应为1.0，实际: {:.2}", demon_sum);

    println!("✓ 场景601：所有敌人类型的AI概率配置有效");
}

#[test]
fn scenario_ai_602_enemy_types_have_distinct_patterns() {
    let demonic_wolf = Enemy::with_type(0, "嗜血妖狼", 30, EnemyType::DemonicWolf);
    let ghost_soldier = Enemy::with_type(1, "巡逻阴兵", 40, EnemyType::GhostSoldier);
    let spirit_fire = Enemy::with_type(2, "地府幽火", 50, EnemyType::SpiritFire);
    let great_demon = Enemy::with_type(3, "筑基大妖", 100, EnemyType::GreatDemon);

    // 嗜血妖狼攻击概率最高
    assert!(demonic_wolf.ai_pattern.attack_chance > ghost_soldier.ai_pattern.attack_chance);
    assert!(demonic_wolf.ai_pattern.attack_chance > spirit_fire.ai_pattern.attack_chance);

    // 地府幽火防御概率最高
    assert!(spirit_fire.ai_pattern.defend_chance > demonic_wolf.ai_pattern.defend_chance);
    assert!(spirit_fire.ai_pattern.defend_chance > ghost_soldier.ai_pattern.defend_chance);

    // 筑基大妖伤害最高
    assert!(great_demon.ai_pattern.damage_range.0 > demonic_wolf.ai_pattern.damage_range.0);
    assert!(great_demon.ai_pattern.damage_range.1 > demonic_wolf.ai_pattern.damage_range.1);

    println!("✓ 场景602：敌人类型有明显的AI模式差异");
}
