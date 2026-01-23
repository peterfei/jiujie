//! 地图组件和系统

use bevy::prelude::*;

// ============================================================================
// 地图组件
// ============================================================================

/// 地图节点
#[derive(Component, Debug, Clone)]
pub struct MapNode {
    /// 节点ID
    pub id: u32,
    /// 节点类型（普通、精英、Boss、休息等）
    pub node_type: NodeType,
    /// 节点位置（地图坐标）
    pub position: (i32, i32),
    /// 是否已解锁
    pub unlocked: bool,
    /// 是否已完成
    pub completed: bool,
}

/// 节点类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    /// 普通战斗
    Normal,
    /// 精英战斗
    Elite,
    /// Boss战
    Boss,
    /// 休息点
    Rest,
    /// 商店
    Shop,
    /// 宝箱
    Treasure,
    /// 未知
    Unknown,
}

// ============================================================================
// 地图配置
// ============================================================================

/// 地图配置资源
#[derive(Resource, Debug, Clone)]
pub struct MapConfig {
    /// 地图层数
    pub layers: u32,
    /// 每层节点数量
    pub nodes_per_layer: u32,
    /// 节点间距
    pub node_spacing: f32,
}

impl Default for MapConfig {
    fn default() -> Self {
        Self {
            layers: 3,
            nodes_per_layer: 4,
            node_spacing: 150.0,
        }
    }
}

// ============================================================================
// 地图生成系统
// ============================================================================

/// 生成地图节点
pub fn generate_map_nodes(config: &MapConfig, _current_layer: u32) -> Vec<MapNode> {
    let mut nodes = Vec::new();
    let mut id = 0;

    for layer in 0..config.layers {
        for node_idx in 0..config.nodes_per_layer {
            // 第一层总是可解锁的
            let unlocked = layer == 0;

            // 随机节点类型
            let node_type = if layer == config.layers - 1 && node_idx == 0 {
                NodeType::Boss
            } else if layer == config.layers - 1 {
                NodeType::Elite
            } else if node_idx % 3 == 0 {
                NodeType::Rest
            } else {
                NodeType::Normal
            };

            nodes.push(MapNode {
                id,
                node_type,
                position: (layer as i32, node_idx as i32),
                unlocked,
                completed: false,
            });

            id += 1;
        }
    }

    nodes
}
