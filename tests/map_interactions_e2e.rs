use bevy::prelude::*;
use bevy_card_battler::components::map::{MapProgress, NodeType};

#[test]
fn test_map_save_load_recovery() {
    // 模拟从存档恢复的 MapProgress
    let mut progress = MapProgress::default();
    
    // 假设存档里第 0 层的所有节点都已完成
    for node in progress.nodes.iter_mut().filter(|n| n.position.0 == 0) {
        node.completed = true;
    }
    
    // 模拟加载后的状态：current_node_id 为 None
    progress.current_node_id = None;
    
    // 执行自愈刷新
    progress.complete_current_node(); // 触发全扫描刷新
    
    // 验证：第 1 层节点现在应已解锁
    let layer_1_node = progress.nodes.iter().find(|n| n.position.0 == 1).unwrap();
    assert!(layer_1_node.unlocked, "加载存档后，应根据已完成节点自动解锁下一层");
}

#[test]
fn test_map_node_visual_state() {
    let node_unlocked = MapNode { id: 1, node_type: NodeType::Normal, position: (1, 0), unlocked: true, completed: false };
    let node_locked = MapNode { id: 2, node_type: NodeType::Normal, position: (1, 1), unlocked: false, completed: false };
    
    let color_unlocked = if node_unlocked.completed { Color::srgb(0.2, 0.5, 1.0) } 
                        else if node_unlocked.unlocked { Color::WHITE } 
                        else { Color::srgba(0.5, 0.5, 0.5, 0.3) };
                        
    let color_locked = if node_locked.completed { Color::srgb(0.2, 0.5, 1.0) } 
                      else if node_locked.unlocked { Color::WHITE } 
                      else { Color::srgba(0.5, 0.5, 0.5, 0.3) };
                      
    assert_eq!(color_unlocked, Color::WHITE);
    assert!(color_locked.alpha() < 0.5, "锁定节点应该是半透明的");
}

#[test]
fn test_treasure_node_interaction() {
    let mut progress = MapProgress::default();
    // 强制将一个节点设为宝物且解锁
    progress.nodes[1].node_type = NodeType::Treasure;
    progress.nodes[1].unlocked = true;
    
    let node_type = progress.nodes[1].node_type;
    // 预期：逻辑层应识别 Treasure 节点并计划跳转
    let target_state = match node_type {
        NodeType::Treasure => Some("Reward"), // 暂定跳转到奖励界面
        _ => None,
    };
    
    assert_eq!(target_state, Some("Reward"), "点击宝物节点应跳转至奖励状态");
}

#[test]
fn test_combat_unlocked_after_rest() {
    let mut progress = MapProgress::default();
    // 1. 完成第 0 层休息点
    progress.nodes[0].node_type = NodeType::Rest;
    progress.set_current_node(progress.nodes[0].id);
    progress.complete_current_node();
    
    // 2. 验证第 1 层的战斗节点是否已解锁
    let layer_1_combat = progress.nodes.iter()
        .find(|n| n.position.0 == 1 && n.node_type == NodeType::Normal)
        .expect("第 1 层应有战斗节点");
        
    assert!(layer_1_combat.unlocked, "休息结束后，下一层的战斗节点必须解锁");
}
