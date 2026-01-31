//! Bevy Card Battler - 主入口点
//!
//! 这是游戏的主入口，负责：
//! 1. 创建Bevy应用
//! 2. 注册所有插件
//! 3. 启动游戏循环

// 在 Release 构建中隐藏控制台窗口 (Windows)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy_card_battler::plugins::{CorePlugin, MenuPlugin, GamePlugin};
use bevy_card_battler::systems::{RelicPlugin, RelicUiPlugin, ShopPlugin, RestPlugin};
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{WgpuSettings, PowerPreference};
use bevy::winit::WinitWindows;
use winit::window::Icon;
use image::GenericImageView;
use bevy::log::LogPlugin;
use directories::ProjectDirs;

// ============================================================================
// 主函数
// ============================================================================

fn main() {
    // 初始化日志系统 (支持文件输出)
    // 注意：_guard 必须保留在 main 作用域内，否则非阻塞写入器会被提前释放
    let _guard = init_logging();

    // 全局 Panic 捕获：记录到日志，防止静默闪退
    std::panic::set_hook(Box::new(|info| {
        error!("🔥 程序发生严重错误 (Panic): {:?}", info);
        // 在 Windows GUI 模式下，Panic 不会显示在控制台，必须记下来
        eprintln!("{:?}", info); 
    }));

    // 创建应用并运行
    App::new()
        // 添加默认插件（渲染、输入、音频等）
        .add_plugins(DefaultPlugins
            // 禁用默认 LogPlugin，因为我们已经手动初始化了 tracing
            .build().disable::<LogPlugin>()
            .set(
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

/// 初始化日志系统
/// 返回 worker guard，必须保持存活直到程序结束
fn init_logging() -> Option<tracing_appender::non_blocking::WorkerGuard> {
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::fmt;
    use tracing_appender::rolling::{RollingFileAppender, Rotation};

    // 获取标准的用户数据目录 (跨平台)
    // Windows: C:\Users\Alice\AppData\Roaming\PeterFei\Jiujie\logs
    // macOS:   /Users/Alice/Library/Application Support/com.PeterFei.Jiujie/logs
    // Linux:   /home/alice/.config/jiujie/logs
    let log_dir = if let Some(proj_dirs) = ProjectDirs::from("com", "PeterFei", "Jiujie") {
        proj_dirs.data_dir().join("logs")
    } else {
        // 回退方案：当前目录 (在安装目录下可能因权限失败，但好过没有)
        std::path::PathBuf::from("logs")
    };

    // 1. 文件输出层 (jiujie.log.YYYY-MM-DD)
    let file_appender = RollingFileAppender::new(Rotation::DAILY, &log_dir, "jiujie.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    
    let file_layer = fmt::layer()
        .with_ansi(false)
        .with_writer(non_blocking)
        .with_target(false)
        .with_thread_names(true);

    // 2. 控制台输出层 (仅在 Debug 模式或非 Windows 子系统下启用)
    let stdout_layer = fmt::layer()
        .with_ansi(true)
        .pretty();

    // 3. 过滤器 (EnvFilter)
    let filter_layer = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "wgpu=error,bevy=info,jiujie=debug".into());

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(stdout_layer)
        .with(file_layer)
        .init();

    // 记录启动信息，方便定位日志位置
    info!("日志系统初始化完成。日志路径: {:?}", log_dir);

    Some(guard)
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
