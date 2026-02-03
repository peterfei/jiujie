use bevy::prelude::*;
use bevy_card_battler::systems::enemy_gen::EnemyGenerator;
use bevy_card_battler::components::combat::EnemyType;

#[test]
fn test_hp_scaling_with_depth() {
    // 深度 1 的敌人
    let enemy_lvl1 = EnemyGenerator::generate_enemy(1, 0);
    // 深度 10 的敌人
    let enemy_lvl10 = EnemyGenerator::generate_enemy(10, 1);

    println!("Level 1 HP: {}", enemy_lvl1.hp);
    println!("Level 10 HP: {}", enemy_lvl10.hp);

    // 假设原型相同（或者即使不同，深度加成也应该让 Level 10 显著更强）
    // 为了严谨，我们可以多次生成取平均值，或者直接比较趋势
    // 这里简单比较：深度 10 的敌人 HP 应该 > 深度 1 的敌人 HP
    // 注意：如果是不同种类（如狼 vs 恶魔），本身基础数值就有差异，符合预期
    assert!(enemy_lvl10.hp > enemy_lvl1.hp, "Higher depth should result in higher HP");
}

#[test]
fn test_name_prefix_logic() {
    let enemy_lvl1 = EnemyGenerator::generate_enemy(1, 0);
    assert!(enemy_lvl1.name.starts_with("幼年"), "Depth 1 should have '幼年' prefix, got: {}", enemy_lvl1.name);

    let enemy_lvl4 = EnemyGenerator::generate_enemy(4, 0);
    assert!(enemy_lvl4.name.starts_with("成年"), "Depth 4 should have '成年' prefix, got: {}", enemy_lvl4.name);

    let enemy_lvl7 = EnemyGenerator::generate_enemy(7, 0);
    assert!(enemy_lvl7.name.starts_with("狂暴"), "Depth 7 should have '狂暴' prefix, got: {}", enemy_lvl7.name);

    let enemy_lvl10 = EnemyGenerator::generate_enemy(10, 0);
    assert!(enemy_lvl10.name.starts_with("千年"), "Depth 10 should have '千年' prefix, got: {}", enemy_lvl10.name);
}

#[test]
fn test_strength_scaling() {
    let enemy_lvl1 = EnemyGenerator::generate_enemy(1, 0);
    assert_eq!(enemy_lvl1.strength, 0, "Depth 1 should have 0 strength bonus");

    let enemy_lvl10 = EnemyGenerator::generate_enemy(10, 0);
    // Depth 10 -> (10 - 5) * 0.5 = 2.5 -> 2
    assert_eq!(enemy_lvl10.strength, 2, "Depth 10 should have strength bonus");
}

#[test]
fn test_enemy_type_distribution() {
    // 简单验证一下深度 1 应该只有狼和蜘蛛
    for _ in 0..50 {
        let enemy = EnemyGenerator::generate_enemy(1, 0);
        assert!(
            matches!(enemy.enemy_type, EnemyType::DemonicWolf | EnemyType::PoisonSpider),
            "Depth 1 should only spawn Wolf or Spider, got: {:?}", enemy.enemy_type
        );
    }
}
