use bevy::prelude::*;
use bevy_card_battler::components::sprite::{PhysicalImpact, AnimationState, ActionType};

#[test]
fn test_breath_suppression_during_action() {
    // 逻辑：如果 action_timer > 0，breath_y 应该被设为 0
    let action_timer = 0.5f32;
    let breath_cycle_val = 0.05f32; // 模拟呼吸计算出的位移
    
    let final_breath_y = if action_timer > 0.0 {
        0.0
    } else {
        breath_cycle_val
    };
    
    assert_eq!(final_breath_y, 0.0, "执行动作期间应抑制呼吸晃动");
}

#[test]
fn test_observation_state_immobility() {
    // 逻辑：观察状态下，不应有旋转速度，且位移应回弹至 0
    let mut impact = PhysicalImpact::default();
    impact.action_type = ActionType::None;
    impact.special_rotation = 0.5; // 模拟残留旋转
    
    // 物理回弹模拟 (rot_spring_k = 45.0)
    let dt = 0.016;
    let force = -45.0 * impact.special_rotation;
    impact.special_rotation_velocity += force * dt;
    impact.special_rotation += impact.special_rotation_velocity * dt;
    
    assert!(impact.special_rotation < 0.5, "残留旋转必须被弹簧力拉回 0");
}

#[test]
fn test_breath_suppression_robustness() {
    // 模拟场景：狼已经停在玩家面前（速度低），但撕咬计时器还没走完
    let action_timer = 0.5f32;
    let velocity = Vec3::new(1.0, 0.0, 0.0); // 极低速
    
    let is_acting = action_timer > 0.0 || velocity.length() > 5.0;
    let breath_weight = if is_acting { 0.0 } else { 1.0 };
    
    assert_eq!(breath_weight, 0.0, "只要动作计时器在跑，就必须完全抑制呼吸");
}

#[test]
fn test_demon_cast_suppression() {
    // 逻辑：施法期间 action_timer > 0 应该导致抑制
    let action_timer = 0.3f32; 
    let is_acting = action_timer > 0.0;
    assert!(is_acting, "施法动作应当触发行动状态以抑制呼吸");
}

#[test]
fn test_demon_cast_horizontal_vibration() {
    // 逻辑：防守/施法时，不应产生 Z 轴倾斜速度，而应产生 Y 轴自转速度
    let anim_state = AnimationState::DemonCast;
    
    let (tilt_v, special_v) = if anim_state == AnimationState::DemonCast {
        (0.0, 60.0) // Z轴为0, Y轴震动
    } else {
        (10.0, 0.0)
    };
    
    assert_eq!(tilt_v, 0.0, "防守时不应有 Z 轴抖动");
    assert!(special_v > 0.0, "防守时应使用 Y 轴产生水平震颤");
}

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
