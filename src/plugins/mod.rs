//! 游戏插件定义

use bevy::prelude::*;
use crate::states::GameState;

/// 核心游戏插件
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GameState>();
    }
}
