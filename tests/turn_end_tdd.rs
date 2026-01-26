use bevy::prelude::*;
use bevy_card_battler::components::cards::{Hand, DiscardPile, Card, CardType, CardEffect, CardRarity};

#[test]
fn test_hand_clear_on_turn_end() {
    let mut app = App::new();
    
    // 1. 准备数据：手牌中有 3 张牌，弃牌堆为空
    let mut hand = Hand::new(10);
    for i in 0..3 {
        hand.add_card(Card::new(i, "测试", "", CardType::Attack, 1, CardEffect::DealDamage { amount: 5 }, CardRarity::Common, ""));
    }
    let discard_pile = DiscardPile::new();
    
    let hand_entity = app.world_mut().spawn(hand).id();
    let discard_entity = app.world_mut().spawn(discard_pile).id();

    // 2. 模拟“结束回合”时的清空逻辑
    {
        // 先取出所有牌
        let mut cards_to_discard = Vec::new();
        if let Some(mut hand) = app.world_mut().get_mut::<Hand>(hand_entity) {
            while let Some(card) = hand.remove_card(0) {
                cards_to_discard.push(card);
            }
        }
        
        // 再存入弃牌堆
        if let Some(mut discard) = app.world_mut().get_mut::<DiscardPile>(discard_entity) {
            for card in cards_to_discard {
                discard.add_card(card);
            }
        }
    }

    // 3. 验证结果
    let final_hand = app.world().get::<Hand>(hand_entity).unwrap();
    let final_discard = app.world().get::<DiscardPile>(discard_entity).unwrap();
    
    assert_eq!(final_hand.len(), 0, "结束回合后手牌应为空");
    assert_eq!(final_discard.count, 3, "牌应进入弃牌堆");
}
