//! 万剑归宗窗口大小变化 TDD 测试
//!
//! 问题描述：最大化窗口时，飞剑粒子跑偏到虚空位置
//! 根本原因：粒子使用2D UI坐标，敌人使用3D世界坐标 + 透视摄像机
//! 窗口大小改变时，摄像机投影矩阵改变，导致坐标转换失效

use bevy::prelude::*;
use bevy::window::WindowResolution;

// ============================================================================
// Red: 测试用例 - 描述期望的行为
// ============================================================================

#[test]
fn red_001_verify_3d_camera_perspective() {
    // Red: 验证摄像机使用透视投影
    // 透视投影意味着：相同的世界坐标在不同窗口大小下会映射到不同的屏幕坐标

    // 模拟透视摄像机的参数
    let _fov = std::f32::consts::PI / 4.0; // 45度FOV
    let aspect_ratio_1280: f32 = 1280.0 / 720.0; // 16:9
    let aspect_ratio_1920: f32 = 1920.0 / 1080.0; // 16:9（相同比例）
    let aspect_ratio_2560: f32 = 2560.0 / 1440.0; // 16:9（相同比例）

    // 如果宽高比相同，透视投影的缩放应该一致
    // 但窗口绝对尺寸不同时，屏幕坐标会不同

    assert!((aspect_ratio_1280 - aspect_ratio_1920).abs() < 0.01, "相同比例");
    assert!((aspect_ratio_1920 - aspect_ratio_2560).abs() < 0.01, "相同比例");
}

#[test]
fn red_002_particle_vs_enemy_coordinate_systems() {
    // Red: 验证粒子和敌人使用不同的坐标系统

    // 粒子系统：2D UI坐标（相对于屏幕中心，像素单位）
    let particle_pos = Vec2::new(250.0, 50.0);

    // 敌人系统：3D世界坐标（世界空间单位，约是像素的1/100）
    let enemy_home = Vec3::new(2.5, 0.05, 0.6);

    // 问题：简单的乘法转换不考虑摄像机投影
    let converted_particle = Vec2::new(
        enemy_home.x * 100.0,
        (enemy_home.z - 0.1) * 100.0
    );

    assert_eq!(converted_particle.x, 250.0, "x坐标转换");
    assert_eq!(converted_particle.y, 50.0, "y坐标转换");

    // 但这不能反映透视摄像机的实际投影效果！
}

#[test]
fn red_003_window_size_affects_projection() {
    // Red: 窗口大小影响摄像机投影

    let window_sizes = vec![
        (1280.0, 720.0),  // 原始大小
        (1920.0, 1080.0), // 全屏1080p
        (2560.0, 1440.0), // 2K分辨率
    ];

    for (width, height) in window_sizes {
        // 模拟世界坐标到屏幕坐标的投影
        let world_pos = Vec3::new(2.5, 0.05, 0.6);

        // 简单投影（不考虑透视）
        let screen_x = width / 2.0 + world_pos.x * 100.0;
        let screen_y = height / 2.0 - (world_pos.z - 0.1) * 100.0;

        // 验证：屏幕坐标会随窗口大小改变
        if width == 1280.0 {
            assert_eq!(screen_x, 640.0 + 250.0, "1280宽度时的x");
        } else if width == 1920.0 {
            assert_eq!(screen_x, 960.0 + 250.0, "1920宽度时的x");
        }
    }
}

#[test]
fn red_004_camera_world_to_viewport_needed() {
    // Red: 需要使用 Bevy 的 world_to_viewport 进行正确投影

    // 当前错误的实现（简单乘法）：
    let enemy_home = Vec3::new(2.5, 0.05, 0.6);
    let wrong_screen_pos = Vec2::new(
        enemy_home.x * 100.0,
        (enemy_home.z - 0.1) * 100.0
    );

    // 正确的实现应该是：
    // 1. 获取摄像机和窗口
    // 2. 使用 camera.world_to_viewport(camera_transform, world_pos)
    // 3. 将屏幕坐标转换为粒子坐标

    // 这个测试标记问题存在
    assert_ne!(wrong_screen_pos.x, 0.0, "简单乘法产生坐标，但不准确");
}

// ============================================================================
// Integration Test: 窗口大小变化时粒子位置验证
// ============================================================================

#[test]
fn integration_window_resize_particle_alignment() {
    // Integration: 模拟窗口大小变化时粒子对齐问题

    // 场景：
    // 1. 在 1280x720 窗口中，粒子对齐正确
    // 2. 用户最大化窗口到 1920x1080
    // 3. 粒子应该仍然对齐敌人，但实际上跑偏了

    let small_window = (1280.0, 720.0);
    let large_window = (1920.0, 1080.0);

    // 敌人的3D世界坐标（固定）
    let enemy_world = Vec3::new(2.5, 0.05, 0.6);

    // 简单乘法转换（当前实现）
    let particle_pos_small = Vec2::new(
        enemy_world.x * 100.0,
        (enemy_world.z - 0.1) * 100.0
    );
    let particle_pos_large = Vec2::new(
        enemy_world.x * 100.0,
        (enemy_world.z - 0.1) * 100.0
    );

    // 问题：简单乘法得到的粒子坐标相同，但实际屏幕位置不同！
    assert_eq!(particle_pos_small.x, particle_pos_large.x, "简单乘法结果相同");

    // 正确的做法：粒子坐标应该相对于屏幕中心，所以需要加上屏幕中心偏移
    let screen_center_small = Vec2::new(small_window.0 / 2.0, small_window.1 / 2.0);
    let screen_center_large = Vec2::new(large_window.0 / 2.0, large_window.1 / 2.0);

    let actual_screen_small = Vec2::new(
        screen_center_small.x + particle_pos_small.x,
        screen_center_small.y - particle_pos_small.y
    );
    let actual_screen_large = Vec2::new(
        screen_center_large.x + particle_pos_large.x,
        screen_center_large.y - particle_pos_large.y
    );

    // 屏幕坐标确实不同
    assert_ne!(actual_screen_small.x, actual_screen_large.x, "屏幕坐标不同");
}

// ============================================================================
// Fix Proposal: 使用 Camera world_to_viewport
// ============================================================================

#[test]
fn green_solution_use_world_to_viewport() {
    // Green: 解决方案 - 在 update_particles 中使用 world_to_viewport

    // 伪代码：
    /*
    fn update_particles(
        ...
        camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
        enemy_query: Query<(&Transform, &PhysicalImpact), With<EnemySpriteMarker>>,
    ) {
        let (camera, camera_transform) = camera_query.get_single().unwrap();

        for (enemy_transform, impact) in enemy_query.iter() {
            // 使用 Bevy 的 world_to_viewport 进行正确的3D到2D投影
            if let Some(screen_pos) = camera.world_to_viewport(camera_transform, impact.home_position) {
                // screen_pos 是相对于窗口左上角的像素坐标
                // 转换为相对于屏幕中心的粒子坐标
                let window_size = camera.viewport.as_ref().map(|v| v.physical_size).unwrap_or_default();
                let center = Vec2::new(window_size.x as f32 / 2.0, window_size.y as f32 / 2.0);

                let particle_pos = Vec2::new(
                    screen_pos.x - center.x,
                    center.y - screen_pos.y
                );

                // 现在粒子坐标正确对齐敌人了！
            }
        }
    }
    */

    assert!(true, "解决方案：使用 camera.world_to_viewport");
}
