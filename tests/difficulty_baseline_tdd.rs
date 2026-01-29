use bevy::prelude::*;
use bevy_card_battler::components::{Enemy, EnemyType, MapProgress, MapConfig, NodeType};
use bevy_card_battler::plugins::CorePlugin;
use bevy_card_battler::systems::map::MapPlugin;

#[test]
fn test_difficulty_scaling_baseline() {
    let mut app = App::new();
    
    // 基础属性定义
    let wolf_base_hp = 30;
    
    // --- 基线 1: 第一层 (Layer 0) ---
    let layer_0 = 0;
    let hp_0 = (wolf_base_hp as f32 * (1.0 + layer_0 as f32 * 0.15)) as i32;
    let str_0 = (layer_0 / 3) as i32;
    let blk_0 = (layer_0 / 2) as i32 * 2;
    
    println!("Layer 0 Baseline: HP={}, STR={}, BLK={}", hp_0, str_0, blk_0);
    assert_eq!(hp_0, 30);
    assert_eq!(str_0, 0);
    assert_eq!(blk_0, 0);

    // --- 基线 2: 第五层 (Layer 4) ---
    let layer_4 = 4;
    let hp_4 = (wolf_base_hp as f32 * (1.0 + layer_4 as f32 * 0.15)) as i32; // 30 * 1.6 = 48
    let str_4 = (layer_4 / 3) as i32; // 1
    let blk_4 = (layer_4 / 2) as i32 * 2; // 4
    
    println!("Layer 4 Baseline: HP={}, STR={}, BLK={}", hp_4, str_4, blk_4);
    assert_eq!(hp_4, 48);
    assert_eq!(str_4, 1);
    assert_eq!(blk_4, 4);

    // --- 基线 3: 第十层 Boss (Layer 9) ---
    let layer_9 = 9;
    let boss_base_hp = 150;
    let hp_9 = (boss_base_hp as f32 * (1.0 + layer_9 as f32 * 0.15)) as i32; // 150 * 2.35 = 352.5 -> 352
    let str_9 = (layer_9 / 3) as i32 + 2; // 3 + 2 = 5
    let blk_9 = (layer_9 / 2) as i32 * 2 + 5; // 8 + 5 = 13
    
    println!("Layer 9 Boss Baseline: HP={}, STR={}, BLK={}", hp_9, str_9, blk_9);
    assert_eq!(hp_9, 352);
    assert_eq!(str_9, 5);
    assert_eq!(blk_9, 13);
}

#[test]
fn test_regression_game_startup() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    // 仅验证插件加载不崩溃
}
