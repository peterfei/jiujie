use bevy::prelude::*;
use bevy_card_battler::components::cards::{Card, CardType, CardEffect, CardRarity};
use bevy_card_battler::components::{Player, PlayerDeck};
use bevy_card_battler::states::GameState;
use bevy_card_battler::systems::rest::{UpgradeButton, LeaveButton, handle_rest_interactions, handle_leave_interaction};
use bevy_card_battler::components::map::MapProgress;

#[test]
fn test_rest_upgrade_with_confirmation_flow() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<GameState>();
    app.insert_state(GameState::Rest);

    // 1. 设置环境
    let card = Card::new(1, "测试卡", "描述", CardType::Attack, 1, CardEffect::DealDamage { amount: 5 }, CardRarity::Common, "");
    let mut deck = PlayerDeck::new();
    deck.add_card(card);
    app.insert_resource(deck);
    app.insert_resource(MapProgress::default());
    app.world_mut().spawn((Player::default(), bevy_card_battler::components::Cultivation::new()));

    // 模拟按钮
    let upgrade_btn = app.world_mut().spawn((UpgradeButton, Interaction::None)).id();
    let leave_btn = app.world_mut().spawn((LeaveButton, Interaction::None)).id();

    // 注册系统 (注意这里模拟的是重构后的逻辑)
    app.add_systems(Update, (handle_rest_interactions, handle_leave_interaction));
    app.update();

    // 2. 模拟点击“功法精进”
    if let Some(mut interaction) = app.world_mut().get_mut::<Interaction>(upgrade_btn) {
        *interaction = Interaction::Pressed;
    }
    app.update();

    // 验证：卡牌进阶了，但状态【不应该】改变
    {
        let deck = app.world().resource::<PlayerDeck>();
        assert!(deck.cards[0].upgraded, "卡牌应该已进阶");
        let state = app.world().resource::<State<GameState>>();
        assert_eq!(**state, GameState::Rest, "点击精进后应该留在原处供玩家确认结果");
    }

    // 3. 模拟点击“离开洞府”
    // 注意：我们需要先重置 interaction 状态，因为 handle_leave_interaction 也检测 Changed<Interaction>
    if let Some(mut interaction) = app.world_mut().get_mut::<Interaction>(leave_btn) {
        *interaction = Interaction::Pressed;
    }
    app.update();

    // 验证：现在状态应该变为 Map 了
    let state = app.world().resource::<State<GameState>>();
    assert_eq!(**state, GameState::Map, "点击离开后才真正跳转");

    println!("✅ 确认优化成功：功法精进现在有了确认流程，不再瞬间跳走。");
}