pub mod test_utils;
use crate::test_utils::*;
use bevy::prelude::*;
use bevy_card_battler::states::GameState;
use bevy_card_battler::components::map::{MapProgress, MapNode, NodeType, ConnectorDot};
use bevy_card_battler::systems::map::setup_map_ui;

#[test]
fn test_map_ui_culling_performance() {
    let mut app = create_test_app();
    
    // 1. 构造一个 50 层的地图，全部已完成
    let mut nodes = Vec::new();
    let layers = 50;
    let width = 3;
    let mut id_counter = 0;
    
    for layer in 0..layers {
        for _ in 0..width {
            let mut next_nodes = Vec::new();
            if layer < layers - 1 {
                next_nodes.push(id_counter + width); // 简单线性连接
            }
            nodes.push(MapNode {
                id: id_counter,
                node_type: NodeType::Normal,
                position: (layer as i32, 0),
                unlocked: true,
                completed: true, // 全部已完成
                next_nodes,
            });
            id_counter += 1;
        }
    }
    
    // 设置当前层为 50 (最后)
    app.insert_resource(MapProgress {
        nodes,
        current_node_id: Some(id_counter - 1),
        current_layer: (layers - 1) as u32,
        game_completed: false, // 必须提供完整字段
    });

    // 2. 运行 setup_map_ui (通过进入 Map 状态)
    // 此时应该触发剔除逻辑
    transition_to_state(&mut app, GameState::Map);
    app.update();

    // 3. 统计生成的 UI 节点数量
    let dot_count = app.world_mut().query::<&ConnectorDot>().iter(app.world()).count();
    
    println!("Generated ConnectorDots: {}", dot_count);

    // 4. 断言
    // 如果没有剔除，50层 * 3节点 * 1连接 * 15点 = 2250 点
    // 我们期望剔除生效，只渲染最近的层级。
    // 如果渲染了所有层级，这个断言会失败。
    assert!(dot_count < 1000, "Map UI generating too many dots! Culling logic might be broken. Count: {}", dot_count);
}
