//! 战斗系统专项测试
//!
//! 测试战斗相关的 bug 和场景
//! 继承自 commit 2a818d5 测试框架

use crate::test_utils::*;
use bevy::prelude::*;
use bevy_card_battler::components::{Player, MapNode, NodeType, MapProgress};
use bevy_card_battler::states::GameState;
use bevy_card_battler::components::combat::CombatUiRoot;
use bevy_card_battler::plugins::MapUiRoot;

// ============================================================================
// 测试1: 跨状态转换 - 实体唯一性
// ============================================================================

#[test]
fn test_no_duplicate_players_after_state_transitions() {
    // 场景：多次状态转换后应该只有一个 Player 实体
    // 预防：每个 OnEnter 系统都创建 Player，导致重复

    let mut app = create_test_app();

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

    // 状态转换：Map → Shop → Map → Combat
    transition_to_state(&mut app, GameState::Map);
    advance_frames(&mut app, 5);

    transition_to_state(&mut app, GameState::Shop);
    advance_frames(&mut app, 10);

    transition_to_state(&mut app, GameState::Map);
    advance_frames(&mut app, 5);

    setup_combat_scene(&mut app);
    advance_frames(&mut app, 10);

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
// 测试2: 系统执行顺序 - reset 在交互之前
// ============================================================================

#[test]
fn test_system_order_reset_before_interaction() {
    // 场景：验证系统执行顺序正确
    // reset_player_on_combat_start 应在交互系统之前执行

    let mut app = create_test_app();
    setup_combat_scene(&mut app);

    // 运行 OnEnter 系统
    advance_frames(&mut app, 1);

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

    advance_frames(&mut app, 1);

    // 验证：能量应该保持为1（不应该被reset重置）
    if let Some(player) = app.world_mut().query::<&Player>().iter(&app.world()).next() {
        println!("✓ 出牌后 Player 能量: {}", player.energy);
        assert_eq!(player.energy, 1, "出牌后能量应该保持为1");
    }

    println!("✓ 系统执行顺序测试通过");
}

// ============================================================================
// 测试3: Commands延迟行为验证
// ============================================================================

#[test]
fn test_commands_spawn_is_deferred() {
    // 场景：验证 Commands::spawn() 的延迟特性
    // 注意：Bevy 0.15 中，world_mut().spawn() 是立即的
    // 只有使用 Commands 参数的 spawn 才是延迟的

    let mut app = create_test_app();

    // 使用 world_mut() 直接创建 Player - Bevy 0.15 中这是立即的
    app.world_mut().spawn(Player::default());

    // 立即查询 - 应该找得到（Bevy 0.15 中直接 world spawn 是立即的）
    let mut immediate_query = app.world_mut().query::<&Player>();
    let immediate_count = immediate_query.iter(&app.world()).count();

    println!("✓ Bevy 0.15 world.spawn() 立即生效: 立即查询数量 = {}", immediate_count);
    assert_eq!(immediate_count, 1, "world_mut().spawn() 在 Bevy 0.15 中是立即生效的");

    // 测试延迟 Commands - 需要使用系统中的 Commands 参数
    app.add_systems(Update, |mut commands: Commands| {
        commands.spawn(Player { gold: 200, ..Default::default() });
    });

    // 运行系统，Commands 会在帧末应用
    advance_frames(&mut app, 1);

    let query_after_system = app.world_mut().query::<&Player>().iter(&app.world()).count();
    println!("✓ 延迟 Commands 应用后查询数量 = {}", query_after_system);
    assert_eq!(query_after_system, 2, "Commands 在帧末应用，应该有2个Player");

    println!("✓ Commands 行为测试通过");
}
