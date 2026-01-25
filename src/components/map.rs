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
// 地图进度系统
// ============================================================================

/// 地图进度资源（持久化）
#[derive(Resource, Debug, Clone)]
pub struct MapProgress {
    /// 所有地图节点
    pub nodes: Vec<MapNode>,
    /// 当前所在的节点ID
    pub current_node_id: Option<u32>,
    /// 当前层数
    pub current_layer: u32,
    /// 游戏是否完成
    pub game_completed: bool,
}

impl MapProgress {
    /// 创建新地图进度
    pub fn new(config: &MapConfig) -> Self {
        let nodes = generate_map_nodes(config, 0);
        Self {
            nodes,
            current_node_id: None,
            current_layer: 0,
            game_completed: false,
        }
    }

    /// 获取当前节点
    pub fn get_current_node(&self) -> Option<&MapNode> {
        self.current_node_id.and_then(|id| self.nodes.iter().find(|n| n.id == id))
    }

    /// 设置当前节点
    pub fn set_current_node(&mut self, node_id: u32) {
        self.current_node_id = Some(node_id);
        if let Some(node) = self.nodes.iter().find(|n| n.id == node_id) {
            self.current_layer = node.position.0 as u32;
        }
    }

    /// 完成当前节点
    pub fn complete_current_node(&mut self) {
        if let Some(node_id) = self.current_node_id {
            if let Some(node) = self.nodes.iter_mut().find(|n| n.id == node_id) {
                node.completed = true;
            }
            // 解锁下一层的所有节点
            self.unlock_next_layer();
        }
    }

    /// 解锁下一层节点
    fn unlock_next_layer(&mut self) {
        let current_layer = self.current_layer;
        for node in &mut self.nodes {
            // 解锁下一层的所有节点
            if node.position.0 as u32 == current_layer + 1 {
                node.unlocked = true;
            }
        }
    }

    /// 检查是否到达Boss
    pub fn is_at_boss(&self) -> bool {
        self.get_current_node()
            .map(|n| n.node_type == NodeType::Boss)
            .unwrap_or(false)
    }

    /// 检查Boss是否被击败
    pub fn is_boss_defeated(&self) -> bool {
        self.nodes.iter().any(|n| n.node_type == NodeType::Boss && n.completed)
    }

    /// 重置地图进度（用于重新开始游戏）
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

impl Default for MapProgress {
    fn default() -> Self {
        // 创建默认配置并生成初始地图
        let config = MapConfig::default();
        Self::new(&config)
    }
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
            layers: 10,
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

            // 随机节点类型（互斥逻辑避免重叠）
            let node_type = if layer == config.layers - 1 && node_idx == 0 {
                NodeType::Boss
            } else if layer == config.layers - 1 {
                NodeType::Elite
            } else if node_idx % 7 == 0 {
                // 每7个节点1个商店（避免与每3个休息重叠）
                NodeType::Shop
            } else if node_idx % 3 == 0 {
                // 每3个节点1个休息
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
