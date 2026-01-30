//! Bevy Card Battler - 主入口点
//!
//! 这是游戏的主入口，负责：
//! 1. 创建Bevy应用
//! 2. 注册所有插件
//! 3. 启动游戏循环

use bevy_card_battler::plugins::{CorePlugin, MenuPlugin, GamePlugin};
use bevy_card_battler::systems::{RelicPlugin, RelicUiPlugin, ShopPlugin, RestPlugin};
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{WgpuSettings, PowerPreference};
use bevy::winit::WinitWindows;
use winit::window::Icon;

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
                    present_mode: bevy::window::PresentMode::AutoNoVsync, // 减少输入延迟
                    ..default()
                }),
                ..default()
            },
        ).set(RenderPlugin {
            render_creation: WgpuSettings {
                power_preference: PowerPreference::HighPerformance,
                ..default()
            }.into(),
            ..default()
        }))
        // 注册图标设置系统
        .add_systems(Startup, set_window_icon)
        // 注册核心插件（包含状态注册）
        .add_plugins(CorePlugin)
        // ... (其他插件保持不变)
        .add_plugins(MenuPlugin)
        .add_plugins(GamePlugin)
        .add_plugins(ShopPlugin)
        .add_plugins(RestPlugin)
        .add_plugins(RelicPlugin)
        .add_plugins(RelicUiPlugin)
        // 运行应用
        .run();
}

/// 设置窗口图标
fn set_window_icon(
    windows: NonSend<WinitWindows>,
) {
    // 在 Bevy 0.15 中，主窗口可以通过 winit 获取
    for window in windows.windows.values() {
        // 使用 image 库读取图片（Bevy 默认包含 image 依赖）
        // 我们直接读取 assets 目录下的 256px 图标
        let path = std::path::Path::new("assets/icons/icon_256.png");
        if let Ok(image) = image::open(path) {
            let image = image.into_rgba8();
            let (width, height) = image.dimensions();
            let rgba = image.into_raw();
            
            if let Ok(icon) = Icon::from_rgba(rgba, width, height) {
                window.set_window_icon(Some(icon));
            }
        }
    }
}
