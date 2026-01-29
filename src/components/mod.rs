//! ECS组件定义汇聚模块

pub mod animation;
pub mod audio;
pub mod background_music;
pub mod cards;
pub mod combat;
pub mod cultivation;
pub mod dialogue;
pub mod map;
pub mod particle;
pub mod relic;
pub mod screen_effect;
pub mod shop;
pub mod sprite;

// 批量重导出
pub use animation::*;
pub use audio::*;
pub use background_music::*;
pub use cards::*;
pub use combat::*;
pub use cultivation::*;
pub use dialogue::*;
pub use map::*;
pub use particle::*;
pub use relic::*;
pub use screen_effect::*;
pub use shop::*;
pub use sprite::*;