use bevy::prelude::*;
use bevy_card_battler::components::map::{MapProgress, MapConfig, NodeType};

#[test]
fn test_map_node_completion_unlocks_next() {
    let mut progress = MapProgress::default();
    
    // 初始状态：第一层 (0) 已解锁，第二层 (1) 锁定
    let layer_1_node_id = progress.nodes.iter().find(|n| n.position.0 == 1).unwrap().id;
    assert!(!progress.nodes.iter().find(|n| n.id == layer_1_node_id).unwrap().unlocked);
    
    // 模拟点击并完成第一层 (0) 的某个节点
    let layer_0_node_id = progress.nodes.iter().find(|n| n.position.0 == 0).unwrap().id;
    progress.set_current_node(layer_0_node_id);
    progress.complete_current_node();
    
    // 验证：第二层 (1) 的节点现在应已解锁
    let node_after = progress.nodes.iter().find(|n| n.id == layer_1_node_id).unwrap();
    assert!(node_after.unlocked, "点击完成当前层节点后，下一层节点应被解锁");
}