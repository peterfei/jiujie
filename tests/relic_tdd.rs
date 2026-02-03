//! 遗物系统逻辑测试 (TDD)

use bevy::prelude::*;
use bevy_card_battler::components::relic::{Relic, RelicId, RelicEffect, RelicCollection, RelicRarity};

#[test]
fn test_relic_creation() {
    let relic = Relic::burning_blood();
    assert_eq!(relic.id, RelicId::BurningBlood);
    assert_eq!(relic.name, "飞剑符");
    assert!(!relic.effects.is_empty());
}

#[test]
fn test_relic_collection_management() {
    let mut collection = RelicCollection::default();
    assert!(collection.is_empty());

    let relic = Relic::burning_blood();
    collection.add_relic_forced(relic);
    assert_eq!(collection.count(), 1);
    assert!(collection.has(RelicId::BurningBlood));
}

#[test]
fn test_burning_blood_damage_value() {
    let relic = Relic::burning_blood();
    match &relic.effects[0] {
        RelicEffect::OnCombatStart { damage, .. } => {
            assert_eq!(*damage, 3, "飞剑符初始伤害应为3");
        }
        _ => panic!("飞剑符效果不正确"),
    }
}

#[test]
fn test_bag_of_preparation_draw_value() {
    let relic = Relic::bag_of_preparation();
    match &relic.effects[0] {
        RelicEffect::OnCombatStart { draw_cards, .. } => {
            assert_eq!(*draw_cards, 1, "乾坤袋初始抽牌应为1");
        }
        _ => panic!("乾坤袋效果不正确"),
    }
}

#[test]
fn test_anchor_block_value() {
    let relic = Relic::anchor();
    match &relic.effects[0] {
        RelicEffect::OnTurnEnd { keep_cards } => {
            assert_eq!(*keep_cards, 3, "定风珠应保留3张牌");
        }
        _ => panic!("定风珠效果不正确"),
    }
}

#[test]
fn test_strange_spoon_trigger_value() {
    let relic = Relic::strange_spoon();
    match &relic.effects[0] {
        RelicEffect::OnCardPlayed { every_nth, draw_cards } => {
            assert_eq!(*every_nth, 3);
            assert_eq!(*draw_cards, 1);
        }
        _ => panic!("聚灵阵效果不正确"),
    }
}

#[test]
fn test_custom_relic_id() {
    let relic = Relic {
        id: RelicId::Custom(999),
        name: "神秘古玉".to_string(),
        description: "未知效果".to_string(),
        rarity: RelicRarity::Special,
        effects: vec![],
    };
    
    match relic.id {
        RelicId::Custom(val) => assert_eq!(val, 999),
        _ => panic!("应为自定义ID"),
    }
}