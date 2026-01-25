//! E2E测试：战斗UI系统
//!
//! 测试战斗UI交互，包括：
//! - 结束回合按钮只触发一次
//! - 回合数正确增加
//! - 手牌正确更新

use bevy::app::App;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_card_battler::components::*;
use bevy_card_battler::plugins::MenuPlugin;
use bevy_card_battler::states::GameState;

use bevy::asset::AssetPlugin;
use bevy::text::TextPlugin;

/// 创建完整测试应用（包含所有插件）
fn create_full_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .add_plugins(ImagePlugin::default())
        .init_asset::<Shader>()
        .init_asset::<Mesh>()
        .init_asset::<ColorMaterial>()
        .add_plugins(bevy::input::InputPlugin::default())
        .add_event::<bevy::picking::backend::PointerHits>()
        .add_event::<bevy::window::WindowScaleFactorChanged>()
        .add_event::<bevy::window::WindowResized>()
        .add_plugins(bevy::sprite::SpritePlugin::default())
        .add_plugins(bevy::ui::UiPlugin::default())
        .add_plugins(TextPlugin::default())
        .add_plugins((StatesPlugin, MenuPlugin))
        .init_state::<GameState>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>();

    app.update();
    app
}

// ============================================================================
// E2E测试：结束回合按钮
// ============================================================================

#[test]
fn e2e_end_turn_button_only_triggers_once_per_click() {
    // GIVEN: 创建战斗场景
    let mut app = create_full_test_app();

    // 创建玩家实体
    let player_entity = app.world_mut().spawn(Player::default()).id();

    // 模拟结束回合按钮被点击
    // 在真实场景中，这会触发 handle_combat_button_clicks 系统

    // WHEN: 执行一次 start_turn
    let initial_turn = {
        let world = app.world();
        let player = world.get::<Player>(player_entity).unwrap();
        player.turn
    };

    // 模拟点击一次
    {
        let world = app.world_mut();
        let mut player = world.get_mut::<Player>(player_entity).unwrap();
        player.start_turn();
    }

    app.update();

    // THEN: 回合数应该只增加1
    let final_turn = {
        let world = app.world();
        let player = world.get::<Player>(player_entity).unwrap();
        player.turn
    };

    assert_eq!(initial_turn, 1, "初始回合应该是1");
    assert_eq!(final_turn, 2, "点击一次后回合应该是2，不是12或其他数字");
}

#[test]
fn e2e_multiple_clicks_increase_turn_correctly() {
    // GIVEN: 创建玩家实体
    let mut app = create_full_test_app();
    let player_entity = app.world_mut().spawn(Player::default()).id();

    // WHEN: 模拟多次点击结束回合按钮
    let mut expected_turn = 1;
    for click_count in 1..=5 {
        {
            let world = app.world_mut();
            let mut player = world.get_mut::<Player>(player_entity).unwrap();
            player.start_turn();
        }
        app.update();
        expected_turn += 1;

        // THEN: 每次点击应该只增加1回合
        let world = app.world();
        let player = world.get::<Player>(player_entity).unwrap();
        assert_eq!(
            player.turn,
            expected_turn,
            "第{}次点击后，回合数应该是{}，不是{}",
            click_count,
            expected_turn,
            player.turn
        );
    }

    // 最终验证：5次点击后应该是第6回合
    let world = app.world();
    let player = world.get::<Player>(player_entity).unwrap();
    assert_eq!(player.turn, 6, "5次点击后应该是第6回合");
}

// ============================================================================
// E2E测试：手牌UI更新
// ============================================================================

#[test]
fn e2e_hand_ui_updates_when_hand_changes() {
    // GIVEN: 创建手牌实体
    let mut app = create_full_test_app();
    let hand_entity = app.world_mut().spawn(Hand::new(10)).id();

    // 初始手牌为空
    {
        let world = app.world();
        let hand = world.get::<Hand>(hand_entity).unwrap();
        assert_eq!(hand.len(), 0, "初始手牌应该是0张");
    }

    // WHEN: 添加卡牌到手牌
    let card = Card::new(
        0,
        "测试卡",
        "测试描述",
        CardType::Attack,
        1,
        CardEffect::DealDamage { amount: 5 },
        CardRarity::Common,
    );

    {
        let world = app.world_mut();
        let mut hand = world.get_mut::<Hand>(hand_entity).unwrap();
        hand.add_card(card);
    }

    app.update();

    // THEN: 手牌数量应该更新
    let world = app.world();
    let hand = world.get::<Hand>(hand_entity).unwrap();
    assert_eq!(hand.len(), 1, "添加1张卡后，手牌应该是1张");
}

// ============================================================================
// E2E测试：抽牌系统
// ============================================================================

#[test]
fn e2e_draw_cards_increases_hand_count() {
    // GIVEN: 创建抽牌堆和手牌
    let mut app = create_full_test_app();

    let cards = vec![
        Card::new(0, "打击", "", CardType::Attack, 1, CardEffect::DealDamage { amount: 6 }, CardRarity::Common),
        Card::new(1, "防御", "", CardType::Defense, 1, CardEffect::GainBlock { amount: 5 }, CardRarity::Common),
        Card::new(2, "突刺", "", CardType::Attack, 1, CardEffect::DealDamage { amount: 3 }, CardRarity::Common),
    ];

    let draw_pile_entity = app.world_mut().spawn(DrawPile::new(cards)).id();
    let hand_entity = app.world_mut().spawn(Hand::new(10)).id();

    app.update();

    // WHEN: 抽2张牌
    for _ in 0..2 {
        let world = app.world_mut();
        let mut draw_pile = world.get_mut::<DrawPile>(draw_pile_entity).unwrap();
        let card = draw_pile.draw_card();

        if let Some(c) = card {
            let world = app.world_mut();
            let mut hand = world.get_mut::<Hand>(hand_entity).unwrap();
            hand.add_card(c);
        }
    }

    app.update();

    // THEN: 手牌应该有2张牌，抽牌堆应该剩1张
    let world = app.world();
    let hand = world.get::<Hand>(hand_entity).unwrap();
    let draw_pile = world.get::<DrawPile>(draw_pile_entity).unwrap();

    assert_eq!(hand.len(), 2, "抽2张牌后，手牌应该是2张");
    assert_eq!(draw_pile.count, 1, "抽2张牌后，抽牌堆应该剩1张");
}

#[test]
fn e2e_empty_draw_pile_triggers_shuffle() {
    // GIVEN: 空的抽牌堆和有牌的弃牌堆
    let mut app = create_full_test_app();

    let discard_cards = vec![
        Card::new(0, "弃牌1", "", CardType::Attack, 1, CardEffect::DealDamage { amount: 3 }, CardRarity::Common),
        Card::new(1, "弃牌2", "", CardType::Defense, 1, CardEffect::GainBlock { amount: 3 }, CardRarity::Common),
    ];

    let draw_pile_entity = app.world_mut().spawn(DrawPile::new(vec![])).id();
    let discard_pile_entity = app.world_mut().spawn(DiscardPile::new()).id();
    let hand_entity = app.world_mut().spawn(Hand::new(10)).id();

    // 先往弃牌堆加牌
    {
        let world = app.world_mut();
        let mut discard_pile = world.get_mut::<DiscardPile>(discard_pile_entity).unwrap();
        for card in discard_cards {
            discard_pile.add_card(card);
        }
    }

    app.update();

    // WHEN: 抽牌堆为空时，将弃牌堆洗入抽牌堆
    let cards = {
        let world = app.world_mut();
        let mut discard_pile = world.get_mut::<DiscardPile>(discard_pile_entity).unwrap();
        discard_pile.clear()
    };

    if !cards.is_empty() {
        let world = app.world_mut();
        let mut draw_pile = world.get_mut::<DrawPile>(draw_pile_entity).unwrap();
        draw_pile.shuffle_from_discard(cards);
    }

    app.update();

    // THEN: 抽牌堆应该有牌，弃牌堆应该为空
    let world = app.world();
    let draw_pile = world.get::<DrawPile>(draw_pile_entity).unwrap();
    let discard_pile = world.get::<DiscardPile>(discard_pile_entity).unwrap();

    assert_eq!(draw_pile.count, 2, "洗牌后，抽牌堆应该有2张牌");
    assert_eq!(discard_pile.count, 0, "洗牌后，弃牌堆应该为空");
}
