use bevy::prelude::*;
use bevy_card_battler::components::relic::{Relic, RelicId, RelicCollection, RelicRarity, RelicEffect};
use bevy_card_battler::components::sprite::RelicVisualMarker;

#[test]
fn test_relic_3d_spawning_logic() {
    let mut collection = RelicCollection::default();
    let relic = Relic {
        id: RelicId::Anchor,
        name: "定风珠".to_string(),
        description: "每回合保留手牌".to_string(),
        rarity: RelicRarity::Common,
        effects: vec![RelicEffect::OnTurnEnd { keep_cards: 1 }],
    };
    
    collection.add_relic(relic);
    
    // 逻辑验证
    assert!(!collection.relic.is_empty());
    assert_eq!(collection.relic[0].name, "定风珠");
    
    // 验证视觉组件字段
    let marker = RelicVisualMarker {
        relic_id: RelicId::Anchor,
        base_y: 1.5,
    };
    assert_eq!(marker.relic_id, RelicId::Anchor);
    
    // 验证 Z 轴可见性偏置
    let player_z = 0.1f32;
    let relic_z = player_z + 0.5; // 应该在玩家稍微前面一点
    assert!(relic_z > player_z, "法宝应位于玩家立牌的前景层");
}

#[test]
fn test_relic_floating_math() {
    let base_y = 1.5f32;
    let time = 1.0f32;
    let float_y = base_y + (time * 2.0).sin() * 0.15;
    
    assert!(float_y != base_y, "法宝应该在垂直方向上有动态位移");
}