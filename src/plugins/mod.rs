//! 游戏插件定义

use bevy::prelude::*;
use crate::states::GameState;

/// 核心游戏插件
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GameState>();
        // 应用启动时设置2D相机（用于渲染UI）
        app.add_systems(Startup, setup_camera);
    }
}

/// 主菜单UI插件
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        // 在进入MainMenu状态时设置主菜单
        app.add_systems(OnEnter(GameState::MainMenu), setup_main_menu);
        // 在退出MainMenu状态时清理主菜单
        app.add_systems(OnExit(GameState::MainMenu), cleanup_main_menu);
        // 处理按钮点击
        app.add_systems(Update, handle_button_clicks.run_if(in_state(GameState::MainMenu)));

        // 在进入Map状态时设置地图UI（临时占位）
        app.add_systems(OnEnter(GameState::Map), setup_map_ui);
    }
}

// ============================================================================
// 核心系统
// ============================================================================

/// 设置2D相机（用于渲染UI和游戏场景）
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

// ============================================================================
// 主菜单系统
// ============================================================================

/// 设置主菜单UI
fn setup_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 加载中文字体
    let chinese_font: Handle<Font> = asset_server.load("fonts/Arial Unicode.ttf");

    // 创建根节点（全屏容器）
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
            ..default()
        })
        .with_children(|parent| {
            // 游戏标题
            parent.spawn((
                Text::new("Bevy Card Battler"),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                // 居中对齐
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // 开始游戏按钮
            parent.spawn((
                Node {
                width: Val::Px(200.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                Button,
                StartGameButton,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("开始游戏"),
                    TextFont {
                        font: chinese_font.clone(),
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });

            // 退出按钮（可选，暂时注释）
            /*
            parent.spawn((
                Button {
                    width: Val::Px(200.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                QuitGameButton,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("退出"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
            */
        });
}

/// 清理主菜单UI
fn cleanup_main_menu(mut commands: Commands, query: Query<Entity, With<Node>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// ============================================================================
// 组件标记
// ============================================================================

/// 开始游戏按钮标记
#[derive(Component)]
struct StartGameButton;

/// 退出游戏按钮标记（未使用）
#[derive(Component)]
struct QuitGameButton;

// ============================================================================
// 按钮交互系统
// ============================================================================

/// 处理按钮点击事件
fn handle_button_clicks(
    mut next_state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (With<StartGameButton>, Without<QuitGameButton>),
    >,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        if matches!(interaction, Interaction::Pressed) {
            // 点击开始游戏按钮
            info!("开始游戏按钮被点击");
            next_state.set(GameState::Map);
        } else if matches!(interaction, Interaction::Hovered) {
            // 悬停效果
            *color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3));
        } else {
            // 默认颜色
            *color = BackgroundColor(Color::srgb(0.2, 0.2, 0.2));
        }
    }
}

// ============================================================================
// 地图系统（临时占位）
// ============================================================================

/// 设置地图UI（临时占位界面）
fn setup_map_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 加载中文字体
    let chinese_font: Handle<Font> = asset_server.load("fonts/Arial Unicode.ttf");

    // 创建根节点（全屏容器）
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
            ..default()
        })
        .with_children(|parent| {
            // 地图标题
            parent.spawn((
                Text::new("地图界面"),
                TextFont {
                    font: chinese_font.clone(),
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // 提示文字
            parent.spawn((
                Text::new("（地图功能将在Sprint 2实现）"),
                TextFont {
                    font: chinese_font,
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}
