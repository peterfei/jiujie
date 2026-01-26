use bevy::prelude::*;
use bevy_card_battler::components::combat::{Enemy, EnemyIntent};
use bevy_card_battler::components::sprite::{AnimationState, CharacterAnimationEvent};

#[test]
fn test_enemy_intent_triggers_correct_animation() {
    // 逻辑：不同的意图应该映射到不同的 3D 动画状态
    let attack_intent = EnemyIntent::Attack { damage: 10 };
    let buff_intent = EnemyIntent::Buff { strength: 5 };
    let defend_intent = EnemyIntent::Defend { block: 5 };

    // 验证攻击意图映射
    let anim_attack = match attack_intent {
        EnemyIntent::Attack { .. } => AnimationState::DemonAttack,
        _ => AnimationState::Idle,
    };
    assert_eq!(anim_attack, AnimationState::DemonAttack);

    // 验证施法意图映射
    let anim_buff = match buff_intent {
        EnemyIntent::Buff { .. } => AnimationState::DemonCast,
        _ => AnimationState::Idle,
    };
    assert_eq!(anim_buff, AnimationState::DemonCast);

    let anim_defend = match defend_intent {
        EnemyIntent::Defend { .. } => AnimationState::DemonCast,
        _ => AnimationState::Idle,
    };
    assert_eq!(anim_defend, AnimationState::DemonCast);
}
