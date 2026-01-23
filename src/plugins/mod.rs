//! 游戏插件定义

use bevy::prelude::*;
use crate::states::GameState;
use crate::components::{MapNode, NodeType, MapConfig, generate_map_nodes, Player, Enemy, CombatState, TurnPhase, Hand, DrawPile, DiscardPile, DeckConfig, CardEffect, Card, EnemyUiMarker, PlayerUiMarker, EnemyAttackEvent, CharacterType, SpriteMarker};
use crate::systems::sprite::spawn_character_sprite;

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
        // 在进入Combat状态时抽牌
        app.add_systems(OnEnter(GameState::Combat), draw_cards_on_combat_start);
        // 在退出Combat状态时清理战斗UI
        app.add_systems(OnExit(GameState::Combat), cleanup_combat_ui);
        // 处理战斗界面按钮点击
        app.add_systems(Update, handle_combat_button_clicks.run_if(in_state(GameState::Combat)));
        // 更新战斗UI显示
        app.add_systems(Update, update_combat_ui.run_if(in_state(GameState::Combat)));
        // 回合开始时抽牌
        app.add_systems(Update, draw_cards_on_turn_start.run_if(in_state(GameState::Combat)));
        // 更新手牌UI
        app.add_systems(Update, update_hand_ui.run_if(in_state(GameState::Combat)));
        // 处理出牌
        app.add_systems(Update, handle_card_play.run_if(in_state(GameState::Combat)));
        // 检查战斗结束
        app.add_systems(Update, check_combat_end.run_if(in_state(GameState::Combat)));
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
struct PlayerBlockText;

#[derive(Component)]
struct TurnText;

// 卡牌UI标记组件
#[derive(Component)]
struct HandCard {
    card_id: u32,
}

#[derive(Component)]
struct DrawPileText;

#[derive(Component)]
struct DiscardPileText;

#[derive(Component)]
struct HandCountText;

#[derive(Component)]
struct HandArea;

/// 设置战斗UI
fn setup_combat_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let chinese_font: Handle<Font> = asset_server.load("fonts/Arial Unicode.ttf");

    // 创建玩家和敌人实体
    commands.spawn(Player::default());
    commands.spawn(Enemy::new(0, "哥布林", 30));

    // 创建敌人精灵
    spawn_character_sprite(
        &mut commands,
        CharacterType::NormalEnemy,
        Vec3::new(0.0, 100.0, 10.0),
        Vec2::new(70.0, 100.0),
    );

    // 创建玩家精灵
    spawn_character_sprite(
        &mut commands,
        CharacterType::Player,
        Vec3::new(0.0, -200.0, 10.0),
        Vec2::new(80.0, 120.0),
    );

    // 初始化战斗状态
    commands.insert_resource(CombatState::default());

    // 创建牌组
    let deck_config = DeckConfig::default();
    commands.insert_resource(deck_config.clone());

    // 计算初始抽牌后剩余的卡牌
    let initial_draw = 5.min(deck_config.starting_deck.len());
    let drawn_cards: Vec<Card> = deck_config.starting_deck.iter().take(initial_draw).cloned().collect();
    let remaining_deck: Vec<Card> = deck_config.starting_deck.iter().skip(initial_draw).cloned().collect();

    commands.spawn(DrawPile::new(remaining_deck));
    commands.spawn(DiscardPile::new());

    // 创建手牌并添加初始卡牌
    let mut hand = Hand::new(10);
    for card in drawn_cards {
        hand.add_card(card);
    }
    info!("战斗开始：初始抽了 {} 张牌到手牌", hand.cards.len());
    commands.spawn(hand);

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
                        .spawn((
                            Node {
                            width: Val::Px(200.0),
                            height: Val::Px(150.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(10.0),
                            ..default()
                        },
                        EnemyUiMarker,
                    ))
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
                        .spawn((
                            Node {
                            width: Val::Px(300.0),
                            height: Val::Px(150.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(10.0),
                            ..default()
                        },
                        PlayerUiMarker,
                    ))
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

                            // 玩家护甲
                            player_panel.spawn((
                                Text::new("护甲: 0"),
                                TextFont {
                                    font: chinese_font.clone(),
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.8, 0.5, 0.2)),
                                PlayerBlockText,
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

            // 底部：控制区域（左侧：牌组信息，右侧：控制按钮，下方：手牌区）
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(40.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(10.0)),
                    row_gap: Val::Px(10.0),
                    ..default()
                })
                .with_children(|control_area| {
                    // 上半部分：牌组信息 + 控制按钮
                    control_area
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(50.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|top_row| {
                            // 左侧：抽牌堆和弃牌堆信息
                            top_row
                                .spawn(Node {
                                    width: Val::Px(200.0),
                                    height: Val::Px(50.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(20.0),
                                    ..default()
                                })
                                .with_children(|deck_info| {
                                    // 抽牌堆
                                    deck_info.spawn((
                                        Text::new("抽牌堆: 10"),
                                        TextFont {
                                            font: chinese_font.clone(),
                                            font_size: 14.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                                        DrawPileText,
                                    ));

                                    // 弃牌堆
                                    deck_info.spawn((
                                        Text::new("弃牌堆: 0"),
                                        TextFont {
                                            font: chinese_font.clone(),
                                            font_size: 14.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                                        DiscardPileText,
                                    ));
                                });

                            // 右侧：控制按钮
                            top_row
                                .spawn(Node {
                                    width: Val::Px(280.0),
                                    height: Val::Px(50.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(10.0),
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
                                                    font: chinese_font.clone(),
                                                    font_size: 16.0,
                                                    ..default()
                                                },
                                                TextColor(Color::WHITE),
                                            ));
                                        });
                                });
                        });

                    // 下半部分：手牌区域
                    control_area
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Auto,
                                min_height: Val::Px(100.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(10.0),
                                flex_wrap: FlexWrap::Wrap,
                                ..default()
                            },
                            HandArea,
                        ))
                        .with_children(|hand_area| {
                            // 手牌卡片容器（稍后动态添加）
                            hand_area.spawn((
                                Text::new("手牌: 0/10"),
                                TextFont {
                                    font: chinese_font.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.6, 0.6, 0.6)),
                                Node {
                                    margin: UiRect::top(Val::Px(5.0)),
                                    ..default()
                                },
                                HandCountText,
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
    sprite_query: Query<Entity, With<SpriteMarker>>,
    draw_pile_query: Query<Entity, With<DrawPile>>,
    discard_pile_query: Query<Entity, With<DiscardPile>>,
    hand_query: Query<Entity, With<Hand>>,
    hand_area_query: Query<Entity, With<HandArea>>,
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
    // 清理精灵实体
    for entity in sprite_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    // 清理牌组实体
    for entity in draw_pile_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in discard_pile_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in hand_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    // 清理手牌区域标记实体
    for entity in hand_area_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    // 移除战斗状态资源
    commands.remove_resource::<CombatState>();
    commands.remove_resource::<DeckConfig>();
}

/// 处理战斗界面按钮点击
fn handle_combat_button_clicks(
    mut next_state: ResMut<NextState<GameState>>,
    mut combat_state: ResMut<CombatState>,
    mut player_query: Query<&mut Player>,
    _enemy_query: Query<&mut Enemy>,
    mut attack_events: EventWriter<EnemyAttackEvent>,
    mut button_queries: ParamSet<(
        Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<EndTurnButton>)>,
        Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<ReturnToMapButton>)>,
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
                // 检查是否破甲（护甲被完全击破）
                let block_broken = player.block > 0 && 10 >= player.block;

                player.take_damage(10);
                info!("玩家受到10点伤害，剩余HP: {}", player.hp);

                // 发送攻击事件，触发动画
                attack_events.send(EnemyAttackEvent::new(10, block_broken));
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
                info!("回合开始：护甲清零");
            }
            // 重置抽牌标志，允许本回合抽牌
            combat_state.cards_drawn_this_turn = false;
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
        Query<&mut Text, With<PlayerBlockText>>,
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
                let old_text = hp_text.0.clone();
                hp_text.0 = format!("HP: {}/{}", player.hp, player.max_hp);
                if old_text != hp_text.0 {
                    info!("玩家HP更新: {} -> {}", old_text, hp_text.0);
                }
            } else {
                error!("严重错误: PlayerHpText 查询失败！UI可能没有正确创建");
            }

            // 更新玩家能量
            if let Ok(mut energy_text) = text_queries.p3().get_single_mut() {
                energy_text.0 = format!("能量: {}/{}", player.energy, player.max_energy);
            }

            // 更新玩家护甲
            if let Ok(mut block_text) = text_queries.p4().get_single_mut() {
                block_text.0 = format!("护甲: {}", player.block);
            }

            // 更新回合数
            if let Ok(mut turn_text) = text_queries.p5().get_single_mut() {
                turn_text.0 = format!("第 {} 回合", player.turn);
            }
        }
    }
}

// ============================================================================
// 抽牌系统
// ============================================================================

/// 战斗开始时抽牌
fn draw_cards_on_combat_start(
    mut draw_pile_query: Query<&mut DrawPile>,
    mut hand_query: Query<&mut Hand>,
) {
    info!("draw_cards_on_combat_start 被调用");
    match (draw_pile_query.get_single_mut(), hand_query.get_single_mut()) {
        (Ok(mut draw_pile), Ok(mut hand)) => {
            info!("抽牌堆卡牌数: {}, 手牌当前数量: {}", draw_pile.count, hand.cards.len());
            // 初始抽5张牌
            let cards_to_draw = 5;
            for _ in 0..cards_to_draw {
                if let Some(card) = draw_pile.draw_card() {
                    hand.add_card(card);
                }
            }
            info!("战斗开始：抽了 {} 张牌，手牌现在有 {} 张", cards_to_draw, hand.cards.len());
        }
        (Err(e), _) => {
            info!("DrawPile 查询失败: {:?}", e);
        }
        (_, Err(e)) => {
            info!("Hand 查询失败: {:?}", e);
        }
    }
}

/// 回合开始时抽牌
fn draw_cards_on_turn_start(
    mut draw_pile_query: Query<&mut DrawPile>,
    mut hand_query: Query<&mut Hand>,
    mut discard_pile_query: Query<&mut DiscardPile>,
    player_query: Query<&Player>,
    mut combat_state: ResMut<CombatState>,
) {
    // 只在玩家回合且回合数大于1时抽牌（避免战斗开始时重复抽牌）
    let player_turn = if let Ok(player) = player_query.get_single() {
        player.turn
    } else {
        return;
    };

    if player_turn <= 1 {
        return;
    }

    // 检查是否已经在这个回合抽过牌
    if combat_state.cards_drawn_this_turn {
        return;
    }

    if let Ok(mut draw_pile) = draw_pile_query.get_single_mut() {
        if let Ok(mut hand) = hand_query.get_single_mut() {
            let cards_to_draw = 5; // 每回合抽5张牌

            // 如果抽牌堆为空，将弃牌堆洗入抽牌堆
            if draw_pile.count == 0 {
                if let Ok(mut discard_pile) = discard_pile_query.get_single_mut() {
                    let cards = discard_pile.clear();
                    if !cards.is_empty() {
                        draw_pile.shuffle_from_discard(cards);
                        info!("抽牌堆为空，将弃牌堆洗入抽牌堆，共 {} 张牌", draw_pile.count);
                    }
                }
            }

            // 抽牌
            let mut drawn = 0;
            for _ in 0..cards_to_draw {
                if let Some(card) = draw_pile.draw_card() {
                    hand.add_card(card);
                    drawn += 1;
                }
            }
            if drawn > 0 {
                info!("回合开始：抽了 {} 张牌", drawn);
                combat_state.cards_drawn_this_turn = true;
            }
        }
    }
}

/// 更新手牌区UI
fn update_hand_ui(
    hand_query: Query<&Hand>,
    hand_changed_query: Query<&Hand, Changed<Hand>>,
    draw_pile_query: Query<&DrawPile>,
    discard_pile_query: Query<&DiscardPile>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<DrawPileText>>,
        Query<&mut Text, With<DiscardPileText>>,
        Query<&mut Text, With<HandCountText>>,
    )>,
    mut hand_area_query: Query<(Entity, &Children), With<HandArea>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // 更新抽牌堆/弃牌堆文本（每帧更新，因为这些数字会变化）
    if let Ok(draw_pile) = draw_pile_query.get_single() {
        if let Ok(mut text) = text_queries.p0().get_single_mut() {
            text.0 = format!("抽牌堆: {}", draw_pile.count);
        }
    }

    if let Ok(discard_pile) = discard_pile_query.get_single() {
        if let Ok(mut text) = text_queries.p1().get_single_mut() {
            text.0 = format!("弃牌堆: {}", discard_pile.count);
        }
    }

    // 每帧更新手牌计数文本
    if let Ok(hand) = hand_query.get_single() {
        match text_queries.p2().get_single_mut() {
            Ok(mut text) => {
                let new_text = format!("手牌: {}/{}", hand.cards.len(), hand.max_size);
                if text.0 != new_text {
                    info!("更新手牌计数文本: {}", new_text);
                    text.0 = new_text;
                }
            }
            Err(e) => {
                // HandCountText 查询失败（可能还没有创建）
                trace!("HandCountText 查询失败: {:?}", e);
            }
        }
    }

    // 只在手牌变化时更新卡牌UI
    if let Ok(hand) = hand_changed_query.get_single() {
        info!("更新手牌UI，手牌数量: {}", hand.cards.len());
        if let Ok((hand_area_entity, children)) = hand_area_query.get_single_mut() {
            let chinese_font: Handle<Font> = asset_server.load("fonts/Arial Unicode.ttf");

            // 清空现有手牌显示（保留第一个子元素，即"手牌: X/Y"文本）
            for (i, child) in children.iter().enumerate() {
                if i > 0 {
                    commands.entity(*child).despawn_recursive();
                }
            }

            // 为每张手牌创建UI卡片
            for (i, card) in hand.cards.iter().enumerate() {
                info!("生成卡牌UI: {} (索引: {})", card.name, i);
                let card_color = card.get_color();
                let cost_text = if card.cost > 0 {
                    format!("{}", card.cost)
                } else {
                    "0".to_string()
                };

                commands.entity(hand_area_entity).with_children(|parent| {
                    // 卡牌容器
                    parent
                        .spawn((
                            Node {
                                width: Val::Px(80.0),
                                height: Val::Px(110.0),
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                padding: UiRect::all(Val::Px(5.0)),
                                margin: UiRect::horizontal(Val::Px(3.0)),
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(card_color),
                            BorderColor(Color::BLACK),
                            HandCard { card_id: card.id },
                            Button,
                        ))
                        .with_children(|card_ui| {
                            // 能量消耗
                            card_ui.spawn((
                                Text::new(cost_text),
                                TextFont {
                                    font: chinese_font.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                Node {
                                    margin: UiRect::top(Val::Px(2.0)),
                                    ..default()
                                },
                            ));

                            // 卡牌名称
                            card_ui.spawn((
                                Text::new(card.name.clone()),
                                TextFont {
                                    font: chinese_font.clone(),
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                Node {
                                    margin: UiRect::top(Val::Px(5.0)),
                                    ..default()
                                },
                            ));

                            // 卡牌描述
                            card_ui.spawn((
                                Text::new(card.description.clone()),
                                TextFont {
                                    font: chinese_font.clone(),
                                    font_size: 10.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                Node {
                                    margin: UiRect::bottom(Val::Px(2.0)),
                                    ..default()
                                },
                            ));
                        });
                });
            }
        }
    }
}

// ============================================================================
// 出牌系统
// ============================================================================

/// 处理卡牌点击事件
fn handle_card_play(
    mut commands: Commands,
    card_query: Query<(&Interaction, &HandCard), (Changed<Interaction>, With<HandCard>)>,
    mut player_query: Query<&mut Player>,
    mut hand_query: Query<&mut Hand>,
    mut discard_pile_query: Query<&mut DiscardPile>,
    mut enemy_query: Query<&mut Enemy>,
    mut draw_pile_query: Query<&mut DrawPile>,
) {
    for (interaction, hand_card) in card_query.iter() {
        if matches!(interaction, Interaction::Pressed) {
            // 获取玩家信息
            let (_can_play, player_energy) = if let Ok(player) = player_query.get_single() {
                (player.energy > 0, player.energy)
            } else {
                (false, 0)
            };

            // 获取手牌信息
            let (card_opt, hand_entity) = if let Ok(hand) = hand_query.get_single() {
                // 找到对应的卡牌
                let card_index = hand.cards.iter().position(|c| c.id == hand_card.card_id);
                (card_index.map(|i| hand.cards[i].clone()), ())
            } else {
                (None, ())
            };

            if let (Some(card), _) = (card_opt, hand_entity) {
                // 检查能量是否足够
                if player_energy >= card.cost {
                    info!("打出卡牌: {} (消耗: {})", card.name, card.cost);

                    // 扣除能量
                    if let Ok(mut player) = player_query.get_single_mut() {
                        player.energy -= card.cost;
                    }

                    // 触发卡牌效果
                    apply_card_effect(
                        &card.effect,
                        &mut commands,
                        &mut player_query,
                        &mut enemy_query,
                        &mut hand_query,
                        &mut draw_pile_query,
                        &mut discard_pile_query,
                    );

                    // 从手牌移除卡牌
                    if let Ok(mut hand) = hand_query.get_single_mut() {
                        if let Some(index) = hand.cards.iter().position(|c| c.id == card.id) {
                            let played_card = hand.remove_card(index).unwrap();
                            // 添加到弃牌堆
                            if let Ok(mut discard_pile) = discard_pile_query.get_single_mut() {
                                discard_pile.add_card(played_card);
                            }
                        }
                    }
                } else {
                    info!("能量不足！需要: {}, 当前: {}", card.cost, player_energy);
                }
            }
        }
    }
}

/// 应用卡牌效果
fn apply_card_effect(
    effect: &CardEffect,
    _commands: &mut Commands,
    player_query: &mut Query<&mut Player>,
    enemy_query: &mut Query<&mut Enemy>,
    hand_query: &mut Query<&mut Hand>,
    draw_pile_query: &mut Query<&mut DrawPile>,
    discard_pile_query: &mut Query<&mut DiscardPile>,
) {
    match effect {
        CardEffect::DealDamage { amount } => {
            if let Ok(mut enemy) = enemy_query.get_single_mut() {
                enemy.take_damage(*amount);
                info!("卡牌效果：对敌人造成 {} 点伤害，敌人剩余HP: {}", amount, enemy.hp);
            }
        }
        CardEffect::GainBlock { amount } => {
            if let Ok(mut player) = player_query.get_single_mut() {
                let old_block = player.block;
                player.gain_block(*amount);
                info!("卡牌效果：获得 {} 点护甲，{} -> {}", amount, old_block, player.block);
            }
        }
        CardEffect::Heal { amount } => {
            if let Ok(mut player) = player_query.get_single_mut() {
                let old_hp = player.hp;
                player.hp = (player.hp + amount).min(player.max_hp);
                info!("卡牌效果：回复 {} 点生命，{} -> {}", amount, old_hp, player.hp);
            }
        }
        CardEffect::DrawCards { amount } => {
            // 从抽牌堆抽牌到手牌
            let mut drawn = 0;
            let cards_to_draw = *amount;

            // 如果抽牌堆为空，先将弃牌堆洗入抽牌堆
            if let Ok(mut draw_pile) = draw_pile_query.get_single_mut() {
                if draw_pile.count == 0 {
                    if let Ok(mut discard_pile) = discard_pile_query.get_single_mut() {
                        let cards = discard_pile.clear();
                        if !cards.is_empty() {
                            draw_pile.shuffle_from_discard(cards);
                            info!("卡牌效果：抽牌堆为空，将弃牌堆洗入抽牌堆，共 {} 张牌", draw_pile.count);
                        }
                    }
                }

                // 抽牌
                for _ in 0..cards_to_draw {
                    if let Some(card) = draw_pile.draw_card() {
                        if let Ok(mut hand) = hand_query.get_single_mut() {
                            if hand.add_card(card) {
                                drawn += 1;
                            }
                        }
                    } else {
                        break; // 抽牌堆空了，停止抽牌
                    }
                }
            }

            if drawn > 0 {
                info!("卡牌效果：抽了 {} 张牌", drawn);
            }
        }
        CardEffect::GainEnergy { amount } => {
            if let Ok(mut player) = player_query.get_single_mut() {
                let old_energy = player.energy;
                player.energy = (player.energy + amount).min(player.max_energy);
                info!("卡牌效果：获得 {} 点能量，{} -> {}", amount, old_energy, player.energy);
            }
        }
        CardEffect::AttackAndDraw { damage, cards } => {
            if let Ok(mut enemy) = enemy_query.get_single_mut() {
                enemy.take_damage(*damage);
                info!("卡牌效果：造成 {} 点伤害，敌人剩余HP: {}", damage, enemy.hp);
            }

            // 抽牌效果
            let mut drawn = 0;
            if let Ok(mut draw_pile) = draw_pile_query.get_single_mut() {
                // 如果抽牌堆为空，先将弃牌堆洗入抽牌堆
                if draw_pile.count == 0 {
                    if let Ok(mut discard_pile) = discard_pile_query.get_single_mut() {
                        let cards = discard_pile.clear();
                        if !cards.is_empty() {
                            draw_pile.shuffle_from_discard(cards);
                            info!("卡牌效果：抽牌堆为空，将弃牌堆洗入抽牌堆，共 {} 张牌", draw_pile.count);
                        }
                    }
                }

                for _ in 0..*cards {
                    if let Some(card) = draw_pile.draw_card() {
                        if let Ok(mut hand) = hand_query.get_single_mut() {
                            if hand.add_card(card) {
                                drawn += 1;
                            }
                        }
                    } else {
                        break;
                    }
                }
            }

            if drawn > 0 {
                info!("卡牌效果：抽了 {} 张牌", drawn);
            }
        }
        CardEffect::MultiAttack { damage, times } => {
            if let Ok(mut enemy) = enemy_query.get_single_mut() {
                let total_damage = damage * times;
                enemy.take_damage(total_damage);
                info!("卡牌效果：{} 次攻击，每次 {} 点伤害，共 {} 点，敌人剩余HP: {}", times, damage, total_damage, enemy.hp);
            }
        }
    }
}

/// 检查战斗是否结束
fn check_combat_end(
    player_query: Query<&Player>,
    enemy_query: Query<&Enemy>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // 检查敌人是否死亡
    if let Ok(enemy) = enemy_query.get_single() {
        if enemy.is_dead() {
            info!("敌人被击败！战斗胜利！");
            // TODO: 显示胜利界面和奖励
            // 暂时直接返回地图
            next_state.set(GameState::Map);
            return;
        }
    }

    // 检查玩家是否死亡
    if let Ok(player) = player_query.get_single() {
        if player.hp <= 0 {
            info!("玩家败北！");
            // TODO: 显示失败界面
            // 暂时直接返回地图
            next_state.set(GameState::Map);
            return;
        }
    }
}
