use bevy::prelude::*;
use bevy_card_battler::components::{Hand, DrawPile, DiscardPile};
use bevy_card_battler::components::cards::Card;
use bevy_card_battler::plugins::{HandArea, HandCountText, HandCard, CombatUiRoot};
use bevy_card_battler::states::GameState;

#[test]
#[ignore = "Depends on complex UI rendering environment and AssetServer"]
fn test_hand_ui_reconstruction_on_data_mismatch() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        ImagePlugin::default(),
    ));
    app.init_asset::<Font>();

    // 1. 设置环境：手牌数据中有 3 张牌
    let mut hand = Hand::new(10);
    use bevy_card_battler::components::cards::{CardType, CardEffect, CardRarity};
    hand.cards.push(Card::new(1, "基础剑法", "描述", CardType::Attack, 1, CardEffect::DealDamage { amount: 5 }, CardRarity::Common, ""));
    hand.cards.push(Card::new(2, "基础剑法", "描述", CardType::Attack, 1, CardEffect::DealDamage { amount: 5 }, CardRarity::Common, ""));
    hand.cards.push(Card::new(3, "基础剑法", "描述", CardType::Attack, 1, CardEffect::DealDamage { amount: 5 }, CardRarity::Common, ""));
    
    app.world_mut().spawn(hand);
    app.world_mut().spawn(DrawPile::new(vec![]));
    app.world_mut().spawn(DiscardPile::new());

    // 2. 模拟 UI：创建 HandArea 但里面只有 1 个卡牌实体 (数据是3张，UI只有1张，不匹配)
    let hand_area = app.world_mut().spawn((
        Node::default(),
        HandArea,
        CombatUiRoot,
    )).with_children(|parent| {
        // 只有计数文本 (子节点索引 0)
        parent.spawn((Text::new("手牌: 3/10"), HandCountText));
        // 只有一张牌实体 (子节点索引 1)
        parent.spawn((Node::default(), HandCard { card_id: 1, base_bottom: 0.0, base_rotation: 0.0, index: 0 }));
    }).id();

    // 3. 运行 update_hand_ui
    app.add_systems(Update, bevy_card_battler::plugins::update_hand_ui);
    
    // 运行一帧
    app.update();

    // 4. 验证：子节点数量应该恢复到 4 个 (1个文本 + 3张牌)
    let children = app.world().get::<Children>(hand_area).expect("HandArea should have children");
    println!("Final UI Children count: {}", children.len());
    
    assert_eq!(children.len(), 4, "Hand UI should auto-heal to 4 children (1 text + 3 cards)");
}
