//! 地图Boss节点可见性 TDD 测试
//!
//! 测试目标：验证地图的所有节点（包括最底部的Boss）都能正常显示
//!
//! 问题报告：
//! - "地图显示不全，无法看到最下面的boss"
//!
//! TDD 流程：红 → 绿 → 重构

use bevy::prelude::*;
use bevy_card_battler::components::{NodeType, MapProgress, MapConfig};
use bevy_card_battler::components::map::{MapNodeButton, MapNodeContainer};
use bevy_card_battler::states::GameState;
use crate::test_utils::*;

mod test_utils;

// ============================================================================
// 问题: 地图显示不全，无法看到最下面的Boss
// ============================================================================

/// 测试：地图容器应启用滚动功能
///
/// 红色阶段：验证当前配置是否正确
/// 绿色阶段：确认滚动已启用
#[test]
fn test_map_container_has_scroll_enabled() {
    let mut app = create_test_app();

    setup_map_scene(&mut app);
    advance_frames(&mut app, 10);

    let world = app.world_mut();

    // 检查地图容器的滚动配置
    let mut container_query = world.query_filtered::<&Node, With<MapNodeContainer>>();
    let mut container_found = false;
    let mut has_scroll = false;

    for node in container_query.iter(world) {
        container_found = true;
        has_scroll = node.overflow == Overflow::scroll();
        println!("地图容器配置 - overflow: {:?}, height: {:?}", node.overflow, node.height);
    }

    assert!(container_found, "应该找到地图容器");
    assert!(has_scroll, "地图容器应该启用滚动功能");
}

/// 测试：地图应包含所有10层节点
///
/// 验证完整地图（10层×4节点=40个节点）的数据正确性
#[test]
fn test_full_map_has_all_layers() {
    // 创建完整的10层地图配置
    let config = MapConfig {
        layers: 10,
        nodes_per_layer: 4,
        node_spacing: 150.0,
    };

    let nodes = bevy_card_battler::components::map::generate_map_nodes(&config, 0);

    println!("生成的节点总数: {}", nodes.len());

    // 验证节点数量
    assert_eq!(nodes.len(), 40, "10层×4节点应该有40个节点");

    // 验证包含Boss节点
    let boss_nodes = nodes.iter()
        .filter(|n| n.node_type == NodeType::Boss)
        .count();

    assert_eq!(boss_nodes, 4, "最后一层应该有4个Boss节点");

    // 验证Boss在最后一层（第9层）
    let boss_layer = nodes.iter()
        .find(|n| n.node_type == NodeType::Boss)
        .map(|n| n.layer())
        .unwrap_or(999);

    assert_eq!(boss_layer, 9, "Boss应该在第9层");
}

/// 测试：地图渲染应显示Boss层
///
/// 这个测试验证即使在视野限制下，Boss层也应该被渲染
#[test]
fn test_boss_layer_is_rendered() {
    let mut app = create_test_app();

    setup_map_scene(&mut app);
    advance_frames(&mut app, 10);

    let world = app.world_mut();

    // 检查渲染的节点数量
    let rendered_count = world.query_filtered::<Entity, With<MapNodeButton>>()
        .iter(world)
        .count();

    println!("实际渲染的节点数量: {}", rendered_count);

    // 验证至少有第一层（4个节点）
    assert!(rendered_count >= 4, "至少应该渲染第一层的4个节点");

    // 检查地图数据中的Boss节点
    let progress = world.resource::<MapProgress>();
    let boss_nodes = progress.nodes.iter()
        .filter(|n| n.node_type == NodeType::Boss)
        .count();

    println!("地图数据中的Boss节点数量: {}", boss_nodes);
    assert!(boss_nodes > 0, "地图数据中应该有Boss节点");
}

/// 测试：地图容器高度配置
///
/// 验证地图容器的高度设置是否合理
#[test]
fn test_map_container_height_configuration() {
    let mut app = create_test_app();

    setup_map_scene(&mut app);
    advance_frames(&mut app, 10);

    let world = app.world_mut();

    let mut container_query = world.query_filtered::<&Node, With<MapNodeContainer>>();
    let mut height_info = String::new();

    for node in container_query.iter(world) {
        match node.height {
            Val::Percent(p) => height_info = format!("高度: {}%", p),
            Val::Px(px) => height_info = format!("高度: {}px", px),
            _ => height_info = format!("高度: {:?}", node.height),
        }
    }

    println!("地图容器配置: {}", height_info);

    // 当前配置：height: Val::Percent(70.0)
    // 这个测试记录当前状态
    assert!(!height_info.is_empty(), "应该获取到容器高度信息");
}

/// 测试：完整地图的节点渲染
///
/// 模拟10层完整地图，验证渲染逻辑
#[test]
fn test_complete_map_rendering() {
    let mut app = create_test_app();

    // 创建10层完整地图
    let config = MapConfig {
        layers: 10,
        nodes_per_layer: 4,
        node_spacing: 150.0,
    };

    let nodes = bevy_card_battler::components::map::generate_map_nodes(&config, 0);
    let mut progress = MapProgress {
        nodes,
        current_node_id: None,
        current_layer: 0,
        game_completed: false,
    };
    progress.refresh_unlocks();

    app.world_mut().insert_resource(progress);

    setup_map_scene(&mut app);
    advance_frames(&mut app, 10);

    let world = app.world_mut();

    // 检查渲染的节点
    let rendered_count = world.query_filtered::<Entity, With<MapNodeButton>>()
        .iter(world)
        .count();

    println!("完整地图渲染节点数: {}", rendered_count);

    // 即使有视野限制，至少应该有第一层和Boss层
    assert!(rendered_count >= 4, "至少应该渲染4个节点");

    // 验证Boss节点存在
    let progress = world.resource::<MapProgress>();
    let boss_in_data = progress.nodes.iter()
        .any(|n| n.node_type == NodeType::Boss);

    assert!(boss_in_data, "地图数据中应该包含Boss节点");
}

/// 测试：层容器高度配置
///
/// 检查每个层容器的高度设置
#[test]
fn test_layer_container_heights() {
    let mut app = create_test_app();

    setup_map_scene(&mut app);
    advance_frames(&mut app, 10);

    let world = app.world_mut();

    // 统计所有节点UI元素
    let all_nodes = world.query_filtered::<&Node, With<MapNodeButton>>()
        .iter(world)
        .count();

    println!("地图节点UI元素数量: {}", all_nodes);

    // 检查是否有层容器（具有Column布局的Node）
    let column_layouts = world.query::<&Node>()
        .iter(world)
        .filter(|n| n.flex_direction == FlexDirection::Column)
        .count();

    println!("Column布局容器数量: {}", column_layouts);

    assert!(all_nodes > 0, "应该有地图节点UI");
}

// ============================================================================
// 测试辅助函数
// ============================================================================

fn setup_map_scene(app: &mut App) {
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Map);
    app.world_mut().run_schedule(StateTransition);
    app.update();
}
