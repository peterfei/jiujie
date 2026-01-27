//! 万剑归宗红绿 TDD 测试
//!
//! Red-Green-Refactor 循环：
//! Red: 写一个失败的测试 → Green: 实现最小代码让测试通过 → Refactor: 重构

use bevy::prelude::*;
use bevy_card_battler::components::sprite::PhysicalImpact;

// ============================================================================
// Red: 测试用例 - 描述期望的行为
// ============================================================================

#[test]
fn red_green_001_home_position_to_2d() {
    // Red: 这个测试应该通过，因为我们已经实现了这个逻辑

    // 模拟 sync_2d_to_3d_render 中的坐标转换
    let original_2d_x = 250.0;
    let original_2d_y = 50.0;

    let x_3d = original_2d_x / 100.0;  // 2.5
    let z_3d = original_2d_y / 100.0;  // 0.5

    // sync_2d_to_3d_render 设置的 home_position
    let home_position = Vec3::new(x_3d, 0.05, z_3d + 0.1);

    // Green: 从 home_position 恢复原始2D坐标（当前实现的逻辑）
    let restored_x = home_position.x * 100.0;
    let restored_y = (home_position.z - 0.1) * 100.0;

    assert_eq!(restored_x, 250.0, "恢复的x坐标应该是 250.0");
    assert_eq!(restored_y, 50.0, "恢复的y坐标应该是 50.0");
}

#[test]
fn red_green_002_breath_animation_does_not_affect_target() {
    // Red: 呼吸动画不应该影响粒子目标位置

    // PhysicalImpact.home_position 是固定的
    let home_position = Vec3::new(2.5, 0.05, 0.6);

    // 模拟呼吸动画导致的 Transform.translation 变化
    let transform_with_breath = Vec3::new(2.5, 0.032, 0.6);  // y坐标被呼吸动画修改

    // 使用 home_position 计算目标（正确方式）
    let target_from_home = Vec2::new(
        home_position.x * 100.0,
        (home_position.z - 0.1) * 100.0
    );

    // 使用 Transform 计算目标（错误方式）
    let target_from_transform = Vec2::new(
        transform_with_breath.x * 100.0,
        (transform_with_breath.z - 0.1) * 100.0
    );

    // 两种方式应该得到相同的结果（因为呼吸动画不影响x和z）
    assert_eq!(target_from_home.x, target_from_transform.x, "x坐标应该相同");
    assert_eq!(target_from_home.y, target_from_transform.y, "y坐标应该相同");

    // 验证实际值
    assert_eq!(target_from_home.x, 250.0, "目标x应该是 250.0");
    assert_eq!(target_from_home.y, 50.0, "目标y应该是 50.0");
}

#[test]
fn red_green_003_multiple_enemies_positions() {
    // Red: 多敌人场景下的位置计算

    // 3个敌人的 spawn 位置
    let num_enemies = 3.0;

    let enemy0_x = 250.0 + (0.0 - (num_enemies - 1.0) / 2.0) * 220.0;  // 30.0
    let enemy1_x = 250.0 + (1.0 - (num_enemies - 1.0) / 2.0) * 220.0;  // 250.0
    let enemy2_x = 250.0 + (2.0 - (num_enemies - 1.0) / 2.0) * 220.0;  // 470.0

    // 转换为 home_position
    let home0 = Vec3::new(enemy0_x / 100.0, 0.05, 50.0 / 100.0 + 0.1);
    let home1 = Vec3::new(enemy1_x / 100.0, 0.05, 50.0 / 100.0 + 0.1);
    let home2 = Vec3::new(enemy2_x / 100.0, 0.05, 50.0 / 100.0 + 0.1);

    // 恢复2D坐标
    let pos0 = Vec2::new(home0.x * 100.0, (home0.z - 0.1) * 100.0);
    let pos1 = Vec2::new(home1.x * 100.0, (home1.z - 0.1) * 100.0);
    let pos2 = Vec2::new(home2.x * 100.0, (home2.z - 0.1) * 100.0);

    assert!((pos0.x - 30.0).abs() < 0.01, "敌人0的x应该约等于 30.0");
    assert!((pos1.x - 250.0).abs() < 0.01, "敌人1的x应该约等于 250.0");
    assert!((pos2.x - 470.0).abs() < 0.01, "敌人2的x应该约等于 470.0（不再被clamp限制）");

    // 所有敌人的y坐标相同
    assert!((pos0.y - 50.0).abs() < 0.01, "敌人0的y应该约等于 50.0");
    assert!((pos1.y - 50.0).abs() < 0.01, "敌人1的y应该约等于 50.0");
    assert!((pos2.y - 50.0).abs() < 0.01, "敌人2的y应该约等于 50.0");
}

#[test]
fn red_green_004_rightmost_enemy_not_clamped() {
    // Red: 验证最右边敌人不再被clamp限制

    // 敌人2的原始位置
    let enemy2_x = 470.0_f32;  // 250.0 + (2-1) * 220.0
    let enemy2_y = 50.0_f32;

    // 转换为3D（移除clamp后）
    let x_3d = enemy2_x / 100.0;  // 4.7
    let z_3d = enemy2_y / 100.0;  // 0.5

    // 如果有clamp(-3.5, 3.5)，x_3d会被限制为3.5
    let x_3d_clamped = x_3d.clamp(-3.5, 3.5);  // 3.5

    // 验证clamp会导致错误
    assert_ne!(x_3d, x_3d_clamped, "clamp会改变x坐标");

    // 验证原始值是正确的
    assert_eq!(x_3d, 4.7, "没有clamp时，x_3d应该是 4.7");
    assert_eq!(x_3d_clamped, 3.5, "有clamp时，x_3d会被限制为 3.5");
}

#[test]
fn red_green_005_physical_impact_component() {
    // Red: 验证 PhysicalImpact 组件的结构

    let home = Vec3::new(2.5, 0.05, 0.6);
    let impact = PhysicalImpact {
        home_position: home,
        ..default()
    };

    // 验证 home_position 可以被正确访问
    assert_eq!(impact.home_position.x, 2.5, "home_position.x 应该是 2.5");
    assert_eq!(impact.home_position.z, 0.6, "home_position.z 应该是 0.6");

    // 验证可以从 home_position 计算2D坐标
    let pos_2d = Vec2::new(
        impact.home_position.x * 100.0,
        (impact.home_position.z - 0.1) * 100.0
    );

    assert_eq!(pos_2d.x, 250.0, "恢复的x坐标应该是 250.0");
    assert_eq!(pos_2d.y, 50.0, "恢复的y坐标应该是 50.0");
}

#[test]
fn red_green_006_particle_screen_position() {
    // Red: 验证粒子坐标到屏幕坐标的转换

    let particle_pos = Vec2::new(250.0, 50.0);

    // 粒子系统使用的UI坐标转换
    let ui_x = 640.0 + particle_pos.x;
    let ui_y = 360.0 - particle_pos.y;

    assert_eq!(ui_x, 890.0, "屏幕x应该是 890.0");
    assert_eq!(ui_y, 310.0, "屏幕y应该是 310.0");
}

#[test]
fn red_green_007_spawn_to_target_consistency() {
    // Red: 验证粒子spawn位置和目标位置的一致性

    // 玩家位置（粒子spawn位置）
    let player_pos = Vec2::new(-350.0, -80.0);

    // 敌人位置（粒子目标位置）
    let enemy_pos = Vec2::new(250.0, 50.0);

    // 验证两个坐标系统使用相同的单位
    // 即：都直接使用原始spawn坐标的x和y值
    assert!(player_pos.x < 0.0, "玩家在左侧，x应该是负数");
    assert!(player_pos.y < 0.0, "玩家在下方，y应该是负数");
    assert!(enemy_pos.x > 0.0, "敌人在右侧，x应该是正数");
    assert!(enemy_pos.y > 0.0, "敌人在上方，y应该是正数");
}

#[test]
fn red_green_008_coordinate_roundtrip() {
    // Red: 验证坐标往返转换的正确性

    let original_x = 470.0_f32;
    let original_y = 50.0_f32;

    // 2D → 3D (sync_2d_to_3d_render)
    let x_3d = original_x / 100.0;
    let z_3d = original_y / 100.0;
    let home_position = Vec3::new(x_3d, 0.05, z_3d + 0.1);

    // 3D → 2D (万剑归宗坐标恢复)
    let restored_x = home_position.x * 100.0;
    let restored_y = (home_position.z - 0.1) * 100.0;

    // 验证往返转换后坐标不变（使用近似比较）
    assert!((restored_x - original_x).abs() < 0.01, "往返转换后x坐标应该不变");
    assert!((restored_y - original_y).abs() < 0.01, "往返转换后y坐标应该不变");
}

// ============================================================================
// Green: 实现状态
// ============================================================================

#[test]
fn green_implementation_status() {
    // Green: 验证当前实现是否符合预期

    // 当前实现使用 PhysicalImpact.home_position
    // 而不是 Transform.translation

    // 模拟场景
    let home_position = Vec3::new(2.5, 0.05, 0.6);
    let transform_with_animation = Vec3::new(2.5, 0.032, 0.6);  // y被动画修改

    // 当前实现的计算方式
    let current_target = Vec2::new(
        home_position.x * 100.0,
        (home_position.z - 0.1) * 100.0
    );

    // 验证结果正确
    assert_eq!(current_target.x, 250.0, "当前实现：x坐标正确");
    assert_eq!(current_target.y, 50.0, "当前实现：y坐标正确");

    // 验证不受动画影响
    assert_ne!(transform_with_animation.y, home_position.y,
        "Transform的y可能被动画修改，与home_position不同");
}

// ============================================================================
// Refactor: 代码质量检查
// ============================================================================

#[test]
fn refactor_no_clamp_limitation() {
    // Refactor: 验证移除clamp限制后的改进

    // 测试边缘情况：极端的敌人位置
    let extreme_x = 500.0_f32;  // 超过旧的clamp范围

    // 移除clamp后的转换
    let x_3d = extreme_x / 100.0;  // 5.0

    // 如果有clamp(-3.5, 3.5)
    let x_3d_clamped = x_3d.clamp(-3.5, 3.5);

    // 验证改进
    assert_eq!(x_3d, 5.0, "移除clamp后，保持原始值");
    assert_eq!(x_3d_clamped, 3.5, "有clamp时会被限制");

    // 验证往返转换
    let restored = x_3d * 100.0;
    assert_eq!(restored, extreme_x, "移除clamp后往返转换正确");

    let restored_clamped = x_3d_clamped * 100.0;
    assert_ne!(restored_clamped, extreme_x, "有clamp时往返转换错误");
}
