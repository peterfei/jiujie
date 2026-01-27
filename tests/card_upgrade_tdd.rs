use bevy::prelude::*;
use bevy_card_battler::components::cards::{Card, CardType, CardEffect, CardRarity};

#[test]
fn test_card_upgrade_logic() {
    // 1. 创建一张基础卡牌
    let mut card = Card::new(
        1,
        "御剑术",
        "造成6点伤害",
        CardType::Attack,
        1,
        CardEffect::DealDamage { amount: 6 },
        CardRarity::Common,
        "textures/cards/attack.png",
    );

    // 验证初始状态
    assert!(!card.upgraded); 

    // 2. 执行进阶
    card.upgrade();

    // 3. 验证进阶后的变化
    assert_eq!(card.name, "御剑术+");
    assert!(card.upgraded);
    
    // 伤害应该从 6 提升到 9
    match card.effect {
        CardEffect::DealDamage { amount } => assert_eq!(amount, 9),
        _ => panic!("效果类型不匹配"),
    }
    
    // 描述也应该更新
    assert!(card.description.contains("9"));

    println!("✅ 绿灯：单体攻击卡牌进阶逻辑验证通过");
}

#[test]
fn test_aoe_card_upgrade() {
    let mut card = Card::new(
        151,
        "万剑归宗",
        "对所有妖兽造成10点伤害",
        CardType::Attack,
        2,
        CardEffect::DealAoEDamage { amount: 10 },
        CardRarity::Rare,
        "textures/cards/attack.png",
    );

    card.upgrade();
    
    // 验证 AOE 伤害提升 (10 -> 12)
    match card.effect {
        CardEffect::DealAoEDamage { amount } => assert_eq!(amount, 12),
        _ => panic!("效果类型不匹配"),
    }
    
    assert_eq!(card.name, "万剑归宗+");
    println!("✅ 绿灯：AOE卡牌进阶逻辑验证通过");
}

#[test]
fn test_upgrade_idempotency() {
    let mut card = Card::new(
        1,
        "御剑术",
        "造成6点伤害",
        CardType::Attack,
        1,
        CardEffect::DealDamage { amount: 6 },
        CardRarity::Common,
        "textures/cards/attack.png",
    );

    card.upgrade();
    let name_after_first = card.name.clone();
    
    // 再次进阶不应有变化
    card.upgrade();
    assert_eq!(card.name, name_after_first);
    
    match card.effect {
        CardEffect::DealDamage { amount } => assert_eq!(amount, 9),
        _ => panic!(),
    }
    
    println!("✅ 绿灯：进阶幂等性验证通过");
}