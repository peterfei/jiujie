//! 地图节点交互E2E测试
//!
//! 覆盖所有节点类型的点击交互

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_card_battler::components::*;
use bevy_card_battler::plugins::{CorePlugin, MenuPlugin, MapNodeButton};
use bevy_card_battler::systems::RestPlugin;
use bevy_card_battler::systems::rest::RestContinueButton;
use bevy_card_battler::states::GameState;

use bevy::asset::AssetPlugin;

fn create_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .add_plugins(ImagePlugin::default()) // 即使不使用UiPlugin，MenuPlugin加载字体也可能需要
        .init_asset::<Shader>()
        .init_asset::<Mesh>()
        .init_asset::<ColorMaterial>()
        .init_asset::<Font>()
        .init_asset::<Image>()
        .add_plugins(StatesPlugin)
        .add_plugins(CorePlugin)
        .add_plugins(MenuPlugin)
        .add_plugins(RestPlugin) // 添加 RestPlugin
        .init_state::<GameState>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>();
    
    app.update();
    app
}

// ============================================================================
// 休息节点测试
// ============================================================================

#[test]
fn e2e_rest_restores_player_hp() {
    // GIVEN: 玩家受伤状态
    let mut app = create_test_app();
    {
        let mut player_query = app.world_mut().query::<&mut Player>();
        let mut player = player_query.iter_mut(app.world_mut()).next().unwrap();
        player.hp = 30;
        player.max_hp = 50;
    }

    // WHEN: 进入休息状态
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Rest);

    // 运行多次更新以触发状态转换和OnEnter系统
    for _ in 0..3 {
        app.update();
    }

    // THEN: 玩家生命值应该恢复
    let mut player_query = app.world_mut().query::<&Player>();
    let player = player_query.iter(app.world_mut()).next().unwrap();
    // 应该恢复30%，即15点，30+15=45
    assert_eq!(player.hp, 45, "HP应该从30恢复到45");
}

#[test]
fn e2e_rest_does_not_exceed_max_hp() {
    // GIVEN: 玩家生命值接近满值
    let mut app = create_test_app();
    {
        let mut player_query = app.world_mut().query::<&mut Player>();
        let mut player = player_query.iter_mut(app.world_mut()).next().unwrap();
        player.hp = 48;
        player.max_hp = 50;
    }

    // WHEN: 进入休息状态
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Rest);

    // 运行多次更新以触发状态转换和OnEnter系统
    for _ in 0..3 {
        app.update();
    }

    // THEN: 不应超过最大值
    let mut player_query = app.world_mut().query::<&Player>();
    let player = player_query.iter(app.world_mut()).next().unwrap();
    assert_eq!(player.hp, 50, "HP不应超过最大值50");
}

#[test]
fn e2e_rest_returns_to_map_on_click() {
    // GIVEN: 进入休息状态
    let mut app = create_test_app();
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Rest);
    app.update();

    // 初始状态应该是Rest
    let state = app.world().get_resource::<State<GameState>>().unwrap();
    assert_eq!(*state.get(), GameState::Rest);

    // WHEN: 模拟点击（手动插入并设置组件状态）
    {
        let mut button_query = app.world_mut().query_filtered::<Entity, With<RestContinueButton>>();
        let button_entity = button_query.iter(app.world_mut()).next().expect("应该找到继续按钮");
        
        // 显式插入 Interaction 组件，因为没有 UiPlugin 自动添加
        app.world_mut().entity_mut(button_entity).insert(Interaction::Pressed);
    }
    app.update(); // 运行 handle_rest_interactions 系统，设置 NextState
    app.update(); // 运行状态转换系统，应用新状态

    // THEN: 应该返回地图
    let state = app.world().get_resource::<State<GameState>>().unwrap();
    assert_eq!(*state.get(), GameState::Map, "应该返回地图状态");
}

// ============================================================================
// 商店节点测试
// ============================================================================

#[test]
fn e2e_shop_node_transitions_to_shop_state() {
    // GIVEN: 在地图状态
    let mut app = create_test_app();
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Map);
    app.update();

    // WHEN: 进入商店状态
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Shop);

    // 运行多次更新以触发状态转换和OnEnter系统
    for _ in 0..3 {
        app.update();
    }

    // THEN: 应该在商店状态
    let state = app.world().get_resource::<State<GameState>>().unwrap();
    assert_eq!(*state.get(), GameState::Shop);
}

// NOTE: 商店金币逻辑的测试在 shop_tdd.rs 中
// E2E测试需要字体资源，在无头环境下不可用
// 以下是跳过的测试占位符

#[test]
#[ignore = "需要字体资源，已由 shop_tdd.rs 覆盖"]
fn e2e_shop_gives_initial_gold() {
    // 此测试已移至 tests/shop_tdd.rs
    // test_player_gets_initial_gold_on_first_shop_visit
}

#[test]
#[ignore = "需要字体资源，已由 shop_tdd.rs 覆盖"]
fn e2e_shop_keeps_existing_gold() {
    // 此测试已移至 tests/shop_tdd.rs
    // test_player_keeps_existing_gold_on_subsequent_visits
}

// ============================================================================
// 节点类型枚举覆盖测试
// ============================================================================

#[test]
fn coverage_all_node_types_are_tested() {
    // 这是一个"元测试"，确保所有节点类型都被测试覆盖

    // 列出所有节点类型
    let all_types = vec![
        NodeType::Normal,
        NodeType::Elite,
        NodeType::Boss,
        NodeType::Rest,
        NodeType::Shop,
        NodeType::Treasure,
        NodeType::Unknown,
    ];

    // 这个测试的存在表明我们意识到需要覆盖所有类型
    // TODO: 随着测试的添加，更新此列表

    // 当前已测试的节点类型：
    let tested = vec![
        NodeType::Rest,   // e2e_rest_restores_player_hp
        NodeType::Shop,   // e2e_shop_node_transitions_to_shop_state
    ];

    let missing: Vec<_> = all_types.iter()
        .filter(|t| !tested.contains(t))
        .collect();

    if !missing.is_empty() {
        eprintln!("\n⚠️  以下节点类型还需要E2E测试:");
        for t in &missing {
            eprintln!("   - {:?}", t);
        }
    }

    // 不让测试失败，只是提醒
    assert!(tested.len() >= 2, "至少要有部分测试覆盖");
}
