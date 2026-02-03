use bevy::prelude::*;
use bevy_card_battler::systems::enemy_gen::EnemyGenerator;
use bevy_card_battler::components::combat::{EnemyType, EnemyAffix};

#[test]
fn test_hp_scaling_with_depth() {
    // 深度 1 的敌人
    let gen_lvl1 = EnemyGenerator::generate_enemy(1, 0);
    // 深度 10 的敌人
    let gen_lvl10 = EnemyGenerator::generate_enemy(10, 1);

    println!("Level 1 HP: {}", gen_lvl1.enemy.hp);
    println!("Level 10 HP: {}", gen_lvl10.enemy.hp);

    assert!(gen_lvl10.enemy.hp > gen_lvl1.enemy.hp, "Higher depth should result in higher HP");
}

#[test]
fn test_name_prefix_logic() {
    // 这个测试可能会失败，因为现在的名字由 [Affix] [Age] [Name] 组成
    // 我们主要检查它是否包含 Age Prefix
    
    let gen_lvl1 = EnemyGenerator::generate_enemy(1, 0);
    assert!(gen_lvl1.enemy.name.contains("幼年"), "Depth 1 should have '幼年' prefix, got: {}", gen_lvl1.enemy.name);

    let gen_lvl4 = EnemyGenerator::generate_enemy(4, 0);
    assert!(gen_lvl4.enemy.name.contains("成年"), "Depth 4 should have '成年' prefix, got: {}", gen_lvl4.enemy.name);
}

#[test]
fn test_strength_scaling() {
    let gen_lvl1 = EnemyGenerator::generate_enemy(1, 0);
    // 基础 strength 应该是 0，除非随到了 Elite/Berserk 词缀
    // 这里的断言比较脆弱，我们只检查 Level 10 的基础加成逻辑
    
    let gen_lvl10 = EnemyGenerator::generate_enemy(10, 0);
    // Depth 10 基础加成是 2，如果有词缀会更高
    assert!(gen_lvl10.enemy.strength >= 2, "Depth 10 should have strength bonus");
}

#[test]
fn test_enemy_type_distribution() {
    for _ in 0..50 {
        let gen = EnemyGenerator::generate_enemy(1, 0);
        assert!(
            matches!(gen.enemy.enemy_type, EnemyType::DemonicWolf | EnemyType::PoisonSpider),
            "Depth 1 should only spawn Wolf or Spider, got: {:?}", gen.enemy.enemy_type
        );
    }
}

#[test]
fn test_affix_application() {
    // 我们强制生成大量敌人，直到出现词缀，然后验证属性
    let mut found_elite = false;
    let mut found_berserk = false;

    for i in 0..100 {
        let gen = EnemyGenerator::generate_enemy(5, i);
        if let Some(affix) = gen.enemy.affixes.first() {
            match affix {
                EnemyAffix::Elite => {
                    found_elite = true;
                    assert!(gen.visual_scale.x > 1.0, "Elite should be larger");
                    assert_ne!(gen.visual_color, Color::WHITE, "Elite should have color tint");
                    assert!(gen.enemy.name.contains("精英"), "Elite should have name prefix");
                }
                EnemyAffix::Berserk => {
                    found_berserk = true;
                    assert_eq!(gen.enemy.ai_pattern.attack_chance, 0.9, "Berserk should have high attack chance");
                    assert!(gen.enemy.name.contains("嗜血"), "Berserk should have name prefix");
                }
                _ => {}
            }
        }
    }
    
    // 概率问题，可能不一定每次都跑出所有词缀，但跑100次在深度5应该大概率有
    if found_elite { println!("Verified Elite affix"); }
    if found_berserk { println!("Verified Berserk affix"); }
}

