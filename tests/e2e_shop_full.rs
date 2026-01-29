//! 端到端商店测试
//!
//! 完整测试：地图 → 商店 → 显示商品

use bevy::prelude::*;
use bevy::app::App;
use bevy::state::app::StatesPlugin;
use bevy::asset::AssetPlugin;
use bevy::text::TextPlugin;
use bevy_card_battler::components::*;
use bevy_card_battler::components::shop::*;
use bevy_card_battler::plugins::{CorePlugin, MenuPlugin};
use bevy_card_battler::states::GameState;

use bevy_card_battler::systems::ShopPlugin;

#[test]
fn e2e_full_shop_flow_from_map_to_shop() {
    // 完整测试：从地图进入商店，显示UI和商品

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
        .add_plugins(StatesPlugin)
        .add_plugins(CorePlugin)
        .add_plugins(MenuPlugin)
        .add_plugins(ShopPlugin)
        .init_state::<GameState>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<PlayerDeck>() // 初始化玩家牌组
        .init_resource::<RelicCollection>(); // 初始化遗物背包（以防万一）

    // 初始化地图进度（包含商店节点）
    let mut progress = MapProgress::default();
    progress.nodes = vec![
        MapNode {
            id: 0,
            node_type: NodeType::Shop,
            position: (0, 0),
            unlocked: true,
            completed: false, next_nodes: Vec::new(),
        },
    ];
    app.world_mut().insert_resource(progress);

    // 设置地图状态
    app.world_mut().insert_resource(State::new(GameState::Map));

    // 运行地图UI创建
    for _ in 0..3 {
        app.update();
    }

    // 模拟点击商店节点
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Shop);

    // 运行状态转换和OnEnter系统（关键：这里触发setup_shop_ui）
    for _ in 0..5 {
        app.update();
    }

    // 验证：商店UI应该存在
    let mut shop_ui_query = app.world_mut().query::<&ShopUiRoot>();
    assert!(shop_ui_query.iter(app.world_mut()).count() > 0, "应该有商店UI");

    // 验证：Player实体应该存在
    let mut player_query = app.world_mut().query::<&Player>();
    assert!(player_query.iter(app.world_mut()).count() > 0, "应该有Player实体");

    // 验证：商店商品应该生成
    let shop_items = app.world().get_resource::<CurrentShopItems>();
    assert!(shop_items.is_some(), "应该有CurrentShopItems资源");
    assert!(!shop_items.unwrap().items.is_empty(), "应该有商品");

    println!("✅ 商店端到端测试通过");
    println!("   - 商店UI已创建");
    println!("   - Player实体已创建");
    println!("   - 商品已生成: {} 个", shop_items.unwrap().items.len());
}
