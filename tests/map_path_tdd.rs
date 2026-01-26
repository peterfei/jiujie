use bevy::prelude::*;
use bevy_card_battler::components::map::{generate_map_nodes, MapConfig, NodeType};

#[test]
fn test_boss_accessibility() {
    let config = MapConfig {
        layers: 5,
        nodes_per_layer: 4,
        node_spacing: 100.0,
    };
    let nodes = generate_map_nodes(&config, 0);
    
    // 验证最后一层 (layer 4) 是否有可达节点
    let last_layer_nodes: Vec<_> = nodes.iter()
        .filter(|n| n.position.0 == 4)
        .collect();
        
    assert!(!last_layer_nodes.is_empty(), "最后一层不能为空，否则玩家无法通关");
    assert!(last_layer_nodes.iter().all(|n| n.node_type == NodeType::Boss), "最后一层的所有节点都应是 Boss 类型以确保路径闭环");
}
