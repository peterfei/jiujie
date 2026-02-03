//! 验证地图生成节点数量

use bevy::prelude::*;
use bevy_card_battler::components::map::{MapConfig, generate_map_nodes};

#[test]
fn test_default_map_node_counts() {
    let config = MapConfig::default();
    let nodes = generate_map_nodes(&config, 0);
    
    println!("地图配置: layers={}, nodes_per_layer={}", config.layers, config.nodes_per_layer);
    
    for layer in 0..config.layers {
        let count = nodes.iter().filter(|n| n.position.0 == layer as i32).count();
        println!("第 {} 层节点数: {}", layer, count);
        assert_eq!(count as u32, config.nodes_per_layer, "每层节点数应为 {}", config.nodes_per_layer);
    }
}
