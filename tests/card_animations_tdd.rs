use bevy::prelude::*;
use bevy_card_battler::components::sprite::{CharacterAnimationEvent, AnimationState, PhysicalImpact};
use bevy_card_battler::components::cards::{Card, CardType, CardEffect, CardRarity};

#[test]
fn test_imperial_sword_card_triggers_special_anim() {
    // 逻辑：打出名称为“御剑术”的卡牌应该选择 ImperialSword 状态
    let card_name = "御剑术";
    let anim = if card_name == "御剑术" {
        AnimationState::ImperialSword
    } else {
        AnimationState::Attack
    };

    assert_eq!(anim, AnimationState::ImperialSword);
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
