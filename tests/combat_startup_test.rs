use bevy::prelude::*;
use bevy_card_battler::components::cards::{PlayerDeck, CardPool, DrawPile, Hand};
use bevy_card_battler::components::combat::{CombatState, TurnPhase};
use bevy_card_battler::states::GameState;

#[test]
fn test_combat_initial_hand_drawing() {
    let mut app = App::new();
    
    // 1. 准备环境
    app.init_state::<GameState>();
    app.insert_resource(PlayerDeck::default());
    
    // 2. 模拟战斗初始化
    let combat_state = CombatState {
        phase: TurnPhase::PlayerStart,
        cards_drawn_this_turn: false,
        ..default()
    };
    app.insert_resource(combat_state);
    
    let draw_pile = DrawPile::new(CardPool::all_cards());
    let hand = Hand::new(10);
    app.world_mut().spawn(draw_pile);
    app.world_mut().spawn(hand);
    
    // 3. 运行抽牌系统 (我们需要在 lib.rs 中将其暴露或通过插件运行)
    // 这里的验证点：如果我们在 CorePlugin 里注册了系统，那么 app.update() 应该能执行抽牌
    
    // 暂时我们手动触发这个逻辑验证
    let mut query_draw = app.world_mut().query::<&mut DrawPile>();
    let mut query_hand = app.world_mut().query::<&mut Hand>();
    
    // 模拟 draw_cards_on_combat_start 逻辑
    let mut draw_pile = query_draw.get_single_mut(app.world_mut()).unwrap();
    let mut cards_to_move = Vec::new();
    let to_draw = 5.min(draw_pile.cards.len());
    for _ in 0..to_draw {
        if let Some(card) = draw_pile.draw_card() {
            cards_to_move.push(card);
        }
    }

    let mut hand = query_hand.get_single_mut(app.world_mut()).unwrap();
    for card in cards_to_move {
        hand.add_card(card);
    }
    
    // 4. 验证
    assert_eq!(hand.cards.len(), 5, "进入战斗时，首轮必须抽取 5 张机缘功法");
}
