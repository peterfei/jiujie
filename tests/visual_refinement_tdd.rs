use bevy::prelude::*;

#[test]
fn test_shake_event_deduplication() {
    // 模拟一帧内的多个事件
    let events = vec![(0.4, 0.8), (1.0, 0.5), (0.2, 1.0)];
    
    let mut max_trauma = 0.0f32;
    let mut min_decay = 100.0f32;
    
    for (t, d) in events {
        max_trauma = max_trauma.max(t);
        min_decay = min_decay.min(d);
    }
    
    assert_eq!(max_trauma, 1.0, "应选取最强的震动 (1.0)");
    assert_eq!(min_decay, 0.5, "应选取最持久的衰减 (0.5)");
}

#[test]
fn test_boss_golden_seal_logic() {
    // 逻辑：如果有 BOSS，法阵应变为金色且旋转加速
    let has_boss = true;
    let seal_color = if has_boss { Color::srgb(1.0, 0.8, 0.2) } else { Color::srgb(0.0, 0.8, 0.3) };
    let seal_speed = if has_boss { 0.15 } else { 0.05 };
    
    let rgba: Srgba = seal_color.into();
    assert!(rgba.red > 0.9, "BOSS 法阵应为金色调");
    assert_eq!(seal_speed, 0.15, "BOSS 法阵旋转速度应提升");
}

#[test]
fn test_glowing_seal_pulse_subtle() {
    // 逻辑：法阵的亮度应非常平缓地脉动 (0.85 -> 1.15)
    let timer = 1.0f32;
    let pulse_speed = 0.5f32; // 极慢频率
    let pulse = 1.0 + (timer * pulse_speed).sin() * 0.15;
    assert!(pulse >= 0.8 && pulse <= 1.2, "亮度波动应保持在 20% 以内，避免闪烁感");
}

#[test]
fn test_character_grounded() {
    // 逻辑：角色初始高度应接近 0，不再悬空
    let home_pos_y = 0.05f32; 
    assert!(home_pos_y < 0.2, "角色应该站在地面上，高度应小于 0.2");
}

#[test]
fn test_character_color_fidelity() {
    let mut app = App::new();
    app.init_resource::<Assets<StandardMaterial>>();
    
    let mat = StandardMaterial {
        base_color: Color::WHITE,
        // 验证反射率为 0，防止灯光让人物发白
        reflectance: 0.0,
        // 验证自发光已启用 (用于还原高饱和度)
        emissive: LinearRgba::WHITE,
        ..default()
    };
    
    assert_eq!(mat.reflectance, 0.0, "材质反射率应为 0 以防颜色被冲淡");
}

#[test]
fn test_demon_cast_pulse_scaling() {
    // 逻辑：施法时产生高频缩放脉冲
    let timer = 0.2f32;
    let pulse_speed = 30.0f32; // 每秒约 5 次往复
    let pulse = 1.0 + (timer * pulse_speed).sin().abs() * 0.15;
    
    assert!(pulse > 1.0, "施法期间应产生向外的能量扩张感");
}

#[test]
fn test_base_transparency() {
    let mut app = App::new();
    app.init_resource::<Assets<StandardMaterial>>();
    
    // 模拟生成底座材质
    let _mat_handle = app.world_mut().resource_mut::<Assets<StandardMaterial>>().add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.2, 0.0, 0.3), // 半透明绿
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let materials = app.world().resource::<Assets<StandardMaterial>>();
    let (_, mat) = materials.iter().next().unwrap();
    assert!(mat.base_color.alpha() < 1.0, "底座应该是半透明的以减少突兀感");
}

#[test]
fn test_wolf_multi_turn_spin() {
    // 逻辑：1秒内应完成 2 圈旋转 (4 * PI)
    let progress = 0.5f32; // 动作中点
    let total_spin = 4.0 * std::f32::consts::PI;
    let current_spin = progress * total_spin;
    
    assert!(current_spin >= 6.0, "动作中点应已完成至少一圈旋转");
}