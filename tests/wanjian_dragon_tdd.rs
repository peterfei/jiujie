use bevy::prelude::*;

#[test]
fn test_wanjian_dragon_phases() {
    let lifetime = 2.5f32;
    let seed = 0.1f32; // 第一把剑
    
    // 模拟 20% 进度
    let elapsed_20 = 0.5f32;
    let prog_20 = (elapsed_20 / lifetime * 1.5 - seed * 0.5).clamp(0.0, 1.0);
    let phase_20 = if prog_20 < 0.3 { "Rise" } else if prog_20 < 0.6 { "Gather" } else { "Strike" };
    assert_eq!(phase_20, "Rise", "早期应处于飞升阶段");

    // 模拟 80% 进度
    let elapsed_80 = 2.0f32;
    let prog_80 = (elapsed_80 / lifetime * 1.5 - seed * 0.5).clamp(0.0, 1.0);
    assert!(prog_80 > 0.6, "后期应进入游龙打击阶段");
}

#[test]
fn test_dragon_curve_math() {
    // 模拟贝塞尔曲线游走：P0(起点), P1(控制点), P2(终点)
    let p0 = Vec2::new(0.0, 300.0); // 半空集结地
    let p1 = Vec2::new(300.0, 500.0); // 龙身游走的最高点
    let p2 = Vec2::new(600.0, 0.0);   // 敌阵
    
    let t = 0.5f32;
    // 二阶贝塞尔公式：(1-t)^2*P0 + 2t(1-t)*P1 + t^2*P2
    let pos = (1.0-t).powi(2)*p0 + 2.0*t*(1.0-t)*p1 + t.powi(2)*p2;
    
    assert!(pos.y > 0.0 && pos.x > 0.0, "曲线坐标计算应正确");
}
