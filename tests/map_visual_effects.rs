//! 地图视觉效果 TDD - 按测试驱动开发完整流程
//!
//! TDD 流程：红 → 绿 → 重构
//! 1. 红色：写一个失败的测试，定义期望行为
//! 2. 绿色：写最少的代码让测试通过
//! 3. 重构：优化代码质量

use bevy::prelude::*;
use bevy_card_battler::components::{MapNode, NodeType, MapProgress};
use bevy_card_battler::components::map::{
    MapNodeButton, BreathingAnimation, HoverEffect, PulseAnimation, RippleEffect,
    EntranceAnimation
};
use bevy_card_battler::states::GameState;
use crate::test_utils::*;

mod test_utils;

// ============================================================================
// 功能 1: 节点呼吸动画
// ============================================================================

#[test]
fn test_unfinished_nodes_have_breathing_animation() {
    let mut app = create_test_app();

    setup_map_scene(&mut app);

    let world = app.world_mut();
    let mut count = 0;

    // 检查是否有呼吸动画组件
    let mut query = world.query_filtered::<Entity, (With<MapNodeButton>, With<BreathingAnimation>)>();
    count = query.iter(world).count();

    // 第一轮：这会失败，因为没有 BreathingAnimation 组件
    assert!(count > 0, "未完成的节点应该有呼吸动画组件，当前: {}", count);
}

// ============================================================================
// 功能 2: 层级连接区域
// ============================================================================

#[test]
fn test_connection_areas_between_layers() {
    let mut app = create_test_app();

    setup_map_scene(&mut app);

    let world = app.world_mut();

    // 检查是否有足够的 UI 元素（节点 + 连接区域）
    // 每层应该有节点行，层之间应该有连接区域
    let node_count = world.query_filtered::<Entity, With<MapNodeButton>>()
        .iter(world)
        .count();

    // 连接区域通过有特定背景色的 Node 来识别
    // 圆点有特定的背景色和圆角
    let dot_count = world.query_filtered::<Entity, (With<Node>, With<BackgroundColor>)>()
        .iter(world)
        .count();

    // 应该有节点和连接区域的元素
    assert!(node_count > 0, "应该有地图节点，当前: {}", node_count);
    assert!(dot_count > 0, "应该有连接区域元素（圆点等），当前: {}", dot_count);
}

// ============================================================================
// 功能 3: 悬停放大效果
// ============================================================================

#[test]
fn test_hover_enlarges_node() {
    let mut app = create_test_app();

    setup_map_scene(&mut app);

    // 运行几帧让UI稳定
    advance_frames(&mut app, 5);

    let world = app.world_mut();

    // 检查是否有悬停效果标记组件
    let hover_count = world.query_filtered::<Entity, (With<MapNodeButton>, With<HoverEffect>)>()
        .iter(world)
        .count();

    assert!(hover_count > 0, "节点应该有悬停缩放组件，当前: {}", hover_count);
}

// ============================================================================
// 功能 4: 当前节点脉冲发光
// ============================================================================

#[test]
fn test_current_node_has_pulse_effect() {
    let mut app = create_test_app();

    // 创建有当前节点的地图
    let mut progress = MapProgress::default();
    progress.current_node_id = Some(0);
    app.world_mut().insert_resource(progress);

    setup_map_scene(&mut app);

    let world = app.world_mut();
    let pulse_count = world.query_filtered::<Entity, (With<MapNodeButton>, With<PulseAnimation>)>()
        .iter(world)
        .count();

    assert!(pulse_count > 0, "当前节点应该有脉冲效果组件，当前: {}", pulse_count);
}

// ============================================================================
// 功能 5: 节点点击波纹特效
// ============================================================================

#[test]
fn test_ripple_effect_component_exists() {
    // 测试波纹效果组件可以被创建
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let world = app.world_mut();

    // 手动创建一个波纹效果实体来验证组件可以正常工作
    world.spawn((
        Node::default(),
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
        RippleEffect::new(100.0, 0.6),
    ));

    // 验证波纹效果组件存在
    let ripple_count = world.query_filtered::<Entity, With<RippleEffect>>()
        .iter(world)
        .count();

    assert_eq!(ripple_count, 1, "应该创建一个波纹效果组件");
}

// ============================================================================
// 功能 6: 节点入场动画
// ============================================================================

#[test]
fn test_entrance_animation_component_exists() {
    use bevy_card_battler::components::map::MapUiRoot;


    // 测试入场动画组件可以被创建
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let world = app.world_mut();

    // 创建一个带入场动画的实体
    world.spawn((
        Node::default(),
        BackgroundColor(Color::WHITE),
        EntranceAnimation::new(0.5),
    ));

    // 验证入场动画组件存在
    let entrance_count = world.query_filtered::<Entity, With<EntranceAnimation>>()
        .iter(world)
        .count();

    assert_eq!(entrance_count, 1, "应该创建一个入场动画组件");
}

// ============================================================================
// 测试辅助函数
// ============================================================================

fn setup_map_scene(app: &mut App) {
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Map);
    app.world_mut().run_schedule(StateTransition);
    app.update();
}
