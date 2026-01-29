//! 地图布局对齐 TDD 测试
//!
//! 测试目标：验证地图容器使用正确的对齐方式，确保内容从顶部开始
//!
//! 问题：使用 AlignItems::Center 导致内容在容器中垂直居中，底部节点被遮挡
//!
//! 解决方案：设置 justify_content: JustifyContent::FlexStart

use bevy::prelude::*;
use bevy_card_battler::components::map::{MapNodeButton, MapNodeContainer};
use bevy_card_battler::states::GameState;
use crate::test_utils::*;

mod test_utils;

/// 测试：地图容器应使用 FlexStart 对齐
///
/// 验证容器配置正确，内容从顶部开始而不是居中
#[test]
fn test_map_container_uses_flexstart_alignment() {
    let mut app = create_test_app();

    setup_map_scene(&mut app);
    advance_frames(&mut app, 10);

    let world = app.world_mut();

    // 检查地图容器的对齐配置
    let mut container_query = world.query_filtered::<&Node, With<MapNodeContainer>>();
    let mut found_flexstart = false;
    let mut alignment_info = String::new();

    for node in container_query.iter(world) {
        alignment_info = format!(
            "justify_content: {:?}, align_items: {:?}",
            node.justify_content, node.align_items
        );

        if node.justify_content == JustifyContent::FlexStart {
            found_flexstart = true;
        }
    }

    println!("地图容器对齐配置: {}", alignment_info);

    assert!(found_flexstart, "地图容器应该使用 JustifyContent::FlexStart 以确保内容从顶部开始");
}

/// 测试：地图容器应启用滚动
///
/// 滚动是必要的，因为10层内容会超过70%的容器高度
#[test]
fn test_map_container_has_scroll() {
    let mut app = create_test_app();

    setup_map_scene(&mut app);
    advance_frames(&mut app, 10);

    let world = app.world_mut();

    let mut container_query = world.query_filtered::<&Node, With<MapNodeContainer>>();
    let mut has_scroll = false;

    for node in container_query.iter(world) {
        if node.overflow == Overflow::scroll() {
            has_scroll = true;
            println!("滚动已启用: overflow = {:?}", node.overflow);
        }
    }

    assert!(has_scroll, "地图容器应该启用滚动功能");
}

/// 测试：地图应渲染所有40个节点
///
/// 10层 × 4节点 = 40个节点，都应该被渲染
#[test]
fn test_all_nodes_are_rendered() {
    let mut app = create_test_app();

    setup_map_scene(&mut app);
    advance_frames(&mut app, 10);

    let world = app.world_mut();

    // 统计渲染的节点数量
    let node_count = world.query_filtered::<Entity, With<MapNodeButton>>()
        .iter(world)
        .count();

    println!("渲染的节点数量: {}", node_count);

    assert_eq!(node_count, 40, "应该渲染所有40个节点（10层×4节点）");
}

// ============================================================================
// 测试辅助函数
// ============================================================================

fn setup_map_scene(app: &mut App) {
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Map);
    app.world_mut().run_schedule(StateTransition);
    app.update();
}
