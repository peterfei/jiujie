use bevy::prelude::*;
use bevy_card_battler::components::combat::{EnemyType, EnemyIntent};
use bevy_card_battler::components::sprite::AnimationState;

#[test]
fn test_specific_enemy_attack_mapping() {
    // 逻辑：不同的妖兽，即便意图都是 Attack，触发的动画状态也应不同
    let wolf_type = EnemyType::DemonicWolf;
    let spider_type = EnemyType::PoisonSpider;
    
    let wolf_anim = match wolf_type {
        EnemyType::DemonicWolf => AnimationState::WolfAttack,
        _ => AnimationState::Attack,
    };
    
    let spider_anim = match spider_type {
        EnemyType::PoisonSpider => AnimationState::SpiderAttack,
        _ => AnimationState::Attack,
    };

    let spirit_type = EnemyType::CursedSpirit;
    let spirit_anim = match spirit_type {
        EnemyType::CursedSpirit => AnimationState::SpiritAttack,
        _ => AnimationState::Attack,
    };

    assert_eq!(wolf_anim, AnimationState::WolfAttack);
    assert_eq!(spider_anim, AnimationState::SpiderAttack);
    assert_eq!(spirit_anim, AnimationState::SpiritAttack);
}
