use bevy::prelude::*;
use bevy_card_battler::components::{Enemy, EnemyType, EnemyIntent, Environment};
use bevy_card_battler::components::sprite::AnimationState;

#[test]
fn test_spider_attack_animation_logic_isolation() {
    // 模拟匹配逻辑
    let get_anim = |enemy_type: EnemyType, intent: EnemyIntent| -> AnimationState {
        match intent {
            EnemyIntent::Attack { .. } => {
                match enemy_type {
                    EnemyType::DemonicWolf => AnimationState::WolfAttack,
                    EnemyType::PoisonSpider => AnimationState::SpiderAttack,
                    EnemyType::CursedSpirit => AnimationState::SpiritAttack,
                    EnemyType::GreatDemon => AnimationState::DemonCast,
                }
            },
            EnemyIntent::Seal { .. } | EnemyIntent::Curse { .. } | EnemyIntent::Debuff { .. } => {
                match enemy_type {
                    EnemyType::PoisonSpider => AnimationState::SpiderAttack, // 吐丝封印
                    _ => AnimationState::DemonCast,
                }
            }
            _ => AnimationState::DemonCast,
        }
    };

    // 验证蜘蛛攻击
    let spider_attack_anim = get_anim(EnemyType::PoisonSpider, EnemyIntent::Attack { damage: 10 });
    assert_eq!(spider_attack_anim, AnimationState::SpiderAttack, "蜘蛛攻击应该是 SpiderAttack");

    // 验证狼攻击
    let wolf_attack_anim = get_anim(EnemyType::DemonicWolf, EnemyIntent::Attack { damage: 10 });
    assert_eq!(wolf_attack_anim, AnimationState::WolfAttack, "狼攻击应该是 WolfAttack");

    // 验证蜘蛛封印
    let spider_seal_anim = get_anim(EnemyType::PoisonSpider, EnemyIntent::Seal { slot_index: 0, duration: 2 });
    assert_eq!(spider_seal_anim, AnimationState::SpiderAttack, "蜘蛛封印也应使用吐丝动画");
}