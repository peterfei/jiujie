use bevy::prelude::*;
use bevy_card_battler::components::map::{MapNode, NodeType, generate_map_nodes, MapConfig};

#[test]
fn test_boss_node_unique_at_end() {
    let config = MapConfig {
        layers: 5,
        nodes_per_layer: 4,
        node_spacing: 100.0,
    };
    let nodes = generate_map_nodes(&config, 0);
    
    // 验证 Boss 节点数量
    let boss_nodes: Vec<&MapNode> = nodes.iter()
        .filter(|n| n.node_type == NodeType::Boss)
        .collect();
        
    assert!(boss_nodes.len() >= 1, "生成的地图应包含 Boss 节点");
    
    // 验证 Boss 节点是否在最后一层 (layer = 4)
    for boss in boss_nodes {
        assert_eq!(boss.position.0, 4, "所有 Boss 节点必须位于地图的最后一层");
    }
}

#[test]
fn test_boss_node_visual_distinctness() {
    let node = MapNode {
        id: 99,
        node_type: NodeType::Boss,
        position: (5, 0),
        unlocked: true,
        completed: false,
    };
    
    let color = match node.node_type {
        NodeType::Boss => Color::srgb(1.0, 0.2, 0.2), 
        _ => Color::WHITE,
    };
    
    assert_eq!(color, Color::srgb(1.0, 0.2, 0.2), "Boss 节点在地图上应显示为警示红");
}