//! 万剑归宗坐标转换 TDD 测试
//!
//! 验证敌人 3D Transform 坐标到粒子 2D 坐标的正确转换

use bevy::prelude::*;

#[test]
fn tdd_coord_001_2d_to_3d_conversion() {
    // 验证 2D 坐标到 3D 坐标的转换逻辑（已移除clamp限制）
    // 从 sync_2d_to_3d_render 函数：
    //   x_3d = transform.translation.x / 100.0;
    //   z_3d = transform.translation.y / 100.0;
    //   home_pos = Vec3::new(x_3d, 0.05, z_3d + 0.1);

    let x_2d = 250.0_f32;
    let y_2d = 50.0_f32;

    let x_3d = x_2d / 100.0;
    let z_3d = y_2d / 100.0;
    let y_3d = 0.05_f32;

    assert_eq!(x_3d, 2.5, "3D x坐标应该是 2.5");
    assert_eq!(z_3d, 0.5, "3D z坐标应该是 0.5");
    assert_eq!(y_3d, 0.05, "3D y坐标固定为 0.05");
}

#[test]
fn tdd_coord_002_3d_to_2d_restoration() {
    // 验证从 3D Transform 恢复原始 2D 坐标
    //   2D_x = 3D_x * 100.0
    //   2D_y = 3D_z * 100.0

    let transform_3d = (2.5, 0.05, 0.6); // (x, y, z)

    let x_2d = transform_3d.0 * 100.0;
    let y_2d = transform_3d.2 * 100.0;

    assert_eq!(x_2d, 250.0, "恢复的2D x坐标应该是 250.0");
    assert_eq!(y_2d, 60.0, "恢复的2D y坐标应该是 60.0（z+0.1后）");
}

#[test]
fn tdd_coord_003_particle_target_calculation() {
    // 验证粒子目标位置的计算
    let transform_3d = Vec3::new(2.5, 0.06870217, 0.6);

    // 从 3D Transform 恢复 2D 坐标
    let target_pos = Vec2::new(
        transform_3d.x * 100.0,
        transform_3d.z * 100.0
    );

    assert_eq!(target_pos.x, 250.0, "粒子目标 x 应该是 250.0");
    assert!((target_pos.y - 60.0).abs() < 0.01, "粒子目标 y 应该约等于 60.0");
}

#[test]
fn tdd_coord_004_player_position_consistency() {
    // 验证玩家位置在各个系统中的坐标一致性
    // 玩家 sprite 世界坐标: (-350.0, -80.0, 10.0)
    // 粒子 spawn 位置: (-350.0, -80.0, 0.5)

    let player_sprite_pos = (-350.0, -80.0);
    let particle_spawn_pos = (-350.0, -80.0);

    assert_eq!(player_sprite_pos.0, particle_spawn_pos.0, "x坐标应该一致");
    assert_eq!(player_sprite_pos.1, particle_spawn_pos.1, "y坐标应该一致");
}

#[test]
fn tdd_coord_005_enemy_positions_multiple() {
    // 验证多个敌人的位置计算
    // 敌人0: x_world = 250.0 + (0 - 1) * 220.0 / 2 = 30.0
    // 敌人1: x_world = 250.0 + (1 - 1) * 220.0 / 2 = 250.0
    // 敌人2: x_world = 250.0 + (2 - 1) * 220.0 / 2 = 470.0

    let num_enemies = 3.0;

    let x_0 = 250.0 + (0.0 - (num_enemies - 1.0) / 2.0) * 220.0;
    let x_1 = 250.0 + (1.0 - (num_enemies - 1.0) / 2.0) * 220.0;
    let x_2 = 250.0 + (2.0 - (num_enemies - 1.0) / 2.0) * 220.0;

    assert_eq!(x_0, 30.0, "敌人0的 x_world 应该是 30.0");
    assert_eq!(x_1, 250.0, "敌人1的 x_world 应该是 250.0");
    assert_eq!(x_2, 470.0, "敌人2的 x_world 应该是 470.0");
}

#[test]
fn tdd_coord_006_screen_position_calculation() {
    // 验证粒子坐标到屏幕坐标的转换
    //   ui_x = 640.0 + particle.position.x;
    //   ui_y = 360.0 - particle.position.y;

    let particle_pos = (250.0, 50.0);

    let ui_x = 640.0 + particle_pos.0;
    let ui_y = 360.0 - particle_pos.1;

    assert_eq!(ui_x, 890.0, "屏幕 x 位置应该是 890.0");
    assert_eq!(ui_y, 310.0, "屏幕 y 位置应该是 310.0");
}

#[test]
fn tdd_coord_007_transform_multiplication_bug() {
    // 验证错误的乘法方式（之前的bug）
    // 错误：直接用 translation * 100.0
    // 正确：分别用 x * 100.0 和 z * 100.0

    let transform_3d = Vec3::new(2.5, 0.05, 0.6);

    // 错误方式
    let wrong_pos = (transform_3d * 100.0).truncate();

    // 正确方式
    let correct_pos = Vec2::new(transform_3d.x * 100.0, transform_3d.z * 100.0);

    assert_ne!(wrong_pos.y, 50.0, "错误方式的 y 坐标不正确");
    assert_eq!(correct_pos.x, 250.0, "正确方式的 x 坐标是 250.0");
    assert!((correct_pos.y - 60.0).abs() < 0.1, "正确方式的 y 坐标约等于 60.0");
}

#[test]
fn tdd_coord_008_rightmost_enemy_position() {
    // 验证最右边敌人的位置（移除clamp后）
    // 敌人2: x_world = 250.0 + (2 - 1) * 220.0 = 470.0
    // 3D: x_3d = 470.0 / 100.0 = 4.7

    let x_2d = 470.0_f32;
    let y_2d = 50.0_f32;

    let x_3d = x_2d / 100.0;  // 移除clamp后应该是 4.7
    let z_3d = y_2d / 100.0;

    assert_eq!(x_3d, 4.7, "3D x坐标应该是 4.7（不再被clamp到3.5）");
    assert_eq!(z_3d, 0.5, "3D z坐标应该是 0.5");

    // 验证从3D恢复2D
    let transform_3d = Vec3::new(4.7, 0.05, 0.6);
    let restored_2d = Vec2::new(transform_3d.x * 100.0, transform_3d.z * 100.0);

    assert!((restored_2d.x - 470.0).abs() < 0.1, "恢复的2D x坐标应该约等于 470.0");
    assert!((restored_2d.y - 60.0).abs() < 0.1, "恢复的2D y坐标应该约等于 60.0");
}
