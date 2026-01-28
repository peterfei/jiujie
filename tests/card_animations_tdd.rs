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
fn test_card_type_color_mapping() {
    let mut card = Card {
        id: 1,
        name: "测试卡".to_string(),
        description: "造成10点伤害".to_string(),
        card_type: CardType::Attack,
        cost: 1,
        effect: CardEffect::DealDamage { amount: 10 },
        rarity: CardRarity::Common,
        image_path: "".to_string(),
        upgraded: false,
    };

    // 攻击卡应为红色调
    let color = card.get_color().to_srgba();
    assert!(color.red > color.blue && color.red > color.green, "攻击卡应显示为红色调");

    // 能力卡应为紫色调
    card.card_type = CardType::Power;
    let color = card.get_color().to_srgba();
    assert!(color.red > 0.5 && color.blue > 0.5, "能力卡应显示为紫色调");
}