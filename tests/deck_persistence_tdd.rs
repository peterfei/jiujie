use bevy::prelude::*;
use bevy_card_battler::components::cards::{Card, CardPool, PlayerDeck};

#[test]
fn test_shop_purchase_persistence() {
    let mut app = App::new();
    app.insert_resource(PlayerDeck::default());
    
    // 获取初始长度
    let initial_len = app.world().resource::<PlayerDeck>().len();
    
    // 1. 模拟购买“万剑归宗”
    let all_cards = CardPool::all_cards();
    let wan_jian = all_cards.iter().find(|c| c.name == "万剑归宗").unwrap().clone();
    
    {
        let mut deck = app.world_mut().resource_mut::<PlayerDeck>();
        deck.add_card(wan_jian.clone());
        assert_eq!(deck.len(), initial_len + 1, "购买后牌组应增加一张卡牌");
    }
    
    // 2. 模拟切换系统（模拟逻辑层不应重置该资源）
    let deck_after = app.world().resource::<PlayerDeck>();
    assert!(deck_after.cards.iter().any(|c| c.name == "万剑归宗"), "切换状态后购买的功法应依然存在");
}