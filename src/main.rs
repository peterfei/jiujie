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
        // 注册图标设置系统 (监听窗口创建，确保句柄已就绪)
        .add_systems(Update, set_window_icon.run_if(any_with_component::<Window>))
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
    mut is_set: Local<bool>,
) {
    if *is_set { return; }

    // 在 Bevy 0.15 中，主窗口可以通过 winit 获取
    for window in windows.windows.values() {
        let path = std::path::Path::new("assets/icons/icon_256.png");
        
        match image::open(path) {
            Ok(image) => {
                let image = image.into_rgba8();
                let (width, height) = image.dimensions();
                let rgba = image.into_raw();
                
                match Icon::from_rgba(rgba, width, height) {
                    Ok(icon) => {
                        window.set_window_icon(Some(icon));
                        *is_set = true;
                        info!("【发布准备】窗口图标已成功加载并注入识海");
                    },
                    Err(e) => error!("【图标失败】RGBA转换错误: {}", e),
                }
            },
            Err(e) => {
                // 如果是第一次尝试失败，我们不立即报错，因为可能还没准备好
                static mut FAIL_COUNT: u32 = 0;
                unsafe {
                    FAIL_COUNT += 1;
                    if FAIL_COUNT % 60 == 0 { // 每秒打印一次
                        error!("【图标失败】无法在路径 {:?} 找到图标文件: {}", path, e);
                    }
                }
            }
        }
    }
}
