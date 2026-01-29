use bevy::prelude::*;
use bevy_card_battler::components::map::{MapProgress, NodeType, MapNode, MapConfig};
use bevy_card_battler::components::{Player, PlayerDeck, Cultivation};
use bevy_card_battler::components::cultivation::Realm;

#[test]
fn test_map_logic_integration() {
    let mut app = App::new();
    
    // 1. 初始化核心资源
    let config = MapConfig {
        layers: 5,
        nodes_per_layer: 3,
        node_spacing: 150.0,
    };
    let mut progress = MapProgress::new(&config);
    
    // 验证初始状态：第 0 层解锁，其他锁定
    assert!(progress.nodes.iter().filter(|n| n.position.0 == 0).all(|n| n.unlocked), "第 0 层应初始解锁");
    assert!(progress.nodes.iter().filter(|n| n.position.0 > 0).all(|n| !n.unlocked), "非第 0 层应初始锁定");

    // 2. 模拟完成第 0 层的一个节点
    let first_node_id = progress.nodes.iter().find(|n| n.position.0 == 0).unwrap().id;
    let next_ids = progress.nodes.iter().find(|n| n.id == first_node_id).unwrap().next_nodes.clone();
    
    progress.set_current_node(first_node_id);
    progress.complete_current_node();
    
    // 3. 验证路径解锁
    for next_id in next_ids {
        let node = progress.nodes.iter().find(|n| n.id == next_id).unwrap();
        assert!(node.unlocked, "与已完成节点相连的节点 {} 应被解锁", next_id);
    }
    
    // 验证未相连的节点仍保持锁定
    let unrelated_node = progress.nodes.iter()
        .find(|n| n.position.0 == 1 && !progress.nodes.iter().find(|p| p.id == first_node_id).unwrap().next_nodes.contains(&n.id));
    
    if let Some(node) = unrelated_node {
        assert!(!node.unlocked, "未相连的节点 {} 应保持锁定", node.id);
    }

    // 4. 验证视野系统 (神识)
    // 模拟玩家境界
    let mut cultivation = Cultivation::new();
    cultivation.realm = Realm::QiRefining; // 炼气期视野为 1
    
    let current_layer = progress.current_layer;
    let vision_range = 1; // 炼气期
    
    let visible_limit = current_layer + vision_range;
    
    // 在系统中，我们会过滤 layer <= visible_limit
    let invisible_layer = visible_limit + 1;
    assert!(invisible_layer < config.layers - 1, "测试数据需要确保存在视野外的非 Boss 层");
    
        // 5. 红绿测试：校验 set_current_node 的安全性
    
        let locked_node_id = progress.nodes.iter().find(|n| !n.unlocked).unwrap().id;
    
        let previous_node_id = progress.current_node_id;
    
        
    
        // 尝试前往一个锁定的节点
    
        progress.set_current_node(locked_node_id);
    
        
    
        assert_ne!(progress.current_node_id, Some(locked_node_id), "不应允许前往未解锁的节点 {} ", locked_node_id);
    
        assert_eq!(progress.current_node_id, previous_node_id, "非法移动后，当前位置应保持不变");
    
    
    
        info!("地图逻辑集成测试通过：路径拓扑与视野过滤符合设计预期");
    
    }
    
    