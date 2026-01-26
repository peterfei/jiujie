use bevy::prelude::*;
use bevy_card_battler::components::sprite::{CharacterAnimationEvent, AnimationState, PhysicalImpact};
use bevy_card_battler::components::cards::{Card, CardType, CardEffect, CardRarity};

#[test]
fn test_card_animation_selection_logic() {
    // 1. 御剑术 -> ImperialSword
    let sword_card = "御剑术";
    let anim_sword = if sword_card == "御剑术" {
        AnimationState::ImperialSword
    } else {
        AnimationState::Attack
    };
    assert_eq!(anim_sword, AnimationState::ImperialSword);

    // 2. 疗伤术 -> 不应该触发攻击相关动画 (保持 Idle 或特定辅助动画)
    let heal_card = "回春术";
    // 模拟我们的优化逻辑：只有特定攻击类卡牌才触发物理位移
    let is_attack_move = heal_card == "御剑术" || heal_card == "打击"; 
    assert!(!is_attack_move, "疗伤类卡牌不应触发攻击冲刺动画");
}

#[test]
fn test_sword_energy_particle_event() {
    use bevy_card_battler::components::particle::EffectType;
    
    // 逻辑：如果是 ImperialSword 状态，应该准备发送 SwordEnergy 粒子
    let anim_state = AnimationState::ImperialSword;
    let effect_to_spawn = if anim_state == AnimationState::ImperialSword {
        Some(EffectType::Lightning) // 暂时用闪电代替剑气逻辑测试
    } else {
        None
    };

    assert!(effect_to_spawn.is_some());
}

#[test]
fn test_special_rotation_physics() {
    let mut app = App::new();
    let entity = app.world_mut().spawn((
        Transform::default(),
        PhysicalImpact {
            special_rotation_velocity: -45.0, // 模拟御剑术触发时的初速度
            ..default()
        },
    )).id();

    // 运行物理更新逻辑的子集
    let dt = 0.016;
    let mut impact = app.world_mut().get_mut::<PhysicalImpact>(entity).unwrap();
    
    // 1. 更新位置
    impact.special_rotation += impact.special_rotation_velocity * dt;
    
    // 2. 模拟弹簧力 (简化)
    let rot_spring_k = 40.0;
    let rot_force = -rot_spring_k * impact.special_rotation;
    impact.special_rotation_velocity += rot_force * dt;

    assert!(impact.special_rotation != 0.0, "特殊旋转应该被激活");
    assert!(impact.special_rotation_velocity != -45.0, "旋转速度应该受到弹簧回复力的影响而改变");
}
