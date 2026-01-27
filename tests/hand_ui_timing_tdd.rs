use bevy::prelude::*;
use bevy_card_battler::components::{Hand, DrawPile, DiscardPile};
use bevy_card_battler::plugins::{HandArea, update_hand_ui, HandCard};
use bevy_card_battler::states::GameState;

#[test]
fn test_hand_ui_sync_after_delayed_setup() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        ImagePlugin::default(),
    ));
    app.init_asset::<Font>();

    // 1. 模拟状态：已经抽了 5 张牌，但 UI 还没创建
    let mut hand = Hand::new(10);
    for i in 0..5 {
        hand.cards.push(bevy_card_battler::components::cards::Card::new(
            i, "剑法", "描述", bevy_card_battler::components::cards::CardType::Attack, 
            1, bevy_card_battler::components::cards::CardEffect::Heal { amount: 1 }, 
            bevy_card_battler::components::cards::CardRarity::Common, ""
        ));
    }
    app.world_mut().spawn(hand);
    app.world_mut().spawn(DrawPile::new(vec![]));
    app.world_mut().spawn(DiscardPile::new());

    // 2. 注册 UI 更新系统
    app.add_systems(Update, update_hand_ui);

    // 3. 运行一帧：此时没有 HandArea，系统应该静默失败，不报错但也不生成牌
    app.update();
    {
        let mut query = app.world_mut().query::<&HandCard>();
        assert!(query.iter(app.world()).next().is_none());
    }

    // 4. 现在生成 HandArea (模拟 setup_combat_ui 延迟运行)
    app.world_mut().spawn((Node::default(), HandArea));

    // 5. 再次运行：自愈逻辑应该在检测到 HandArea 存在且子节点为空时立即重建
    app.update();

    // 验证：手牌应该被创建
    let mut query = app.world_mut().query::<&HandCard>();
    let card_count = query.iter(app.world()).count();
    println!("Detected HandCards: {}", card_count);
    assert_eq!(card_count, 5, "Hand UI should eventually sync even if setup was delayed");
}
