//! 游戏插件定义

use bevy::prelude::*;
use crate::states::GameState;
use crate::components::{MapNode, NodeType, MapConfig, generate_map_nodes, Player, Enemy, CombatState, TurnPhase};

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

        // 在进入Map状态时设置地图UI
        app.add_systems(OnEnter(GameState::Map), setup_map_ui);
        // 在退出Map状态时清理地图UI
        app.add_systems(OnExit(GameState::Map), cleanup_map_ui);
        // 处理地图界面按钮点击
        app.add_systems(Update, handle_map_button_clicks.run_if(in_state(GameState::Map)));

        // 在进入Combat状态时设置战斗UI
        app.add_systems(OnEnter(GameState::Combat), setup_combat_ui);
        // 在退出Combat状态时清理战斗UI
        app.add_systems(OnExit(GameState::Combat), cleanup_combat_ui);
        // 处理战斗界面按钮点击
        app.add_systems(Update, handle_combat_button_clicks.run_if(in_state(GameState::Combat)));
        // 更新战斗UI显示
        app.add_systems(Update, update_combat_ui.run_if(in_state(GameState::Combat)));
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
fn cleanup_main_menu(mut commands: Commands, query: Query<Entity, (With<Node>, Without<MapUiRoot>)>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

/// 清理地图UI
fn cleanup_map_ui(mut commands: Commands, query: Query<Entity, With<MapUiRoot>>) {
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

/// 地图UI根节点标记
#[derive(Component)]
struct MapUiRoot;

/// 地图节点按钮标记
#[derive(Component)]
struct MapNodeButton {
    node_id: u32,
}

/// 返回主菜单按钮标记
#[derive(Component)]
struct BackToMenuButton;

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

/// 处理地图界面按钮点击
fn handle_map_button_clicks(
    mut next_state: ResMut<NextState<GameState>>,
    mut button_queries: ParamSet<(
        Query<(&Interaction, &MapNodeButton, &mut BackgroundColor)>,
        Query<(&Interaction, &mut BackgroundColor), With<BackToMenuButton>>,
    )>,
) {
    // 处理地图节点点击
    for (interaction, node_btn, mut color) in button_queries.p0().iter_mut() {
        if matches!(interaction, Interaction::Pressed) {
            // 点击地图节点 - 进入战斗
            info!("地图节点 {} 被点击，进入战斗", node_btn.node_id);
            next_state.set(GameState::Combat);
        } else if matches!(interaction, Interaction::Hovered) {
            // 悬停效果（稍微变亮）
            if let Color::Srgba(ref c) = color.0 {
                *color = BackgroundColor(Color::srgb(
                    (c.red + 0.2).min(1.0),
                    (c.green + 0.2).min(1.0),
                    (c.blue + 0.2).min(1.0),
                ));
            }
        } else {
            // 恢复默认颜色（这里简化处理，实际应该根据节点类型恢复）
            *color = BackgroundColor(Color::srgb(0.3, 0.5, 0.3));
        }
    }

    // 处理返回按钮点击
    for (interaction, mut color) in button_queries.p1().iter_mut() {
        if matches!(interaction, Interaction::Pressed) {
            info!("返回主菜单按钮被点击");
            next_state.set(GameState::MainMenu);
        } else if matches!(interaction, Interaction::Hovered) {
            *color = BackgroundColor(Color::srgb(0.4, 0.4, 0.4));
        } else {
            *color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3));
        }
    }
}

// ============================================================================
// 地图系统
// ============================================================================

/// 设置地图UI
fn setup_map_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 加载中文字体
    let chinese_font: Handle<Font> = asset_server.load("fonts/Arial Unicode.ttf");

    // 创建地图配置
    let config = MapConfig::default();

    // 生成地图节点
    let nodes = generate_map_nodes(&config, 0);

    // 创建地图UI根容器
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            MapUiRoot,
        ))
        .with_children(|parent| {
            // 地图标题
            parent.spawn((
                Text::new("地图"),
                TextFont {
                    font: chinese_font.clone(),
                    font_size: 36.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));

            // 地图节点容器
            parent
                .spawn(Node {
                    width: Val::Percent(90.0),
                    height: Val::Percent(70.0),
                    justify_content: JustifyContent::SpaceEvenly,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|map_parent| {
                    // 按层显示节点
                    for layer in 0..config.layers {
                        // 创建层容器
                        map_parent
                            .spawn(Node {
                                width: Val::Percent(100.0),
                                height: Val::Auto,
                                justify_content: JustifyContent::SpaceEvenly,
                                align_items: AlignItems::Center,
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(20.0),
                                ..default()
                            })
                            .with_children(|layer_parent| {
                                // 在该层中添加节点
                                for node in &nodes {
                                    if node.position.0 == layer as i32 {
                                        spawn_map_node(
                                            layer_parent,
                                            node,
                                            &chinese_font,
                                            &config,
                                        );
                                    }
                                }
                            });
                    }
                });

            // 返回按钮
            parent
                .spawn((
                    Node {
                        width: Val::Px(150.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                    Button,
                    BackToMenuButton,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("返回主菜单"),
                        TextFont {
                            font: chinese_font,
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

/// 生成单个地图节点UI
fn spawn_map_node(
    parent: &mut ChildBuilder,
    node: &MapNode,
    font: &Handle<Font>,
    _config: &MapConfig,
) {
    // 根据节点类型选择颜色
    let node_color = match node.node_type {
        NodeType::Normal => Color::srgb(0.3, 0.5, 0.3),  // 绿色
        NodeType::Elite => Color::srgb(0.6, 0.3, 0.1),   // 橙色
        NodeType::Boss => Color::srgb(0.7, 0.1, 0.1),    // 红色
        NodeType::Rest => Color::srgb(0.3, 0.4, 0.6),    // 蓝色
        NodeType::Shop => Color::srgb(0.5, 0.4, 0.2),    // 黄色
        NodeType::Treasure => Color::srgb(0.5, 0.2, 0.5), // 紫色
        NodeType::Unknown => Color::srgb(0.4, 0.4, 0.4), // 灰色
    };

    // 节点名称
    let node_name = match node.node_type {
        NodeType::Normal => "普通",
        NodeType::Elite => "精英",
        NodeType::Boss => "Boss",
        NodeType::Rest => "休息",
        NodeType::Shop => "商店",
        NodeType::Treasure => "宝箱",
        NodeType::Unknown => "未知",
    };

    // 节点未解锁时的颜色（变暗）
    let display_color = if node.unlocked {
        node_color
    } else {
        // 创建暗色版本
        match node.node_type {
            NodeType::Normal => Color::srgb(0.12, 0.2, 0.12),
            NodeType::Elite => Color::srgb(0.24, 0.12, 0.04),
            NodeType::Boss => Color::srgb(0.28, 0.04, 0.04),
            NodeType::Rest => Color::srgb(0.12, 0.16, 0.24),
            NodeType::Shop => Color::srgb(0.2, 0.16, 0.08),
            NodeType::Treasure => Color::srgb(0.2, 0.08, 0.2),
            NodeType::Unknown => Color::srgb(0.16, 0.16, 0.16),
        }
    };

    let mut entity = parent.spawn((
        Node {
            width: Val::Px(80.0),
            height: Val::Px(80.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(display_color),
        MapNodeButton { node_id: node.id },
    ));

    // 如果节点已解锁，添加按钮组件
    if node.unlocked {
        entity.insert(Button);
    }

    entity.with_children(|node_parent| {
            // 节点类型图标（用文字表示）
            node_parent.spawn((
                Text::new(get_node_icon(node.node_type)),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // 节点名称
            node_parent.spawn((
                Text::new(node_name),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::top(Val::Px(5.0)),
                    ..default()
                },
            ));
        });
}

/// 获取节点图标（使用文字代替emoji以确保兼容性）
fn get_node_icon(node_type: NodeType) -> &'static str {
    match node_type {
        NodeType::Normal => "战",
        NodeType::Elite => "精",
        NodeType::Boss => "王",
        NodeType::Rest => "休",
        NodeType::Shop => "店",
        NodeType::Treasure => "宝",
        NodeType::Unknown => "?",
    }
}

// ============================================================================
// 战斗系统
// ============================================================================

/// 战斗UI根节点标记
#[derive(Component)]
struct CombatUiRoot;

/// 结束回合按钮标记
#[derive(Component)]
struct EndTurnButton;

/// 返回地图按钮标记（战斗结束）
#[derive(Component)]
struct ReturnToMapButton;

// 战斗UI文本标记组件
#[derive(Component)]
struct EnemyHpText;

#[derive(Component)]
struct EnemyIntentText;

#[derive(Component)]
struct PlayerHpText;

#[derive(Component)]
struct PlayerEnergyText;

#[derive(Component)]
struct TurnText;

/// 设置战斗UI
fn setup_combat_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let chinese_font: Handle<Font> = asset_server.load("fonts/Arial Unicode.ttf");

    // 创建玩家和敌人实体
    commands.spawn(Player::default());
    commands.spawn(Enemy::new(0, "哥布林", 30));

    // 初始化战斗状态
    commands.insert_resource(CombatState::default());

    // 创建战斗UI根容器
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            CombatUiRoot,
        ))
        .with_children(|parent| {
            // 顶部：敌人区域
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(40.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|enemy_area| {
                    // 敌人信息面板
                    enemy_area
                        .spawn(Node {
                            width: Val::Px(200.0),
                            height: Val::Px(150.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(10.0),
                            ..default()
                        })
                        .with_children(|enemy_panel| {
                            // 敌人名称
                            enemy_panel.spawn((
                                Text::new("哥布林"),
                                TextFont {
                                    font: chinese_font.clone(),
                                    font_size: 24.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));

                            // 敌人血量
                            enemy_panel.spawn((
                                Text::new("HP: 30/30"),
                                TextFont {
                                    font: chinese_font.clone(),
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(1.0, 0.3, 0.3)),
                                EnemyHpText,
                            ));

                            // 敌人意图
                            enemy_panel.spawn((
                                Text::new("意图: 攻击(10)"),
                                TextFont {
                                    font: chinese_font.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(1.0, 0.8, 0.0)),
                                EnemyIntentText,
                            ));
                        });
                });

            // 中部：玩家区域
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(40.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|player_area| {
                    // 玩家信息面板
                    player_area
                        .spawn(Node {
                            width: Val::Px(300.0),
                            height: Val::Px(150.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(10.0),
                            ..default()
                        })
                        .with_children(|player_panel| {
                            // 玩家血量
                            player_panel.spawn((
                                Text::new("HP: 80/80"),
                                TextFont {
                                    font: chinese_font.clone(),
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.3, 1.0, 0.3)),
                                PlayerHpText,
                            ));

                            // 玩家能量
                            player_panel.spawn((
                                Text::new("能量: 3/3"),
                                TextFont {
                                    font: chinese_font.clone(),
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.3, 0.6, 1.0)),
                                PlayerEnergyText,
                            ));

                            // 当前回合
                            player_panel.spawn((
                                Text::new("第 1 回合"),
                                TextFont {
                                    font: chinese_font.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                                TurnText,
                            ));
                        });
                });

            // 底部：控制按钮区域
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(20.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(20.0),
                    ..default()
                })
                .with_children(|button_area| {
                    // 结束回合按钮
                    button_area
                        .spawn((
                            Node {
                                width: Val::Px(120.0),
                                height: Val::Px(40.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.5, 0.3)),
                            Button,
                            EndTurnButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("结束回合"),
                                TextFont {
                                    font: chinese_font.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                    // 返回地图按钮
                    button_area
                        .spawn((
                            Node {
                                width: Val::Px(120.0),
                                height: Val::Px(40.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                            Button,
                            ReturnToMapButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("返回地图"),
                                TextFont {
                                    font: chinese_font,
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                });
        });
}

/// 清理战斗UI
fn cleanup_combat_ui(
    mut commands: Commands,
    ui_query: Query<Entity, With<CombatUiRoot>>,
    player_query: Query<Entity, With<Player>>,
    enemy_query: Query<Entity, With<Enemy>>,
) {
    // 清理UI
    for entity in ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    // 清理玩家实体
    for entity in player_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    // 清理敌人实体
    for entity in enemy_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    // 移除战斗状态资源
    commands.remove_resource::<CombatState>();
}

/// 处理战斗界面按钮点击
fn handle_combat_button_clicks(
    mut next_state: ResMut<NextState<GameState>>,
    mut combat_state: ResMut<CombatState>,
    mut player_query: Query<&mut Player>,
    _enemy_query: Query<&mut Enemy>,
    mut button_queries: ParamSet<(
        Query<(&Interaction, &mut BackgroundColor), With<EndTurnButton>>,
        Query<(&Interaction, &mut BackgroundColor), With<ReturnToMapButton>>,
    )>,
) {
    // 处理结束回合按钮
    for (interaction, mut color) in button_queries.p0().iter_mut() {
        if matches!(interaction, Interaction::Pressed) {
            info!("结束回合按钮被点击");
            // 简单实现：切换到敌人回合
            combat_state.phase = TurnPhase::EnemyTurn;

            // TODO: 敌人AI行动逻辑
            // 敌人攻击玩家
            if let Ok(mut player) = player_query.get_single_mut() {
                player.take_damage(10);
                info!("玩家受到10点伤害，剩余HP: {}", player.hp);
            }

            // 检查战斗是否结束
            if let Ok(player) = player_query.get_single() {
                if player.hp <= 0 {
                    info!("玩家败北！");
                    // TODO: 游戏结束逻辑
                }
            }

            // 新回合开始
            if let Ok(mut player) = player_query.get_single_mut() {
                player.start_turn();
            }
            combat_state.phase = TurnPhase::PlayerAction;
        } else if matches!(interaction, Interaction::Hovered) {
            *color = BackgroundColor(Color::srgb(0.4, 0.6, 0.4));
        } else {
            *color = BackgroundColor(Color::srgb(0.3, 0.5, 0.3));
        }
    }

    // 处理返回地图按钮
    for (interaction, mut color) in button_queries.p1().iter_mut() {
        if matches!(interaction, Interaction::Pressed) {
            info!("返回地图按钮被点击");
            next_state.set(GameState::Map);
        } else if matches!(interaction, Interaction::Hovered) {
            *color = BackgroundColor(Color::srgb(0.4, 0.4, 0.4));
        } else {
            *color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3));
        }
    }
}

/// 实时更新战斗UI
fn update_combat_ui(
    player_query: Query<&Player>,
    enemy_query: Query<&Enemy>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<EnemyHpText>>,
        Query<&mut Text, With<EnemyIntentText>>,
        Query<&mut Text, With<PlayerHpText>>,
        Query<&mut Text, With<PlayerEnergyText>>,
        Query<&mut Text, With<TurnText>>,
    )>,
) {
    // 获取玩家和敌人数据
    if let Ok(player) = player_query.get_single() {
        if let Ok(enemy) = enemy_query.get_single() {
            // 更新敌人HP
            if let Ok(mut hp_text) = text_queries.p0().get_single_mut() {
                hp_text.0 = format!("HP: {}/{}", enemy.hp, enemy.max_hp);
            }

            // 更新敌人意图
            if let Ok(mut intent_text) = text_queries.p1().get_single_mut() {
                let intent_str = match enemy.intent {
                    crate::components::EnemyIntent::Attack { damage } => format!("攻击({})", damage),
                    crate::components::EnemyIntent::Defend { block } => format!("防御({})", block),
                    crate::components::EnemyIntent::Buff { strength } => format!("强化({})", strength),
                    crate::components::EnemyIntent::Wait => "等待".to_string(),
                };
                intent_text.0 = format!("意图: {}", intent_str);
            }

            // 更新玩家HP
            if let Ok(mut hp_text) = text_queries.p2().get_single_mut() {
                hp_text.0 = format!("HP: {}/{}", player.hp, player.max_hp);
            }

            // 更新玩家能量
            if let Ok(mut energy_text) = text_queries.p3().get_single_mut() {
                energy_text.0 = format!("能量: {}/{}", player.energy, player.max_energy);
            }

            // 更新回合数
            if let Ok(mut turn_text) = text_queries.p4().get_single_mut() {
                turn_text.0 = format!("第 {} 回合", player.turn);
            }
        }
    }
}
