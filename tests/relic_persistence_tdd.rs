use bevy::prelude::*;
use bevy_card_battler::components::relic::{Relic, RelicCollection, RelicId, RelicRarity, RelicEffect};

#[test]
fn test_relic_accumulation_persistence() {
    let mut app = App::new();
    // 1. 模拟初始状态：拥有 1 个法宝
    let mut collection = RelicCollection::default();
    collection.add_relic(Relic::burning_blood());
    app.insert_resource(collection);
    
    // 2. 模拟在奖励界面点击获取了第二个法宝
    {
        let mut relics = app.world_mut().resource_mut::<RelicCollection>();
        relics.add_relic(Relic {
            id: RelicId::Anchor,
            name: "定风珠".to_string(),
            description: "测试".to_string(),
            rarity: RelicRarity::Common,
            effect: RelicEffect::OnTurnEnd { keep_cards: 1 },
        });
    }
    
    // 3. 验证：资源中现在应有 2 个法宝
    let final_collection = app.world().resource::<RelicCollection>();
    assert_eq!(final_collection.count(), 2, "法宝数量应累加，不应被重置或覆盖");
    assert!(final_collection.has(RelicId::BurningBlood));
    assert!(final_collection.has(RelicId::Anchor));
}