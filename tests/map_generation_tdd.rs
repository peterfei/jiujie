use bevy::prelude::*;
use bevy_card_battler::components::map::{MapNode, NodeType, MapConfig, generate_map_nodes};

#[test]
fn tdd_map_boss_at_final_layer() {
    let config = MapConfig { layers: 10, nodes_per_layer: 4, node_spacing: 100.0 };
    let map = generate_map_nodes(&config, 0);
    
    // 验证：最后一层必须包含 BOSS
    // MapNode.position.0 是层数 (i32)
    let boss_nodes: Vec<&MapNode> = map.iter().filter(|n| n.position.0 == (config.layers - 1) as i32 && n.node_type == NodeType::Boss).collect();
    assert!(!boss_nodes.is_empty(), "第 {} 层必须生成 BOSS 节点", config.layers);
}

#[test]
fn tdd_map_elite_distribution() {
    let config = MapConfig { layers: 10, nodes_per_layer: 4, node_spacing: 100.0 };
    let map = generate_map_nodes(&config, 0);
    
    // 验证：精英怪不应出现在前 3 层 (0, 1, 2)
    let early_elites = map.iter().filter(|n| n.position.0 < 3 && n.node_type == NodeType::Elite).count();
    assert_eq!(early_elites, 0, "前3层不应出现精英怪");
    
    // 验证：中间层必须有精英怪 (这里我们主要验证第 8 层是精英，这是目前代码写死的)
    // 如果要更灵活，可以验证 3-8 层之间
    let mid_elites = map.iter().filter(|n| n.position.0 >= 3 && n.position.0 < (config.layers - 1) as i32 && n.node_type == NodeType::Elite).count();
    assert!(mid_elites > 0, "中间层必须包含精英怪挑战");
}

#[test]
fn tdd_map_connectivity() {
    // 验证路径连通性 (简单版：每层节点至少有一个父节点，第一层除外)
    // 这是一个进阶测试，我们先关注节点类型分布
}
