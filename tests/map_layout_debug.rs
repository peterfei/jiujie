//! 地图布局调试测试
//!
//! 检查实际的UI尺寸，诊断为什么没有滚动条

use bevy::prelude::*;
use bevy_card_battler::components::map::{MapNodeButton, MapNodeContainer};
use bevy_card_battler::states::GameState;
use crate::test_utils::*;

mod test_utils;

/// 测试：检查地图容器和内容的实际高度
///
/// 滚动条只有在内容高度 > 容器高度时才会出现
#[test]
fn test_map_container_and_content_heights() {
    let mut app = create_test_app();

    setup_map_scene(&mut app);
    advance_frames(&mut app, 20);  // 让UI完全布局

    let world = app.world_mut();

    // 检查地图容器的尺寸
    let mut container_query = world.query_filtered::<&Node, With<MapNodeContainer>>();
    let mut container_height = 0.0;
    let mut container_width = 0.0;

    for node in container_query.iter(world) {
        // Bevy UI 的 Node 组件存储的是配置值，不是计算后的实际值
        // 实际计算后的值在 UiHash 系统中，测试环境可能无法访问
        println!("地图容器配置:");
        println!("  - width: {:?}", node.width);
        println!("  - height: {:?}", node.height);
        println!("  - overflow: {:?}", node.overflow);
        println!("  - flex_direction: {:?}", node.flex_direction);
        println!("  - justify_content: {:?}", node.justify_content);
        println!("  - align_items: {:?}", node.align_items);

        // 提取配置的高度值用于估算
        if let Val::Percent(p) = node.height {
            container_height = p;  // 70.0 表示 70%
        }
        if let Val::Percent(p) = node.width {
            container_width = p;  // 90.0 表示 90%
        }
    }

    // 统计所有节点和层
    let node_count = world.query_filtered::<&Node, With<MapNodeButton>>()
        .iter(world)
        .count();

    // 查找所有使用 Column 布局的容器（层容器）
    let layer_containers = world.query::<&Node>()
        .iter(world)
        .filter(|n| n.flex_direction == FlexDirection::Column)
        .count();

    println!("=== 布局调试信息 ===");
    println!("容器高度: {} px", container_height);
    println!("容器宽度: {} px", container_width);
    println!("渲染节点数: {}", node_count);
    println!("层容器数: {}", layer_containers);

    // 估算内容高度
    // 每个节点约60px + 间距，10层 = 10 * 60 = 600px
    // 连接区域：9个 × 40px = 360px
    // 层间距：10层 × 40px = 400px
    // 总计约：600 + 360 + 400 = 1360px
    let estimated_content_height = 1360.0;

    println!("估算内容高度: {} px", estimated_content_height);
    println!("内容高度 > 容器高度: {}", estimated_content_height > container_height);

    // 这个测试用于调试，不会失败
    assert!(container_height > 0.0, "容器应该有高度");
}

/// 测试：检查根容器的配置
///
/// 根容器的 align_items 可能会影响地图容器的宽度计算
#[test]
fn test_root_container_configuration() {
    let mut app = create_test_app();

    setup_map_scene(&mut app);
    advance_frames(&mut app, 10);

    let world = app.world_mut();

    // 检查所有根节点（没有父级的节点）
    let mut root_query = world.query_filtered::<&Node, ()>();
    let root_nodes: Vec<_> = root_query.iter(world)
        .filter(|n| n.flex_direction == FlexDirection::Column)
        .collect();

    println!("=== 根容器配置 ===");
    for node in root_nodes {
        println!("flex_direction: {:?}", node.flex_direction);
        println!("align_items: {:?}", node.align_items);
        println!("width: {:?}", node.width);
        println!("height: {:?}", node.height);
        println!("---");
    }
}

// ============================================================================
// 测试辅助函数
// ============================================================================

fn setup_map_scene(app: &mut App) {
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Map);
    app.world_mut().run_schedule(StateTransition);
    app.update();
}
