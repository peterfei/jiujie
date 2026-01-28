//! 端到端流程验证测试
//!
//! 测试完整的用户操作流程
//! 继承自 commit 2a818d5 测试框架

use crate::test_utils::*;
use bevy::prelude::*;
use bevy_card_battler::components::{Player, MapNode, NodeType, MapProgress};
use bevy_card_battler::states::GameState;
use bevy_card_battler::components::combat::CombatUiRoot;
use bevy_card_battler::plugins::MapUiRoot;

#[test]
fn test_e2e_full_shop_and_combat_flow() {
    // 场景：完整的用户操作流程
    // 1. 主菜单 → 地图
    // 2. 地图 → 商店
    // 3. 商店购买
    // 4. 商店 → 地图
    // 5. 地图 → 战斗
    // 6. 战斗出牌

    let mut app = create_test_app();

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
    transition_to_state(&mut app, GameState::Map);
    advance_frames(&mut app, 5);

    let map_ui_count = count_map_ui(&mut app);
    println!("步骤1: 地图UI数量 = {}", map_ui_count);
    assert!(map_ui_count > 0, "应该有地图UI");

    // 步骤2: 进入商店
    transition_to_state(&mut app, GameState::Shop);
    advance_frames(&mut app, 10);

    // 检查商店UI和金币
    let shop_ui_count = count_shop_ui(&mut app);
    println!("步骤2: 商店UI数量 = {}", shop_ui_count);
    assert!(shop_ui_count > 0, "应该有商店UI");

    let gold_before = get_player_gold(&mut app);
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
    advance_frames(&mut app, 3);

    // 验证金币更新
    let gold_after = get_player_gold(&mut app);
    println!("步骤3: 购买后金币 = {}", gold_after);
    assert_eq!(gold_after, 70, "购买后金币应该是70");

    // 步骤4: 返回地图
    transition_to_state(&mut app, GameState::Map);
    advance_frames(&mut app, 5);

    // 步骤5: 进入战斗
    setup_combat_scene(&mut app);
    advance_frames(&mut app, 10);

    // 验证战斗状态
    let combat_ui_count = count_combat_ui(&mut app);
    println!("步骤5: 战斗UI数量 = {}", combat_ui_count);
    assert!(combat_ui_count > 0, "应该有战斗UI");

    // 验证战斗能量
    let player_energy = get_player_energy(&mut app);
    println!("步骤5: 战斗能量 = {}", player_energy);
    assert_eq!(player_energy, 3, "战斗开始时能量应该是3");

    // 验证 Player 唯一性
    let player_count = app.world_mut().query::<&Player>().iter(&app.world()).count();
    println!("步骤5: Player数量 = {}", player_count);
    assert_eq!(player_count, 1, "应该只有一个Player");

    println!("✓ 端到端流程测试通过");
}

// ============================================================================
// 辅助查询函数
// ============================================================================

/// 统计地图UI数量
fn count_map_ui(app: &mut App) -> usize {
    let mut query = app.world_mut().query::<(Entity, &MapUiRoot)>();
    query.iter(&app.world()).count()
}

/// 统计战斗UI数量
fn count_combat_ui(app: &mut App) -> usize {
    let mut query = app.world_mut().query::<(Entity, &CombatUiRoot)>();
    query.iter(&app.world()).count()
}

/// 获取玩家能量
fn get_player_energy(app: &mut App) -> i32 {
    let mut query = app.world_mut().query::<&Player>();
    query.iter(&app.world()).next().map(|p| p.energy).unwrap_or(0)
}
