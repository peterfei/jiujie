//! Bevy Card Battler - 主入口点
//!
//! 这是游戏的主入口，负责：
//! 1. 创建Bevy应用
//! 2. 注册所有插件
//! 3. 启动游戏循环

use bevy_card_battler::plugins::{CorePlugin, MenuPlugin};
use bevy_card_battler::systems::AnimationPlugin;
use bevy_card_battler::states::GameState;
use bevy::prelude::*;

// ============================================================================
// 主函数
// ============================================================================

fn main() {
    // 创建应用并运行
    App::new()
        // 添加默认插件（渲染、输入、音频等）
        .add_plugins(DefaultPlugins.set(
            WindowPlugin {
                primary_window: Some(Window {
                    title: format!("{} v{}", bevy_card_battler::GAME_NAME, bevy_card_battler::VERSION),
                    resolution: (1280., 720.).into(),
                    ..default()
                }),
                ..default()
            },
        ))
        // 初始化游戏状态为MainMenu
        .init_state::<GameState>()
        // 注册核心插件
        .add_plugins(CorePlugin)
        // 注册主菜单插件
        .add_plugins(MenuPlugin)
        // 注册动画插件
        .add_plugins(AnimationPlugin)
        // 运行应用
        .run();
}
