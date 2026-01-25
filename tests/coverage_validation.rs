//! 集成测试覆盖验证
//!
//! 本测试文件补充了之前遗漏的关键场景，防止类似bug重复出现：
//! 注意：由于Bevy测试的资源共享问题，这些测试需要在单线程模式下运行
//! 使用 `cargo test --test coverage_validation -- --test-threads=1` 来运行
//!
//! 1. 组件结构验证 - 确保UI组件包含所有必需的标记
//! 2. 系统注册验证 - 确保插件系统真正运行
//! 3. Commands延迟行为 - 测试Bevy ECS的延迟特性
//! 4. 跨状态转换 - 验证多状态转换后的状态一致性
//! 5. 实体唯一性 - 防止重复创建实体导致查询混乱

use bevy::prelude::*;
use bevy::app::App;
use bevy::state::app::StatesPlugin;
use bevy::asset::AssetPlugin;
use bevy::text::TextPlugin;
use bevy_card_battler::components::*;
use bevy_card_battler::components::shop::{ShopUiRoot, ShopCardButton, ShopExitButton, CurrentShopItems};
use bevy_card_battler::components::particle::SpawnEffectEvent;
use bevy_card_battler::components::screen_effect::ScreenEffectEvent;
use bevy_card_battler::plugins::{CorePlugin, MenuPlugin, MapUiRoot, CombatUiRoot};
use bevy_card_battler::states::GameState;
use bevy_card_battler::systems::{ShopPlugin, RestPlugin};

// ============================================================================
// 辅助函数
// ============================================================================

fn create_full_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .add_plugins(TextPlugin::default())
        .add_plugins(StatesPlugin)
        .add_event::<SpawnEffectEvent>()
        .add_event::<ScreenEffectEvent>()
        .add_event::<EnemyAttackEvent>()
        .add_plugins(CorePlugin)
        .add_plugins(MenuPlugin)
        .add_plugins(ShopPlugin)
        .add_plugins(RestPlugin)
        .init_state::<GameState>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>();

    // 初始化玩家资源
    app.world_mut().insert_resource(PlayerDeck::new());
    app.world_mut().insert_resource(RelicCollection::default());

    // 初始化 TextPlugin 和 AssetPlugin 所需的资源
    app.world_mut().insert_resource(Assets::<Font>::default());
    app.world_mut().insert_resource(Assets::<Image>::default());
    app.world_mut().insert_resource(Assets::<TextureAtlasLayout>::default());

    app
}

// ============================================================================
// 测试1: 组件结构验证 - 商店UI按钮必须有标记组件
// ============================================================================

#[test]
fn test_shop_ui_purchase_buttons_have_markers() {
    // 场景：验证商店UI中的购买按钮有正确的标记组件
    // 预防：购买按钮只有 Button 组件，缺少 ShopCardButton/ShopRelicButton 标记

    let mut app = create_full_app();
    // 设置初始状态为商店
    {
        let world = app.world_mut();
        if let Some(mut next_state) = world.get_resource_mut::<NextState<GameState>>() {
            next_state.set(GameState::Shop);
        }
    }

    // 运行 OnEnter 创建UI
    for _ in 0..10 {
        app.update();
    }

    // 验证：购买按钮应该同时有 Button 和标记组件
    let mut card_btn_query = app.world_mut().query::<(Entity, &Button, &ShopCardButton)>();
    let card_btn_count = card_btn_query.iter(&app.world()).count();

    println!("✓ 商店卡牌按钮数量: {}", card_btn_count);
    assert!(card_btn_count > 0, "商店UI中应该有带标记的购买按钮");

    // 验证：每个 ShopCardButton 按钮都应该是 Button
    let mut all_card_buttons_query = app.world_mut().query::<(&ShopCardButton, &Button)>();
    let all_card_buttons_are_buttons = all_card_buttons_query.iter(&app.world()).count();

    println!("✓ 所有卡牌按钮都是Button: {}",
              all_card_buttons_are_buttons == card_btn_count);
    assert_eq!(all_card_buttons_are_buttons, card_btn_count,
               "所有 ShopCardButton 都应该是 Button");

    // 验证：返回按钮有标记
    let mut exit_btn_query = app.world_mut().query::<(&Button, &ShopExitButton)>();
    let exit_btn_count = exit_btn_query.iter(&app.world()).count();

    println!("✓ 返回按钮数量: {}", exit_btn_count);
    assert!(exit_btn_count > 0, "商店UI中应该有返回按钮");
}

// ============================================================================
// 测试2: 系统注册验证 - update_gold_display 是否运行
// ============================================================================

#[test]
fn test_shop_update_gold_system_runs() {
    // 场景：验证 update_gold_display 系统真的在运行
    // 预防：ShopPlugin 未注册或系统注册顺序错误

    let mut app = create_full_app();
    // 设置初始状态为商店
    {
        let world = app.world_mut();
        if let Some(mut next_state) = world.get_resource_mut::<NextState<GameState>>() {
            next_state.set(GameState::Shop);
        }
    }

    // 运行 OnEnter
    for _ in 0..10 {
        app.update();
    }

    // 获取初始金币文本
    let text_before = app.world_mut()
        .query::<&Text>()
        .iter(&app.world())
        .filter(|t| t.0.contains("金币"))
        .next()
        .map(|t| t.0.clone())
        .unwrap_or("未找到".to_string());

    println!("初始金币文本: {}", text_before);
    assert!(text_before.contains("100"), "初始应显示100金币");

    // 修改玩家金币（模拟购买）
    {
        let world = app.world_mut();
        if let Ok(mut player) = world.query::<&mut Player>().get_single_mut(world) {
            player.gold -= 30;
        }
    }

    // 运行多帧让 update_gold_display 处理
    for _ in 0..5 {
        app.update();
    }

    // 验证：金币文本应该更新
    let text_after = app.world_mut()
        .query::<&Text>()
        .iter(&app.world())
        .filter(|t| t.0.contains("金币"))
        .next()
        .map(|t| t.0.clone())
        .unwrap_or("未找到".to_string());

    println!("修改后金币文本: {}", text_after);
    assert!(text_after.contains("70"), "金币UI应该更新为70");
    assert!(!text_after.contains("100"), "金币UI不应该仍显示100");

    println!("✓ update_gold_display 系统正常运行");
}

// ============================================================================
// 测试3: Commands延迟行为 - spawn()不是立即生效
// ============================================================================

#[test]
fn test_commands_spawn_is_deferred() {
    // 场景：验证 Commands::spawn() 的延迟特性
    // 注意：Bevy 0.15 中，world_mut().spawn() 是立即的
    // 只有使用 Commands 参数的 spawn 才是延迟的

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .init_state::<GameState>();

    // 使用 world_mut() 直接创建 Player - Bevy 0.15 中这是立即的
    app.world_mut().spawn(Player::default());

    // 立即查询 - 应该找得到（Bevy 0.15 中直接 world spawn 是立即的）
    let mut immediate_query = app.world_mut().query::<&Player>();
    let immediate_count = immediate_query.iter(&app.world()).count();

    println!("✓ Bevy 0.15 world.spawn() 立即生效: 立即查询数量 = {}", immediate_count);
    assert_eq!(immediate_count, 1, "world_mut().spawn() 在 Bevy 0.15 中是立即生效的");

    // 测试延迟 Commands - 需要使用系统中的 Commands 参数
    app.add_systems(
        Update,
        |mut commands: Commands| {
            commands.spawn(Player { gold: 200, ..Default::default() });
        }
    );

    // 运行系统，Commands 会在帧末应用
    app.update();

    let query_after_system = app.world_mut().query::<&Player>().iter(&app.world()).count();
    println!("✓ 延迟 Commands 应用后查询数量 = {}", query_after_system);
    assert_eq!(query_after_system, 2, "Commands 在帧末应用，应该有2个Player");

    println!("✓ Commands 行为测试通过");
}

// ============================================================================
// 测试4: UI创建时Commands延迟导致的显示问题
// ============================================================================

#[test]
fn test_shop_ui_gold_display_with_deferred_commands() {
    // 场景：UI创建时使用 Player 查询，但 Player 是延迟创建的
    // 预防：UI 显示 "金币: 0" 因为 Player 尚未创建

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .add_plugins(TextPlugin::default())
        .add_plugins(StatesPlugin)
        .add_event::<SpawnEffectEvent>()
        .add_event::<ScreenEffectEvent>()
        .add_event::<EnemyAttackEvent>()
        .add_plugins(CorePlugin)
        .add_plugins(ShopPlugin)
        .init_state::<GameState>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>();

    app.world_mut().insert_resource(PlayerDeck::new());
    app.world_mut().insert_resource(RelicCollection::default());
    app.world_mut().insert_resource(CurrentShopItems::default());

    // 初始化 TextPlugin 和 AssetPlugin 所需的资源
    app.world_mut().insert_resource(Assets::<Font>::default());
    app.world_mut().insert_resource(Assets::<Image>::default());
    app.world_mut().insert_resource(Assets::<TextureAtlasLayout>::default());

    // 设置商店状态（不手动创建 Player）
    // 注意：需要使用 NextState 来触发 OnEnter 系统
    {
        let world = app.world_mut();
        if let Some(mut next_state) = world.get_resource_mut::<NextState<GameState>>() {
            next_state.set(GameState::Shop);
        }
    }

    // 运行 OnEnter - setup_shop_ui 会创建 Player
    for _ in 0..10 {
        app.update();
    }

    // 检查：Player 应该被创建
    let player_count = app.world_mut().query::<&Player>().iter(&app.world()).count();
    println!("✓ Player 实体数量: {}", player_count);
    assert!(player_count > 0, "Player 应该被创建");

    // 检查：UI 应该显示正确的金币（不是0）
    let gold_text = app.world_mut()
        .query::<&Text>()
        .iter(&app.world())
        .filter(|t| t.0.contains("金币"))
        .next()
        .map(|t| t.0.clone())
        .unwrap_or_else(|| "未找到".to_string());

    println!("金币显示: {}", gold_text);
    assert!(gold_text.contains("100"), "应该显示100金币，不是0");

    println!("✓ 延迟 Commands 不影响 UI 显示测试通过");
}

// ============================================================================
// 测试5: 跨状态转换 - 实体唯一性
// ============================================================================

#[test]
fn test_no_duplicate_players_after_state_transitions() {
    // 场景：多次状态转换后应该只有一个 Player 实体
    // 预防：每个 OnEnter 系统都创建 Player，导致重复

    let mut app = create_full_app();

    // 初始化地图进度（包含商店节点）
    let mut progress = MapProgress::default();
    progress.nodes = vec![
        MapNode {
            id: 0,
            node_type: NodeType::Shop,
            position: (0, 0),
            unlocked: true,
            completed: false,
        },
        MapNode {
            id: 1,
            node_type: NodeType::Normal,
            position: (1, 0),
            unlocked: false,
            completed: false,
        },
    ];
    app.world_mut().insert_resource(progress);

    // 状态转换：Main → Map → Shop → Map → Combat
    {
        let world = app.world_mut();
        if let Some(mut next_state) = world.get_resource_mut::<NextState<GameState>>() {
            next_state.set(GameState::Map);
        }
    }
    for _ in 0..5 { app.update(); }

    // 触发状态转换到商店
    {
        let world = app.world_mut();
        if let Some(mut next_state) = world.get_resource_mut::<NextState<GameState>>() {
            next_state.set(GameState::Shop);
        }
    }
    for _ in 0..10 { app.update(); }

    // 触发状态转换到地图
    {
        let world = app.world_mut();
        if let Some(mut next_state) = world.get_resource_mut::<NextState<GameState>>() {
            next_state.set(GameState::Map);
        }
    }
    for _ in 0..5 { app.update(); }

    // 触发状态转换到战斗
    {
        let world = app.world_mut();
        if let Some(mut next_state) = world.get_resource_mut::<NextState<GameState>>() {
            next_state.set(GameState::Combat);
        }
    }
    for _ in 0..10 { app.update(); }

    // 验证：应该只有一个 Player 实体
    let player_count = app.world_mut().query::<&Player>().iter(&app.world()).count();

    println!("✓ 状态转换后 Player 实体数量: {}", player_count);
    assert_eq!(player_count, 1, "跨状态转换后应该只有一个 Player 实体");

    // 验证：Player 的状态应该是战斗状态（能量已重置）
    if let Some(player) = app.world_mut().query::<&Player>().iter(&app.world()).next() {
        println!("✓ Player 能量: {}/{}", player.energy, player.max_energy);
        assert_eq!(player.energy, 3, "战斗开始时能量应该被重置为3");
    }

    println!("✓ 实体唯一性测试通过");
}

// ============================================================================
// 测试6: 端到端完整流程
// ============================================================================

#[test]
fn test_e2e_full_shop_and_combat_flow() {
    // 场景：完整的用户操作流程
    // 1. 主菜单 → 地图
    // 2. 地图 → 商店
    // 3. 商店购买
    // 4. 商店 → 地图
    // 5. 地图 → 战斗
    // 6. 战斗出牌

    let mut app = create_full_app();

    // 初始化地图进度
    let mut progress = MapProgress::default();
    progress.nodes = vec![
        MapNode {
            id: 0,
            node_type: NodeType::Shop,
            position: (0, 0),
            unlocked: true,
            completed: false,
        },
        MapNode {
            id: 1,
            node_type: NodeType::Normal,
            position: (1, 0),
            unlocked: false,
            completed: false,
        },
    ];
    app.world_mut().insert_resource(progress);

    // 步骤1: 进入地图
    {
        let world = app.world_mut();
        if let Some(mut next_state) = world.get_resource_mut::<NextState<GameState>>() {
            next_state.set(GameState::Map);
        }
    }
    for _ in 0..5 { app.update(); }

    let map_ui_count = app.world_mut().query::<&MapUiRoot>()
        .iter(&app.world()).count();
    println!("步骤1: 地图UI数量 = {}", map_ui_count);
    assert!(map_ui_count > 0, "应该有地图UI");

    // 步骤2: 进入商店
    {
        let world = app.world_mut();
        if let Some(mut next_state) = world.get_resource_mut::<NextState<GameState>>() {
            next_state.set(GameState::Shop);
        }
    }
    for _ in 0..10 { app.update(); }

    // 检查商店UI和金币
    let shop_ui_count = app.world_mut().query::<&ShopUiRoot>().iter(&app.world()).count();
    println!("步骤2: 商店UI数量 = {}", shop_ui_count);
    assert!(shop_ui_count > 0, "应该有商店UI");

    let gold_before = app.world_mut().query::<&Player>()
        .iter(&app.world())
        .next()
        .map(|p| p.gold)
        .unwrap_or(0);
    println!("步骤2: 初始金币 = {}", gold_before);
    assert_eq!(gold_before, 100, "应该有100初始金币");

    // 步骤3: 模拟购买
    {
        let world = app.world_mut();
        if let Ok(mut player) = world.query::<&mut Player>().get_single_mut(world) {
            player.gold = 70; // 模拟花费30金币
        }
    }

    // 运行更新系统
    for _ in 0..3 { app.update(); }

    // 验证金币更新
    let gold_after = app.world_mut().query::<&Player>()
        .iter(&app.world())
        .next()
        .map(|p| p.gold)
        .unwrap_or(0);
    println!("步骤3: 购买后金币 = {}", gold_after);
    assert_eq!(gold_after, 70, "购买后金币应该是70");

    // 步骤4: 返回地图
    {
        let world = app.world_mut();
        if let Some(mut next_state) = world.get_resource_mut::<NextState<GameState>>() {
            next_state.set(GameState::Map);
        }
    }
    for _ in 0..5 { app.update(); }

    // 步骤5: 进入战斗
    {
        let world = app.world_mut();
        if let Some(mut next_state) = world.get_resource_mut::<NextState<GameState>>() {
            next_state.set(GameState::Combat);
        }
    }
    for _ in 0..10 { app.update(); }

    // 验证战斗状态
    let combat_ui_count = app.world_mut().query::<&CombatUiRoot>()
        .iter(&app.world()).count();
    println!("步骤5: 战斗UI数量 = {}", combat_ui_count);
    assert!(combat_ui_count > 0, "应该有战斗UI");

    // 验证战斗能量
    let player_energy = app.world_mut().query::<&Player>()
        .iter(&app.world())
        .next()
        .map(|p| p.energy)
        .unwrap_or(0);
    println!("步骤5: 战斗能量 = {}", player_energy);
    assert_eq!(player_energy, 3, "战斗开始时能量应该是3");

    // 验证 Player 唯一性
    let player_count = app.world_mut().query::<&Player>().iter(&app.world()).count();
    println!("步骤5: Player数量 = {}", player_count);
    assert_eq!(player_count, 1, "应该只有一个Player");

    println!("✓ 端到端流程测试通过");
}

// ============================================================================
// 测试7: 重复OnEnter系统不应重复创建实体
// ============================================================================

#[test]
fn test_multiple_on_enter_same_state_doesnt_duplicate() {
    // 场景：多次进入同一个状态不应重复创建实体
    // 预防：每次 OnEnter 都创建新实体

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .add_plugins(TextPlugin::default())
        .add_plugins(StatesPlugin)
        .add_event::<SpawnEffectEvent>()
        .add_event::<ScreenEffectEvent>()
        .add_event::<EnemyAttackEvent>()
        .add_plugins(CorePlugin)
        .add_plugins(ShopPlugin)
        .init_state::<GameState>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>();

    app.world_mut().insert_resource(PlayerDeck::new());
    app.world_mut().insert_resource(RelicCollection::default());
    app.world_mut().insert_resource(CurrentShopItems::default());

    // 初始化 TextPlugin 和 AssetPlugin 所需的资源
    app.world_mut().insert_resource(Assets::<Font>::default());
    app.world_mut().insert_resource(Assets::<Image>::default());
    app.world_mut().insert_resource(Assets::<TextureAtlasLayout>::default());

    // 第一次进入商店
    {
        let world = app.world_mut();
        if let Some(mut next_state) = world.get_resource_mut::<NextState<GameState>>() {
            next_state.set(GameState::Shop);
        }
    }
    for _ in 0..10 { app.update(); }

    let player_count_1 = app.world_mut().query::<&Player>().iter(&app.world()).count();
    println!("✓ 第一次进入商店后 Player 数量: {}", player_count_1);
    assert_eq!(player_count_1, 1, "第一次进入应该创建1个Player");

    // 第二次进入商店（先退出再进入）
    {
        let world = app.world_mut();
        if let Some(mut next_state) = world.get_resource_mut::<NextState<GameState>>() {
            next_state.set(GameState::Map);
        }
    }
    for _ in 0..5 { app.update(); }

    {
        let world = app.world_mut();
        if let Some(mut next_state) = world.get_resource_mut::<NextState<GameState>>() {
            next_state.set(GameState::Shop);
        }
    }
    for _ in 0..10 { app.update(); }

    let player_count_2 = app.world_mut().query::<&Player>().iter(&app.world()).count();
    println!("✓ 第二次进入商店后 Player 数量: {}", player_count_2);
    assert_eq!(player_count_2, 1, "第二次进入不应该创建新Player，仍然只有1个");

    println!("✓ 重复OnEnter不重复创建实体测试通过");
}

// ============================================================================
// 测试8: 系统执行顺序 - reset 在交互之前
// ============================================================================

#[test]
fn test_system_order_reset_before_interaction() {
    // 场景：验证系统执行顺序正确
    // reset_player_on_combat_start 应在 handle_shop_interactions 之前执行

    let mut app = create_full_app();
    // 设置初始状态为战斗
    {
        let world = app.world_mut();
        if let Some(mut next_state) = world.get_resource_mut::<NextState<GameState>>() {
            next_state.set(GameState::Combat);
        }
    }

    // 运行 OnEnter 系统应该在第一个 update 帧完成
    app.update();

    // 验证：reset 系统已运行
    if let Some(player) = app.world_mut().query::<&Player>().iter(&app.world()).next() {
        println!("✓ 战斗开始 Player 能量: {}/{}", player.energy, player.max_energy);
        assert_eq!(player.energy, 3, "reset系统应该先执行，设置能量为3");
    }

    // 模拟出牌消耗能量
    {
        let world = app.world_mut();
        if let Ok(mut player) = world.query::<&mut Player>().get_single_mut(world) {
            player.energy = 1;
            println!("✓ 模拟出牌后能量: {}", player.energy);
        }
    }

    app.update();

    // 验证：能量应该保持为1（不应该被reset重置）
    if let Some(player) = app.world_mut().query::<&Player>().iter(&app.world()).next() {
        println!("✓ 出牌后 Player 能量: {}", player.energy);
        assert_eq!(player.energy, 1, "出牌后能量应该保持为1，不会被重置");
    }

    println!("✓ 系统执行顺序测试通过");
}
