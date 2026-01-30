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
                // 优先高性能，但不再强制，确保兼容性
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

fn set_window_icon(
    // 确保我们只执行一次，无论成功与否
    mut is_set: Local<bool>,
    windows: Query<(Entity, &Window), With<bevy::window::PrimaryWindow>>,
    winit_windows: NonSend<WinitWindows>,
) {
    if *is_set {
        return;
    }

    for (entity, _) in windows.iter() {
        // 标记为已尝试，防止下一帧重复执行
        *is_set = true;

        if let Some(winit_window) = winit_windows.get_window(entity) {
            let icon_path = "assets/icons/icon_256.png";
            let path = std::path::Path::new(icon_path);
            
            match image::open(path) {
                Ok(image) => {
                    let (width, height) = image.dimensions();
                    let rgba = image.into_rgba8().into_vec();
                    match Icon::from_rgba(rgba, width, height) {
                        Ok(icon) => winit_window.set_window_icon(Some(icon)),
                        Err(e) => warn!("【图标】创建图标对象失败: {:?}", e),
                    }
                    info!("【图标】窗口图标已设置");
                }
                Err(e) => {
                    // 仅在开发环境警告，发布环境如果缺失通常由exe资源处理
                    warn!("【图标】无法加载图标文件 (路径: {}): {:?}。如果是开发环境 cargo run，请确保工作目录正确。", icon_path, e);
                }
            }
        }
    }
}
