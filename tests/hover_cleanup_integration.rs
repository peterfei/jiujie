//! 悬停面板清理集成测试
//! 还原真实场景，确认清理问题

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::text::TextPlugin;
use bevy_card_battler::components::*;
use bevy_card_battler::plugins::{CorePlugin, MenuPlugin, HoveredCard, HoveredRelic, RewardCardButton, RewardRelicButton};
use bevy_card_battler::states::GameState;
use bevy_card_battler::systems::RelicPlugin;

/// 创建完整的测试应用（包含所有插件）
fn create_full_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .add_plugins(TextPlugin::default())
        .add_plugins(StatesPlugin)
        .add_plugins(CorePlugin)
        .add_plugins(MenuPlugin)
        .add_plugins(RelicPlugin)
        .init_state::<GameState>()
        .init_asset::<Font>();

    // 初始化所有必要资源
    let map_config = MapConfig { layers: 15, nodes_per_layer: 3, node_spacing: 150.0 };
    app.insert_resource(MapProgress::new(&map_config));
    app.insert_resource(RelicCollection::default());
    app.insert_resource(PlayerDeck::new());
    app.insert_resource(CombatState::default());

    // 从 MainMenu 转换到 Reward 状态
    app.world_mut().insert_resource(State::new(GameState::MainMenu));
    app.insert_state(GameState::Reward);
    app.update();

    app
}

#[test]
fn test_hover_panel_created_on_hover() {
    // 测试：悬停时面板被创建
    let mut app = create_full_app();

    // 查询奖励卡牌按钮
    let mut button_query = app.world_mut().query_filtered::<Entity, With<RewardCardButton>>();
    let button_count = button_query.iter(app.world()).count();

    if button_count > 0 {
        // 设置悬停状态
        {
            let mut hovered = app.world_mut().get_resource_mut::<HoveredCard>().unwrap();
            hovered.card_id = Some(1);
        }

        // 运行更新系统
        app.update();

        // 验证面板被创建
        let mut panel_query = app.world_mut().query_filtered::<Entity, With<CardHoverPanelMarker>>();
        let panel_count = panel_query.iter(app.world()).count();

        assert!(panel_count > 0, "悬停后应该有卡牌面板");
        info!("✓ 悬停后创建了 {} 个面板", panel_count);
    } else {
        info!("⚠ 没有找到卡牌按钮，跳过测试");
    }
}

#[test]
fn test_hover_panel_cleared_when_mouse_leaves() {
    // 测试：鼠标移开时面板被清理
    let mut app = create_full_app();

    // 步骤1：设置悬停状态
    {
        let mut hovered = app.world_mut().get_resource_mut::<HoveredCard>().unwrap();
        hovered.card_id = Some(1);
    }

    // 步骤2：创建面板（模拟悬停系统已执行）
    app.world_mut().spawn((
        Node::default(),
        CardHoverPanelMarker,
    ));

    // 验证面板存在
    let mut panel_query = app.world_mut().query_filtered::<Entity, With<CardHoverPanelMarker>>();
    assert_eq!(panel_query.iter(app.world()).count(), 1, "应该有1个面板");

    // 步骤3：模拟鼠标移开（这是关键步骤）
    {
        let mut hovered = app.world_mut().get_resource_mut::<HoveredCard>().unwrap();
        hovered.card_id = None;  // ← 这里模拟鼠标移开
    }
    info!("→ 已设置 hovered_card.card_id = None");

    // 步骤4：运行清理系统
    app.update();
    info!("→ 已运行 update()");

    // 步骤5：验证面板被清理
    let panel_count = panel_query.iter(app.world()).count();
    info!("→ 当前面板数量: {}", panel_count);

    assert_eq!(panel_count, 0, "鼠标移开后，面板应该被清理，但还有 {} 个", panel_count);
}

#[test]
fn test_cleanup_system_with_both_panels() {
    // 测试：同时有卡牌面板和遗物面板时的清理逻辑
    let mut app = create_full_app();

    // 创建两种面板
    app.world_mut().spawn((Node::default(), CardHoverPanelMarker));
    app.world_mut().spawn((Node::default(), RelicHoverPanelMarker));

    // 设置状态：两者都没有悬停
    {
        let mut hovered_card = app.world_mut().get_resource_mut::<HoveredCard>().unwrap();
        hovered_card.card_id = None;
    }
    {
        let mut hovered_relic = app.world_mut().get_resource_mut::<HoveredRelic>().unwrap();
        hovered_relic.relic_id = None;
    }

    // 验证初始状态
    let mut card_query = app.world_mut().query_filtered::<Entity, With<CardHoverPanelMarker>>();
    let mut relic_query = app.world_mut().query_filtered::<Entity, With<RelicHoverPanelMarker>>();
    assert_eq!(card_query.iter(app.world()).count(), 1, "初始应该有卡牌面板");
    assert_eq!(relic_query.iter(app.world()).count(), 1, "初始应该有遗物面板");

    // 运行清理系统
    app.update();

    // 验证两者都被清理
    let card_count = card_query.iter(app.world()).count();
    let relic_count = relic_query.iter(app.world()).count();

    info!("清理后：卡牌面板={}, 遗物面板={}", card_count, relic_count);

    assert_eq!(card_count, 0, "卡牌面板应该被清理，实际: {}", card_count);
    assert_eq!(relic_count, 0, "遗物面板应该被清理，实际: {}", relic_count);
}

#[test]
fn test_cleanup_with_only_card_hovered() {
    // 测试：只有卡牌悬停时，遗物面板应该被清理
    let mut app = create_full_app();

    // 创建两种面板
    app.world_mut().spawn((Node::default(), CardHoverPanelMarker));
    app.world_mut().spawn((Node::default(), RelicHoverPanelMarker));

    // 设置：卡牌悬停，遗物未悬停
    {
        let mut hovered_card = app.world_mut().get_resource_mut::<HoveredCard>().unwrap();
        hovered_card.card_id = Some(1);
    }
    {
        let mut hovered_relic = app.world_mut().get_resource_mut::<HoveredRelic>().unwrap();
        hovered_relic.relic_id = None;
    }

    app.update();

    // 验证：遗物面板被清理，卡牌面板保留
    let card_count = app.world_mut().query_filtered::<Entity, With<CardHoverPanelMarker>>()
        .iter(app.world()).count();
    let relic_count = app.world_mut().query_filtered::<Entity, With<RelicHoverPanelMarker>>()
        .iter(app.world()).count();

    info!("只有卡牌悬停时：卡牌面板={}, 遗物面板={}", card_count, relic_count);

    assert_eq!(card_count, 1, "卡牌面板应该保留");
    assert_eq!(relic_count, 0, "遗物面板应该被清理");
}

// 由于 HoveredCard 等是私有类型，这里只测试公开接口
// 实际的集成测试需要在游戏运行时验证
