use bevy::prelude::*;

#[test]
fn test_wanjian_staggered_progress() {
    let lifetime = 2.0f32;
    let elapsed = 0.5f32;
    let seed = 0.5f32; // 代表第 15 把剑左右
    
    // 计算错位后的局部进度
    // 预期：seed 越大，启动越晚
    let global_prog = elapsed / lifetime;
    let local_prog = (global_prog * 1.5 - seed * 0.5).clamp(0.0, 1.0);
    
    assert!(local_prog < global_prog, "后续飞剑应比首把剑启动更晚");
}

#[test]
fn test_wanjian_converge_acceleration() {
    let mut speed = 0.0f32;
    let delta = 0.016f32;
    let mut pos = 0.0f32;
    
    // 模拟加速度阶段 (70% 之后)
    for t in 0..10 {
        let t_f = t as f32 * 0.1;
        speed = 2500.0 * t_f * t_f; // 平方级加速度
        pos += speed * delta;
    }
    
    assert!(pos > 100.0, "爆发阶段位移应具有极强的速度感");
}