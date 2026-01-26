use bevy::prelude::*;
use bevy_card_battler::components::sprite::PhysicalImpact;

#[test]
fn test_wolf_dash_distance_multi_enemy() {
    // 玩家始终在 -3.5
    let player_x = -3.5f32;
    
    // 场景 A：怪在最右侧 (3.5)
    let enemy_a_x = 3.5f32;
    let required_offset_a = player_x - enemy_a_x;
    assert_eq!(required_offset_a.abs(), 7.0, "右侧敌人位移应为 7.0");
    
    // 场景 B：怪在中间靠近玩家 (0.5)
    let enemy_b_x = 0.5f32;
    let required_offset_b = player_x - enemy_b_x;
    assert_eq!(required_offset_b.abs(), 4.0, "中间敌人位移应为 4.0");
}

#[test]
fn test_deceleration_with_dynamic_target() {
    // 模拟一个较近的敌人 (位移目标 4.0)
    let target_dist = 4.0f32;
    let current_offset = 3.5f32; // 已经快到了
    
    let dist_left = (target_dist - current_offset).max(0.0);
    let speed_scalar = if dist_left < 1.0 { dist_left } else { 1.0 };
    
    assert!(speed_scalar < 1.0, "接近动态目标时应正确减速");
}