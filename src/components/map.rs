//! 地图组件和系统定义

use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use rand::Rng;

// ============================================================================
// 地图组件
// ============================================================================

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct MapNode {
    pub id: u32,
    pub node_type: NodeType,
    pub position: (i32, i32), // (layer, index)
    pub unlocked: bool,
    pub completed: bool,
    /// 可前往的后续节点ID
    pub next_nodes: Vec<u32>,
}

impl MapNode {
    pub fn layer(&self) -> u32 {
        self.position.0 as u32
    }
}

/// 节点类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    /// 机缘事件
    Event,
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
        let mut progress = Self {
            nodes,
            current_node_id: None,
            current_layer: 0,
            game_completed: false,
        };
        progress.refresh_unlocks();
        progress
    }

    /// 从存档恢复并刷新解锁
    pub fn from_save(nodes: Vec<MapNode>, current_node_id: Option<u32>, current_layer: u32) -> Self {
        let mut progress = Self {
            nodes,
            current_node_id,
            current_layer,
            game_completed: false,
        };
        progress.refresh_unlocks();
        progress
    }

    /// 获取当前节点
    pub fn get_current_node(&self) -> Option<&MapNode> {
        self.current_node_id.and_then(|id| self.nodes.iter().find(|n| n.id == id))
    }

    /// 设置当前节点
    pub fn set_current_node(&mut self, node_id: u32) {
        if let Some(node) = self.nodes.iter().find(|n| n.id == node_id) {
            if node.unlocked {
                self.current_node_id = Some(node_id);
                self.current_layer = node.position.0 as u32;
            } else {
                warn!("【地图逻辑】尝试前往未解锁的节点: {}", node_id);
            }
        }
    }

    /// 完成当前节点
    pub fn complete_current_node(&mut self) {
        if let Some(node_id) = self.current_node_id {
            if let Some(node) = self.nodes.iter_mut().find(|n| n.id == node_id) {
                node.completed = true;
                self.current_layer = node.position.0 as u32;
            }
        }
        self.refresh_unlocks();
    }

    pub fn refresh_unlocks(&mut self) {
        // 重置所有节点的解锁状态（除了第一层）
        for node in &mut self.nodes {
            if node.position.0 == 0 {
                node.unlocked = true;
            } else {
                node.unlocked = false;
            }
        }

        // 收集所有已完成节点的可达节点
        let mut to_unlock = Vec::new();
        for node in &self.nodes {
            if node.completed {
                for next_id in &node.next_nodes {
                    to_unlock.push(*next_id);
                }
            }
        }

        // 解锁这些节点
        for id in to_unlock {
            if let Some(node) = self.nodes.iter_mut().find(|n| n.id == id) {
                node.unlocked = true;
            }
        }
        
        // 更新当前所在层级
        let max_completed_layer = self.nodes.iter()
            .filter(|n| n.completed)
            .map(|n| n.position.0)
            .max()
            .unwrap_or(-1);

        if max_completed_layer >= 0 {
            self.current_layer = max_completed_layer as u32;
        }
    }

    pub fn is_at_boss(&self) -> bool {
        self.get_current_node()
            .map(|n| n.node_type == NodeType::Boss)
            .unwrap_or(false)
    }

    pub fn is_boss_defeated(&self) -> bool {
        self.nodes.iter().any(|n| n.node_type == NodeType::Boss && n.completed)
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

impl Default for MapProgress {
    fn default() -> Self {
        let config = MapConfig::default();
        Self::new(&config)
    }
}

#[derive(Resource, Debug, Clone)]
pub struct MapConfig {
    pub layers: u32,
    pub nodes_per_layer: u32,
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

pub fn generate_map_nodes(config: &MapConfig, _current_layer: u32) -> Vec<MapNode> {
    let mut nodes = Vec::new();
    let mut id = 0;

    for layer in 0..config.layers {
        for node_idx in 0..config.nodes_per_layer {
            let unlocked = layer == 0;
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let rand_val = rng.gen_range(0..100);

            let node_type = if layer == config.layers - 1 {
                NodeType::Boss 
            } else if layer == config.layers - 2 {
                NodeType::Elite
            } else if rand_val < 15 {
                NodeType::Event 
            } else if node_idx % 7 == 0 {
                NodeType::Shop
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
                next_nodes: Vec::new(),
            });

            id += 1;
        }
    }

    // 生成路径连接 (拓扑连线)
    let nodes_per_layer = config.nodes_per_layer as i32;
    for layer in 0..(config.layers - 1) as i32 {
        for idx in 0..nodes_per_layer {
            let current_id = (layer * nodes_per_layer + idx) as u32;
            let next_layer = layer + 1;
            
            // 找到当前节点
            if let Some(node_pos) = nodes.iter().position(|n| n.id == current_id) {
                let mut next_ids = Vec::new();
                
                // 连向下一层位置相近的节点 (左、中、右)
                for offset in -1..=1 {
                    let next_idx = idx + offset;
                    if next_idx >= 0 && next_idx < nodes_per_layer {
                        let next_id = (next_layer * nodes_per_layer + next_idx) as u32;
                        next_ids.push(next_id);
                    }
                }
                
                // 随机保留 1-3 条路径
                use rand::seq::SliceRandom;
                let mut rng = rand::thread_rng();
                next_ids.shuffle(&mut rng);
                let count = rng.gen_range(1..=next_ids.len());
                nodes[node_pos].next_nodes = next_ids[0..count].to_vec();
            }
        }
    }

    nodes
}

// ============================================================================
// UI 和 视觉组件
// ============================================================================

#[derive(Component)]
pub struct MapUiRoot;

#[derive(Component)]
pub struct MapNodeContainer;

#[derive(Component)]
pub struct MapNodeButton {
    pub node_id: u32,
}

#[derive(Component)]
pub struct BreakthroughButtonMarker;

#[derive(Component)]
pub struct OriginalSize {
    pub width: Val,
    pub height: Val,
}

#[derive(Component)]
pub struct BreathingAnimation {
    pub min_scale: f32,
    pub max_scale: f32,
    pub speed: f32,
    pub current: f32,
    pub phase: f32,
}

impl Default for BreathingAnimation {
    fn default() -> Self {
        Self {
            min_scale: 0.95,
            max_scale: 1.05,
            speed: 2.0,
            current: 1.0,
            phase: 0.0,
        }
    }
}

#[derive(Component)]
pub struct RippleEffect {
    pub radius: f32,
    pub max_radius: f32,
    pub alpha: f32,
    pub duration: f32,
    pub elapsed: f32,
}

impl RippleEffect {
    pub fn new(max_radius: f32, duration: f32) -> Self {
        Self {
            radius: 0.0,
            max_radius,
            alpha: 1.0,
            duration,
            elapsed: 0.0,
        }
    }
}

#[derive(Component)]
pub struct EntranceAnimation {
    pub duration: f32,
    pub elapsed: f32,
    pub start_scale: f32,
    pub start_alpha: f32,
}

impl EntranceAnimation {
    pub fn new(duration: f32) -> Self {
        Self {
            duration,
            elapsed: 0.0,
            start_scale: 0.0,
            start_alpha: 0.0,
        }
    }
}

#[derive(Component)]
pub struct ConnectorDot {
    pub offset: f32,
}

#[derive(Component)]
pub struct ConnectionLine {
    pub from_node_id: u32,
    pub to_node_id: u32,
}

#[derive(Component)]
pub struct PulseAnimation {
    pub intensity: f32,
    pub speed: f32,
    pub phase: f32,
}

impl Default for PulseAnimation {
    fn default() -> Self {
        Self {
            intensity: 0.5,
            speed: 3.0,
            phase: 0.0,
        }
    }
}

#[derive(Component)]
pub struct HoverEffect {
    pub base_scale: f32,
    pub hover_scale: f32,
}

impl Default for HoverEffect {
    fn default() -> Self {
        Self {
            base_scale: 1.0,
            hover_scale: 1.15,
        }
    }
}

#[derive(Component)]
pub struct DimmedEffect;
