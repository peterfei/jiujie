use bevy::prelude::*;
use bevy_card_battler::components::Hand;
use bevy_card_battler::plugins::{HandArea, HandCard, update_hand_ui};

#[test]
fn test_prevent_infinite_rebuild_flicker() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        ImagePlugin::default(),
    ));
    app.init_asset::<Font>();
    
    // 1. 设置手牌数据 (3张)
    let mut hand = Hand::new(10);
    for i in 0..3 {
        hand.cards.push(bevy_card_battler::components::cards::Card::new(
            i, "剑法", "", bevy_card_battler::components::cards::CardType::Attack, 
            1, bevy_card_battler::components::cards::CardEffect::Heal { amount: 1 }, 
            bevy_card_battler::components::cards::CardRarity::Common, ""
        ));
    }
    app.world_mut().spawn(hand);

    // 2. 模拟 UI (初始为空)
    let hand_area = app.world_mut().spawn((Node::default(), HandArea)).id();

    // 3. 运行系统 (第一帧：应该触发重建)
    app.add_systems(Update, update_hand_ui);
    app.update();
    
    // 此时 commands 已发出，但 Children 还没更新到 hand_area 上
    // 4. 连续运行多帧，让冷却结束并观察是否稳定
    for _ in 0..10 {
        app.update();
    }

    let mut query = app.world_mut().query::<&HandCard>();
    let total_hand_cards = query.iter(app.world()).count();
    println!("Final HandCard count after cooldown: {}", total_hand_cards);
    
    assert_eq!(total_hand_cards, 3, "Hand UI should be stable and not flicker/duplicate");
}
