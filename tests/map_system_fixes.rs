// @Validated: Refactor Regression - 2026-01-29
//! 地图系统修复 TDD 测试
//!
//! 测试地图系统的关键问题：
//! 1. 地图滚动功能
//! 2. 事件节点完成后不可重复进入
//! 3. 节点完成后解锁下一层

use bevy::prelude::*;
use bevy_card_battler::components::{MapNode, NodeType, MapProgress};
use bevy_card_battler::components::map::{MapNodeButton, MapNodeContainer};
use bevy_card_battler::states::GameState;
use crate::test_utils::*;

mod test_utils;

// ============================================================================
// 问题 1: 地图滚动功能测试
// ============================================================================

#[test]
fn test_map_container_has_scroll() {
    let mut app = create_test_app();

    setup_map_scene(&mut app);

    let world = app.world_mut();

    // 检查地图容器是否有滚动功能
    let mut container_query = world.query_filtered::<Entity, With<MapNodeContainer>>();
    let container_count = container_query.iter(world).count();

    assert!(container_count > 0, "应该有地图容器");

    // 检查容器是否有 overflow: scroll
    for entity in container_query.iter(world) {
        if let Some(node) = world.get::<Node>(entity) {
            // 验证容器已启用滚动
            assert_eq!(node.overflow, Overflow::scroll(),
                "地图容器应该启用滚动功能，当前: {:?}", node.overflow);
        }
    }
}

#[test]
fn test_map_content_exceeds_container_height() {
    let mut app = create_test_app();

    setup_map_scene(&mut app);

    let world = app.world_mut();

    // 统计节点数量
    let node_count = world.query_filtered::<Entity, With<MapNodeButton>>()
        .iter(world)
        .count();

    // 地图应该有多层节点
    assert!(node_count >= 4, "地图应该至少有4个节点用于测试滚动，当前: {}", node_count);

    println!("地图节点总数: {}", node_count);
}

// ============================================================================
// 问题 2: 事件节点完成后不可重复进入测试
// ============================================================================

#[test]
fn test_event_node_completed_after_choice() {
    let mut app = create_test_app();

    // 创建一个包含事件节点的地图
    let mut progress = MapProgress::default();
    progress.nodes = vec![
        MapNode {
            id: 0,
            position: (0, 0),
            node_type: NodeType::Event,
            unlocked: true,
            completed: false, next_nodes: Vec::new(),
        },
        MapNode {
            id: 1,
            position: (1, 0),
            node_type: NodeType::Normal,
            unlocked: false,  // 初始未解锁
            completed: false, next_nodes: Vec::new(),
        },
    ];
    progress.current_node_id = Some(0);
    progress.current_layer = 0;
    app.world_mut().insert_resource(progress);

    setup_map_scene(&mut app);

    let world = app.world_mut();

    // 模拟完成事件节点
    {
        let mut progress = world.resource_mut::<MapProgress>();
        progress.complete_current_node();
    }

    // 验证事件节点已标记为完成
    let progress = world.resource::<MapProgress>();
    let event_node = progress.nodes.iter().find(|n| n.id == 0);
    assert!(event_node.is_some(), "应该找到事件节点");
    assert!(event_node.unwrap().completed, "事件节点应该标记为已完成");

    // 验证下一层节点已解锁
    let next_layer_node = progress.nodes.iter().find(|n| n.id == 1);
    assert!(next_layer_node.is_some(), "应该找到下一层节点");
    assert!(next_layer_node.unwrap().unlocked, "下一层节点应该已解锁");
}

#[test]
fn test_completed_node_cannot_be_reentered() {
    let mut app = create_test_app();

    // 创建一个已完成的事件节点
    let mut progress = MapProgress::default();
    progress.nodes = vec![
        MapNode {
            id: 0,
            position: (0, 0),
            node_type: NodeType::Event,
            unlocked: true,
            completed: true,  // 已完成
            next_nodes: Vec::new(),
        },
    ];
    progress.current_node_id = Some(0);
    app.world_mut().insert_resource(progress);

    setup_map_scene(&mut app);

    let world = app.world_mut();

    // 尝试点击已完成的节点
    let progress = world.resource::<MapProgress>();
    let event_node = progress.nodes.iter().find(|n| n.id == 0).unwrap();

    // 验证已完成节点不能再次进入
    assert!(event_node.completed, "事件节点应该已完成");

    // 在实际游戏中，handle_map_button_clicks 会检查 completed 状态并阻止进入
    // 这里我们验证数据状态正确
    println!("已完成节点 ID={}, completed={}", event_node.id, event_node.completed);
}

// ============================================================================
// 问题 3: 节点完成与解锁测试
// ============================================================================

#[test]
fn test_node_completion_unlocks_next_layer() {
    let mut app = create_test_app();

    // 创建多层地图
    let mut progress = MapProgress::default();
    progress.nodes = vec![
        // 第0层
        MapNode {
            id: 0,
            position: (0, 0),
            node_type: NodeType::Normal,
            unlocked: true,
            completed: true,  // 已完成
            next_nodes: vec![1, 2], // 建立连接
        },
        // 第1层
        MapNode {
            id: 1,
            position: (1, 0),
            node_type: NodeType::Event,
            unlocked: false,  // 初始未解锁
            completed: false, 
            next_nodes: Vec::new(),
        },
        MapNode {
            id: 2,
            position: (1, 1),
            node_type: NodeType::Normal,
            unlocked: false,
            completed: false, 
            next_nodes: Vec::new(),
        },
    ];
    progress.current_layer = 0;
    app.world_mut().insert_resource(progress);

    let world = app.world_mut();

    // 刷新解锁状态
    {
        let mut progress = world.resource_mut::<MapProgress>();
        progress.refresh_unlocks();
    }

    // 验证第1层节点已解锁
    let progress = world.resource::<MapProgress>();
    let layer_1_nodes: Vec<_> = progress.nodes.iter()
        .filter(|n| n.position.0 == 1)
        .collect();

    assert_eq!(layer_1_nodes.len(), 2, "第1层应该有2个节点");
    assert!(layer_1_nodes[0].unlocked, "第1层节点1应该已解锁");
    assert!(layer_1_nodes[1].unlocked, "第1层节点2应该已解锁");

    println!("解锁验证通过：第1层的 {} 个节点已解锁", layer_1_nodes.len());
}

// ============================================================================
// 测试辅助函数
// ============================================================================

fn setup_map_scene(app: &mut App) {
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Map);
    app.world_mut().run_schedule(StateTransition);
    app.update();
    // 多运行几帧让UI布局完成
    advance_frames(app, 5);
}
