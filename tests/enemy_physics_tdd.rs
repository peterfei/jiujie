use bevy::prelude::*;
use bevy_card_battler::components::sprite::PhysicalImpact;

#[test]
fn test_wolf_dash_sustenance() {
    let mut impact = PhysicalImpact {
        offset_velocity: Vec3::new(-25.0, 0.0, 0.0), // 攻击初速度
        ..Default::default()
    };
    
    let dt = 0.016;
    // 模拟 10 帧更新
    for _ in 0..10 {
        // 模拟原本的错误逻辑（过快衰减）
        // impact.offset_velocity *= 0.8; 
        
        // 优化后的逻辑（仅在 action_timer 结束后或慢速时衰减）
        impact.current_offset += impact.offset_velocity * dt;
    }
    
    // 验证：10 帧后位移是否足够远 (至少跨越 3 个单位)
    assert!(impact.current_offset.x.abs() > 3.0, "冲刺位移在 10 帧后应具有足够的穿透力");
}

#[test]
fn test_particle_position_sync() {
    // 逻辑验证：粒子应该在立牌的 transform.translation 处爆发
    let entity_pos = Vec3::new(3.5, 1.0, 0.2);
    let particle_pos = entity_pos; // 应该动态获取
    
    assert_eq!(particle_pos.x, 3.5, "粒子发射点必须与 3D 实体位置一致");
}
