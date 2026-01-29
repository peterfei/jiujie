//! E2E测试：完整游戏流程
//!
//! 测试从主菜单到战斗的完整流程

use bevy::app::App;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_card_battler::plugins::{CorePlugin, MenuPlugin};
use bevy_card_battler::states::GameState;
use bevy_card_battler::components::{Player, Enemy};

// ============================================================================
// 测试辅助函数
// ============================================================================

/// 创建测试应用（不包含UI系统）
fn create_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(StatesPlugin)
        .init_state::<GameState>();
    app.update();
    app
}

/// 创建包含完整插件的测试应用
fn create_full_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(CorePlugin)
        .add_plugins(MenuPlugin)
        .add_plugins(StatesPlugin)
        .add_plugins(bevy::asset::AssetPlugin::default());
    app.update();
    app
}

/// 获取玩家数量
fn get_player_count(world: &mut World) -> usize {
    world.query::<&Player>().iter(world).count()
}

/// 获取敌人数量
fn get_enemy_count(world: &mut World) -> usize {
    world.query::<&Enemy>().iter(world).count()
}

// ============================================================================
// E2E测试：主菜单流程
// ============================================================================

#[test]
fn e2e_state_transitions_work() {
    // GIVEN: 创建应用，初始状态为MainMenu
    let mut app = create_test_app();

    // 验证初始状态
    let state = app.world_mut().resource::<State<GameState>>();
    assert_eq!(**state, GameState::MainMenu);

    // WHEN: 切换到Map状态
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Map);
    app.update();

    // THEN: 状态应该变为Map
    let state = app.world_mut().resource::<State<GameState>>();
    assert_eq!(**state, GameState::Map);

    // WHEN: 切换到Combat状态
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.update();

    // THEN: 状态应该变为Combat
    let state = app.world_mut().resource::<State<GameState>>();
    assert_eq!(**state, GameState::Combat);
}

#[test]
fn e2e_map_state_reachable() {
    // GIVEN: 从MainMenu开始
    let mut app = create_test_app();

    // WHEN: 手动切换到Map状态
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Map);
    app.update();

    // THEN: 状态应该变为Map
    let state = app.world_mut().resource::<State<GameState>>();
    assert_eq!(**state, GameState::Map);
}

#[test]
fn e2e_combat_state_reachable() {
    // GIVEN: 从Map状态开始
    let mut app = create_test_app();
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Map);
    app.update();

    // WHEN: 切换到Combat状态
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.update();

    // THEN: 状态应该变为Combat
    let state = app.world_mut().resource::<State<GameState>>();
    assert_eq!(**state, GameState::Combat);
}

// ============================================================================
// E2E测试：战斗逻辑
// ============================================================================

#[test]
fn e2e_player_initial_stats() {
    // GIVEN: 创建应用并添加玩家实体
    let mut app = create_test_app();
    let player_entity = app.world_mut().spawn(Player::default()).id();

    // WHEN: 更新一帧
    app.update();

    // THEN: 玩家应该有初始属性
    let world = app.world_mut();
    let player = world
        .query::<&Player>()
        .get(world, player_entity)
        .unwrap();

    assert_eq!(player.hp, 80, "玩家初始HP应该是80");
    assert_eq!(player.max_hp, 80, "玩家最大HP应该是80");
    assert_eq!(player.energy, 3, "玩家初始能量应该是3");
    assert_eq!(player.max_energy, 3, "玩家最大能量应该是3");
    assert_eq!(player.turn, 1, "初始回合应该是1");
}

#[test]
fn e2e_enemy_initial_stats() {
    // GIVEN: 创建应用并添加敌人实体
    let mut app = create_test_app();
    let enemy_entity = app.world_mut().spawn(Enemy::new(0, "嗜血妖狼", 30, 0)).id();

    // WHEN: 更新一帧
    app.update();

    // THEN: 敌人应该有初始属性
    let world = app.world_mut();
    let enemy = world
        .query::<&Enemy>()
        .get(world, enemy_entity)
        .unwrap();

    assert_eq!(enemy.hp, 30, "敌人初始HP应该是30");
    assert_eq!(enemy.max_hp, 30, "敌人最大HP应该是30");
    assert_eq!(enemy.name, "嗜血妖狼", "敌人名称应该是哥布林");
}

#[test]
fn e2e_player_take_damage() {
    // GIVEN: 创建应用并添加玩家实体
    let mut app = create_test_app();
    let player_entity = app.world_mut().spawn(Player::default()).id();
    app.update();

    // WHEN: 玩家受到伤害
    {
        let world = app.world_mut();
        let mut player = world
            .query::<&mut Player>()
            .get_mut(world, player_entity)
            .unwrap();
        player.take_damage(10);
    }

    // THEN: 玩家HP应该减少
    let world = app.world_mut();
    let player = world
        .query::<&Player>()
        .get(world, player_entity)
        .unwrap();

    assert_eq!(player.hp, 70, "玩家HP应该从80降到70");
}

#[test]
fn e2e_player_start_turn_resets_energy() {
    // GIVEN: 创建应用并添加玩家实体
    let mut app = create_test_app();
    let player_entity = app.world_mut().spawn(Player::default()).id();
    app.update();

    // 消耗所有能量
    {
        let world = app.world_mut();
        let mut player = world
            .query::<&mut Player>()
            .get_mut(world, player_entity)
            .unwrap();
        player.energy = 0;
    }

    // WHEN: 开始新回合
    {
        let world = app.world_mut();
        let mut player = world
            .query::<&mut Player>()
            .get_mut(world, player_entity)
            .unwrap();
        player.start_turn();
    }

    // THEN: 能量应该重置为最大值
    let world = app.world_mut();
    let player = world
        .query::<&Player>()
        .get(world, player_entity)
        .unwrap();

    assert_eq!(player.energy, 3, "能量应该重置为3");
    assert_eq!(player.turn, 2, "回合数应该增加");
}

#[test]
fn e2e_enemy_death_detection() {
    // GIVEN: 创建应用并添加敌人实体
    let mut app = create_test_app();
    let enemy_entity = app.world_mut().spawn(Enemy::new(0, "嗜血妖狼", 30, 0)).id();
    app.update();

    // WHEN: 敌人HP降为0
    {
        let world = app.world_mut();
        let mut enemy = world
            .query::<&mut Enemy>()
            .get_mut(world, enemy_entity)
            .unwrap();
        enemy.take_damage(30);
    }

    // THEN: 敌人应该被标记为死亡
    let world = app.world_mut();
    let enemy = world
        .query::<&Enemy>()
        .get(world, enemy_entity)
        .unwrap();

    assert!(enemy.is_dead(), "敌人HP为0时应该被标记为死亡");
}

// ============================================================================
// E2E测试：地图系统
// ============================================================================

#[test]
fn e2e_map_nodes_created() {
    // GIVEN: 创建应用并手动创建地图节点
    let mut app = create_test_app();

    // WHEN: 添加地图节点组件
    use bevy_card_battler::components::{MapNode, NodeType};
    let node_entity = app.world_mut().spawn(MapNode {
        id: 0,
        node_type: NodeType::Normal,
        position: (0, 0),
        unlocked: true,
        completed: false,
        next_nodes: Vec::new(),
    }).id();

    app.update();

    // THEN: 地图节点应该存在
    let world = app.world_mut();
    let node = world
        .query::<&MapNode>()
        .get(world, node_entity)
        .unwrap();

    assert_eq!(node.id, 0);
    assert_eq!(node.node_type, NodeType::Normal);
}

// ============================================================================
// E2E测试：完整流程
// ============================================================================

#[test]
fn e2e_full_game_flow() {
    // GIVEN: 从主菜单开始
    let mut app = create_test_app();

    // 验证初始状态
    let state = app.world_mut().resource::<State<GameState>>();
    assert_eq!(**state, GameState::MainMenu);

    // WHEN: 主菜单 -> 地图
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Map);
    app.update();

    // THEN: 状态变为Map
    let state = app.world_mut().resource::<State<GameState>>();
    assert_eq!(**state, GameState::Map);

    // WHEN: 地图 -> 战斗
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.update();

    // THEN: 状态变为Combat
    let state = app.world_mut().resource::<State<GameState>>();
    assert_eq!(**state, GameState::Combat);

    // WHEN: 战斗 -> 返回地图
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Map);
    app.update();

    // THEN: 状态变回Map
    let state = app.world_mut().resource::<State<GameState>>();
    assert_eq!(**state, GameState::Map);
}
