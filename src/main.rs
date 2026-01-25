//! Bevy Card Battler - 主入口点
//!
//! 这是游戏的主入口，负责：
//! 1. 创建Bevy应用
//! 2. 注册所有插件
//! 3. 启动游戏循环

use bevy_card_battler::plugins::{CorePlugin, MenuPlugin, GamePlugin};
use bevy_card_battler::systems::{AnimationPlugin, SpritePlugin, ParticlePlugin, ScreenEffectPlugin, RelicPlugin, RelicUiPlugin, ShopPlugin, RestPlugin};
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
        // 注册核心插件（包含状态注册）
        .add_plugins(CorePlugin)
        // 注册主菜单插件
        .add_plugins(MenuPlugin)
        // 注册游戏逻辑核心插件（已包含 Animation, Sprite, Particle, ScreenEffect 等插件）
        .add_plugins(GamePlugin)
        // 注册商店插件
        .add_plugins(ShopPlugin)
        // 注册休息插件
        .add_plugins(RestPlugin)
        // 注册遗物插件
        .add_plugins(RelicPlugin)
        // 注册遗物UI插件
        .add_plugins(RelicUiPlugin)
        // 运行应用
        .run();
}
