//! ECS组件定义

pub mod map;
pub mod combat;
pub mod cards;

pub use map::{MapNode, NodeType, MapConfig, generate_map_nodes};
pub use combat::{Player, Enemy, EnemyIntent, CombatConfig, CombatState, TurnPhase};
pub use cards::{
    Card, CardType, CardEffect, CardRarity,
    DrawPile, DiscardPile, Hand,
    DeckConfig,
};
