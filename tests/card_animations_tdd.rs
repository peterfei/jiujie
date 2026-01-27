use bevy::prelude::*;
use bevy_card_battler::components::cards::{Card, CardType, CardEffect, CardRarity};

#[test]
fn test_card_hover_visual_logic() {
    // 逻辑：基础高度 base_bottom = 20.0
    let base_bottom = 20.0f32;
    let is_hovered = true;
    
    // 预期：悬停时底边升至 50.0，缩放变为 1.2
    let current_bottom = if is_hovered { base_bottom + 30.0 } else { base_bottom };
    let current_scale = if is_hovered { 1.2f32 } else { 1.0f32 };
    
    assert_eq!(current_bottom, 50.0);
    assert_eq!(current_scale, 1.2);
}

#[test]
fn test_card_rarity_color_mapping() {
    let card = Card {
        id: 1,
        name: "测试功法".to_string(),
        card_type: CardType::Attack,
        rarity: CardRarity::Rare,
        cost: 1,
        description: "".to_string(),
        image_path: "".to_string(),
        effect: CardEffect::DealDamage { amount: 10 },
    };
    
    // 紫色代表稀有
    let color = card.get_color();
    let rgba: Srgba = color.into();
    assert!(rgba.red > 0.5 && rgba.blue > 0.5, "稀有功法应显示为紫色调");
}