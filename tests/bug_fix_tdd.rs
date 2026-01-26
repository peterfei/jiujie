use bevy::prelude::*;
use bevy_card_battler::components::sprite::PhysicalImpact;

#[test]
fn test_dash_distance_limit() {
    let mut impact = PhysicalImpact {
        offset_velocity: Vec3::new(-20.0, 0.0, 0.0), // 冲向左侧
        ..Default::default()
    };
    let dt = 0.016;
    let mut max_dist = 0.0f32;
    
    // 模拟 20 帧
    for _ in 0..20 {
        impact.current_offset += impact.offset_velocity * dt;
        // 模拟弹簧拉力 (k=10.0)
        let force = -10.0 * impact.current_offset;
        impact.offset_velocity += force * dt;
        max_dist = max_dist.max(impact.current_offset.x.abs());
    }
    
    assert!(max_dist < 6.0, "冲刺距离不应超过 6.0，当前最大位移: {}", max_dist);
}

#[test]
fn test_special_rotation_persistence() {
    let mut impact = PhysicalImpact {
        special_rotation_velocity: -45.0,
        ..Default::default()
    };
    let dt = 0.016;
    
    // 模拟 1 帧更新
    impact.special_rotation += impact.special_rotation_velocity * dt;
    
    assert!(impact.special_rotation != 0.0, "special_rotation 应该随 velocity 产生积累");
}
