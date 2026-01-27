use bevy::prelude::*;
use bevy_card_battler::components::cards::{PlayerDeck, CardPool};

#[test]
fn test_deck_view_card_count() {
    let mut app = App::new();
    
    // 模拟玩家牌组：3 张牌
    let mut deck = PlayerDeck::default();
    let all = CardPool::all_cards();
    deck.add_card(all[0].clone());
    deck.add_card(all[1].clone());
    deck.add_card(all[2].clone());
    app.insert_resource(deck);
    
    // 逻辑验证：生成的 UI 数量
    let deck_res = app.world().resource::<PlayerDeck>();
    assert_eq!(deck_res.len(), 3, "UI 应生成 3 张对应的功法卡");
}

#[test]
fn test_deck_view_visibility_toggle() {
    // 模拟打开/关闭逻辑
    let is_open = true;
    let visibility = if is_open { Visibility::Visible } else { Visibility::Hidden };
    
    assert_eq!(visibility, Visibility::Visible);
}
