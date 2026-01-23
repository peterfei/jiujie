//! ECS组件定义

pub mod map;
pub mod combat;

pub use map::{MapNode, NodeType, MapConfig, generate_map_nodes};
pub use combat::{Player, Enemy, EnemyIntent, CombatConfig, CombatState, TurnPhase};
