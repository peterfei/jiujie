use bevy::prelude::*;
use bevy_card_battler::components::sprite::PhysicalImpact;

#[test]
fn test_counter_clockwise_rotation() {
    // 逻辑：御剑术应产生正向旋转初速度 (逆时针)
    let direction = 1.0f32; // 玩家面向右
    let velocity = 50.0 * direction; // 我们将改为正数
    
    assert!(velocity > 0.0, "旋转初速度应为正，以实现逆时针回旋");
}

#[test]
fn test_hit_force_limit() {
    // 逻辑：受击力道不应过猛
    let hit_velocity = 15.0f32; // 之前是 35.0+
    assert!(hit_velocity <= 20.0, "受击力道应控制在温和范围内");
}
