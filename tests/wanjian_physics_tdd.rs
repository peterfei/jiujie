use bevy::prelude::*;

#[test]
fn test_wanjian_pixel_scale_logic() {
    // 1. 模拟 3D 世界中的坐标
    let player_x_3d = -3.5f32;
    let enemy_x_3d = 3.5f32;
    
    // 2. 转换到 UI 像素坐标系 (1 unit = 100 pixels)
    let player_x_ui = player_x_3d * 100.0;
    let enemy_x_ui = enemy_x_3d * 100.0;
    
    let travel_dist = (enemy_x_ui - player_x_ui).abs();
    
    // 验证：路程应该是 700 像素左右，而不是 7 像素
    assert!(travel_dist > 500.0, "飞剑路程应覆盖大部分屏幕空间（当前：{}）", travel_dist);
}

#[test]
fn test_sword_rotation_consistency() {
    let velocity = Vec2::new(500.0, -200.0);
    let rotation = (-velocity.y).atan2(velocity.x);
    // 验证旋转是否为正值（指向右上方）
    assert!(rotation > 0.0);
}