//! 地图系统
//! 
//! 处理地图界面的生成、交互和状态转换。

use bevy::prelude::*;
use crate::states::GameState;
use crate::components::{
    Player, Cultivation, PlayerDeck, RelicCollection,
};
use crate::components::map::{
    MapProgress, MapNode, NodeType, MapUiRoot, MapNodeContainer, MapNodeButton,
    BreakthroughButtonMarker, OriginalSize, BreathingAnimation, RippleEffect,
    EntranceAnimation, ConnectorDot, PulseAnimation, HoverEffect,
};
use crate::resources::save::GameStateSave;
use crate::plugins::init_player;

/// 地图插件
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        // 在进入Map状态时设置地图UI
        app.add_systems(OnEnter(GameState::Map), (
            init_player,
            setup_map_ui, 
            setup_breakthrough_button, 
            setup_cultivation_status_ui
        ).chain());
        // 在退出Map状态时清理地图UI
        app.add_systems(OnExit(GameState::Map), cleanup_map_ui);
        // 处理地图界面按钮点击
        app.add_systems(Update, handle_map_button_clicks.run_if(in_state(GameState::Map)));
    }
}

/// 清理地图UI
fn cleanup_map_ui(mut commands: Commands, query: Query<Entity, With<MapUiRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

/// 设置地图UI
fn setup_map_ui(
    mut commands: Commands, 
    asset_server: Res<AssetServer>, 
    map_progress: Option<Res<MapProgress>>,
    player_query: Query<(&Player, &Cultivation)>,
) {
    let chinese_font: Handle<Font> = asset_server.load("fonts/Arial Unicode.ttf");

    // 1. 健壮性处理地图进度
    let mut progress = if let Some(p) = map_progress {
        p.clone()
    } else {
        info!("【地图系统】未找到地图进度，创建新机缘地图");
        let new_progress = MapProgress::default();
        commands.insert_resource(new_progress.clone());
        new_progress
    };

    // 关键修复：如果进度中节点为空（可能由于坏档或旧版本存档），强制重新生成
    if progress.nodes.is_empty() {
        warn!("【地图系统】检测到空地图节点，正在强制重新生成...");
        use crate::components::map::{MapConfig, generate_map_nodes};
        progress.nodes = generate_map_nodes(&MapConfig::default(), 0);
        progress.refresh_unlocks();
        // 立即更新资源，防止其它系统也读到空数据
        commands.insert_resource(progress.clone());
    }

    let nodes = progress.nodes.clone();
    
    // 自动推断当前活跃层级：优先取当前所在节点，若无则取已完成节点的最高层
    let current_layer = if let Some(id) = progress.current_node_id {
        nodes.iter().find(|n| n.id == id).map(|n| n.position.0).unwrap_or(0)
    } else {
        nodes.iter()
            .filter(|n| n.completed)
            .map(|n| n.position.0)
            .max()
            .unwrap_or(0)
    };

    // 2. 健壮性处理玩家数据
    let player_info = player_query.get_single().ok();

    
    use crate::components::cultivation::Realm;
    let vision_range = if let Some((_, cultivation)) = player_info {
        match cultivation.realm {
            Realm::QiRefining => 1,
            Realm::FoundationEstablishment => 2,
            Realm::GoldenCore => 3,
            _ => 99,
        }
    } else {
        1 // 默认为修士入门视野
    };

    let player_gold = if let Some((p, _)) = player_info { p.gold } else { 0 };

    // 3. 始终创建 UI 根节点
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.05, 0.05, 0.05)),
            MapUiRoot,
            ZIndex(100),
        ))
        .with_children(|parent| {
            // 地图标题与境界显示
            parent.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            }).with_children(|header| {
                header.spawn((
                    Text::new("寻 仙 觅 缘"),
                    TextFont { font: chinese_font.clone(), font_size: 42.0, ..default() },
                    TextColor(Color::srgb(0.8, 1.0, 0.8)),
                ));
                
                let realm_text = if let Some((_, cultivation)) = player_info {
                    match cultivation.realm {
                        Realm::QiRefining => "炼气期 - 神识范围: 1层",
                        Realm::FoundationEstablishment => "筑基期 - 神识范围: 2层",
                        Realm::GoldenCore => "金丹期 - 神识范围: 3层",
                        Realm::NascentSoul => "元婴期 - 神识纵览全程",
                    }
                } else {
                    "神识扫描中..."
                };
                
                header.spawn((
                    Text::new(realm_text),
                    TextFont { font: chinese_font.clone(), font_size: 16.0, ..default() },
                    TextColor(Color::srgb(0.5, 0.7, 0.5)),
                ));
            });

            // 地图节点容器 - 添加滚动支持
            parent
                .spawn((
                    Node {
                        width: Val::Percent(90.0),
                        height: Val::Percent(70.0),
                        align_items: AlignItems::Center,  // 水平居中
                        justify_content: JustifyContent::FlexStart,  // 从顶部开始
                        flex_direction: FlexDirection::Column,
                        overflow: Overflow::scroll(),  // 启用滚动
                        row_gap: Val::Px(10.0),  // 使用 row_gap 控制层间距
                        ..default()
                    },
                    MapNodeContainer,
                ))
                .with_children(|map_parent| {
                    let max_layer = nodes.iter().map(|n| n.position.0).max().unwrap_or(0);

                    for layer in 0..=max_layer {
                        // 视野增强逻辑：
                        // 1. 在神识视野范围内的层级可见
                        // 2. Boss层总是可见
                        // 3. 具有已解锁节点或已完成节点的层级必须可见（确保玩家知道下一步去哪，或者看到来时的路）
                        let has_relevant_nodes = nodes.iter()
                            .filter(|n| n.position.0 == layer)
                            .any(|n| n.unlocked || n.completed);

                        let is_visible = (layer as i32) <= (current_layer as i32 + vision_range as i32) 
                                        || layer == max_layer as i32
                                        || has_relevant_nodes;
                        
                        if !is_visible {
                            continue;
                        }

                        // 创建层容器
                        map_parent
                            .spawn(Node {
                                width: Val::Percent(100.0),
                                min_height: Val::Px(80.0),  // 设置最小高度，确保每层占空间
                                height: Val::Auto,
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                ..default()
                            })
                            .with_children(|layer_container| {
                                // 节点行
                                layer_container
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
                                        // 显示该层的所有节点
                                        for node in &nodes {
                                            if node.position.0 == layer {
                                                spawn_map_node(layer_parent, node, &chinese_font, &progress);
                                            }
                                        }
                                    });

                                // --- 拓扑连线生成 ---
                                if layer < max_layer {
                                    // 统计当前层和下一层的节点数，用于精确对位
                                    let from_count = nodes.iter().filter(|n| n.position.0 == layer).count() as f32;
                                    let to_count = nodes.iter().filter(|n| n.position.0 == layer + 1).count() as f32;

                                    layer_container.spawn((
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Px(60.0),
                                            ..default()
                                        },
                                    )).with_children(|connector| {
                                        for node in &nodes {
                                            if node.position.0 == layer {
                                                for &next_id in &node.next_nodes {
                                                    if let Some(next_node) = nodes.iter().find(|n| n.id == next_id) {
                                                        spawn_path_indicator(connector, node, next_node, from_count, to_count, &progress);
                                                    }
                                                }
                                            }
                                        }
                                    });
                                }
                            });
                    }
                });

            // 底部状态栏（显示灵石等）
            parent.spawn(Node {
                position_type: PositionType::Absolute,
                left: Val::Px(20.0),
                bottom: Val::Px(20.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(30.0),
                ..default()
            }).with_children(|footer| {
                // 1. 查看牌组按钮
                footer.spawn((
                    Button,
                    Node {
                        width: Val::Px(120.0),
                        height: Val::Px(45.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.4)),
                    BorderRadius::all(Val::Px(8.0)),
                    crate::components::cards::ViewDeckButton,
                )).with_children(|btn| {
                    btn.spawn((
                        Text::new("藏经阁"),
                        TextFont { font: chinese_font.clone(), font_size: 22.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });

                // 2. 灵石信息
                footer.spawn((
                    Text::new(format!("灵石: {}", player_gold)),
                    TextFont { font: chinese_font.clone(), font_size: 20.0, ..default() },
                    TextColor(Color::srgb(1.0, 0.8, 0.2)),
                ));
            });
        });
}

/// 地图节点生成器
fn spawn_map_node(
    parent: &mut ChildBuilder,
    node: &MapNode,
    font: &Handle<Font>,
    map_progress: &MapProgress,
) {
    // 检查是否是当前节点 (且尚未完成)
    let is_current_active = map_progress.current_node_id == Some(node.id) && !node.completed;

    // 根据节点类型选择颜色
    let node_color = match node.node_type {
        NodeType::Normal => Color::srgb(0.3, 0.5, 0.3),  // 绿色
        NodeType::Elite => Color::srgb(0.6, 0.3, 0.1),   // 橙色
        NodeType::Boss => Color::srgb(0.7, 0.1, 0.1),    // 红色
        NodeType::Rest => Color::srgb(0.3, 0.4, 0.6),    // 蓝色
        NodeType::Shop => Color::srgb(0.8, 0.8, 0.2),
        NodeType::Event => Color::srgb(0.2, 0.6, 0.8), 
        NodeType::Treasure => Color::srgb(1.0, 0.8, 0.2), // 金色
        NodeType::Unknown => Color::srgb(0.4, 0.4, 0.4),
    };

    // 根据节点状态计算显示颜色
    let (display_color, border_color) = if node.completed {
        // 已完成：深蓝色足迹
        (Color::srgb(0.1, 0.3, 0.6), Color::BLACK)
    } else if is_current_active {
        // 当前正处于：黄色发光
        (Color::srgb(1.0, 0.9, 0.3), Color::WHITE)
    } else if node.unlocked {
        // 已解锁：高对比度彩色 + 白色描边
        (node_color, Color::WHITE)
    } else {
        // 未解锁：暗色半透明
        let mut c = node_color;
        c.set_alpha(0.2); 
        (c, Color::BLACK)
    };

    let mut entity_cmds = parent.spawn((
        Button,
        Node {
            width: Val::Px(60.0),
            height: Val::Px(60.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: if node.unlocked && !node.completed { UiRect::all(Val::Px(3.0)) } else { UiRect::all(Val::Px(1.0)) },
            ..default()
        },
        BackgroundColor(display_color),
        BorderColor(border_color),
        BorderRadius::all(Val::Px(30.0)),
        MapNodeButton { node_id: node.id },
        // 保存原始尺寸用于动画
        OriginalSize {
            width: Val::Px(60.0),
            height: Val::Px(60.0),
        },
        HoverEffect::default(),
    ));

    // 未完成且已解锁的节点添加呼吸动画
    if node.unlocked && !node.completed {
        entity_cmds.insert(BreathingAnimation::default());
    }

    // 添加入场动画
    entity_cmds.insert(EntranceAnimation::new(0.4));

    // 当前激活节点添加脉冲动画
    if is_current_active {
        entity_cmds.insert(PulseAnimation::default());
    }

    entity_cmds.with_children(|btn| {
        let icon = match node.node_type {
            NodeType::Normal => "战",
            NodeType::Elite => "强",
            NodeType::Boss => "王",
            NodeType::Shop => "坊",
            NodeType::Rest => "府",
            NodeType::Treasure => "缘",
            _ => "？",
        };
        btn.spawn((
            Text::new(icon),
            TextFont { font: font.clone(), font_size: 28.0, ..default() },
            TextColor(if node.unlocked { Color::WHITE } else { Color::srgba(1.0, 1.0, 1.0, 0.1) }),
        ));
    });
}

/// 处理地图界面按钮点击
fn handle_map_button_clicks(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut map_progress: ResMut<MapProgress>,
    player_query: Query<(&Player, &Cultivation)>,
    deck: Res<PlayerDeck>,
    relics: Res<RelicCollection>,
    button_queries: Query<(&Interaction, &MapNodeButton, &Node), Changed<Interaction>>,
) {
    for (interaction, node_btn, node) in button_queries.iter() {
        if matches!(interaction, Interaction::Pressed) {
            // 创建波纹特效
            if let Val::Px(size) = node.width {
                let center = size / 2.0;
                commands.spawn((
                    Node {
                        width: Val::Px(0.0),
                        height: Val::Px(0.0),
                        position_type: PositionType::Absolute,
                        left: Val::Px(center),
                        top: Val::Px(center),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
                    BorderRadius::all(Val::Px(100.0)),
                    RippleEffect::new(size * 1.5, 0.6),
                    ZIndex(-1),
                ));
            }
            let node_id = node_btn.node_id;
            
            // 找到对应的节点
            let node_type = if let Some(node) = map_progress.nodes.iter().find(|n| n.id == node_id) {
                // 只有解锁的节点才能点击
                if !node.unlocked {
                    warn!("【地图】节点 {} 尚未解锁，不可前往", node_id);
                    continue;
                }
                if node.completed {
                    warn!("【地图】节点 {} 已经探索完毕", node_id);
                    continue;
                }
                node.node_type
            } else {
                warn!("【地图】未能找到 ID 为 {} 的节点", node_id);
                continue;
            };

            info!("点击了地图节点: {}, 类型: {:?}", node_id, node_type);
            
            // 更新当前位置
            map_progress.set_current_node(node_id);
            
            // --- 执行自动存档 ---
            if let Ok((player, cultivation)) = player_query.get_single() {
                let save = GameStateSave {
                    player: player.clone(),
                    cultivation: cultivation.clone(),
                    deck: deck.cards.clone(),
                    relics: relics.relic.clone(),
                    map_nodes: map_progress.nodes.clone(),
                    current_map_node_id: map_progress.current_node_id,
                    current_map_layer: map_progress.current_layer,
                };
                let _ = save.save_to_disk();
            }

            // 根据节点类型切换状态
            match node_type {
                NodeType::Normal | NodeType::Elite | NodeType::Boss => {
                    info!("【地图】前往战斗关卡: {}", node_id);
                    next_state.set(GameState::Combat);
                }
                NodeType::Rest => {
                    info!("【地图】前往洞府闭关: {}", node_id);
                    next_state.set(GameState::Rest);
                }
                NodeType::Shop => {
                    info!("【地图】前往仙家坊市: {}", node_id);
                    next_state.set(GameState::Shop);
                }
                NodeType::Event => {
                    info!("【地图】触发随机机缘: {}", node_id);
                    next_state.set(GameState::Event);
                }
                NodeType::Treasure => {
                    info!("【地图】偶遇上古宝箱: {}", node_id);
                    next_state.set(GameState::Reward);
                }
                _ => {
                    warn!("【地图】节点 {} 类型 {:?} 尚未实现逻辑", node_id, node_type);
                }
            }
        }
    }
}

/// 设置修为状态显示（左上角）
fn setup_cultivation_status_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<(&Player, &Cultivation)>,
) {
    let chinese_font: Handle<Font> = asset_server.load("fonts/Arial Unicode.ttf");

    if let Ok((player, cultivation)) = player_query.get_single() {
        let realm_name = match cultivation.realm {
            crate::components::cultivation::Realm::QiRefining => "炼气期",
            crate::components::cultivation::Realm::FoundationEstablishment => "筑基期",
            crate::components::cultivation::Realm::GoldenCore => "金丹期",
            crate::components::cultivation::Realm::NascentSoul => "元婴期",
        };

        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(20.0),
                    left: Val::Px(20.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
                MapUiRoot,
            ))
            .with_children(|parent| {
                // 境界显示
                parent.spawn((
                    Text::new(format!("当前境界: {}", realm_name)),
                    TextFont {
                        font: chinese_font.clone(),
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.4, 1.0, 0.4)),
                ));

                // 感悟进度
                parent.spawn((
                    Text::new(format!("感悟进度: {} / {}", cultivation.insight, cultivation.get_threshold())),
                    TextFont {
                        font: chinese_font.clone(),
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.8, 0.8, 1.0)),
                ));

                // 道行（HP）显示
                parent.spawn((
                    Text::new(format!("当前道行: {} / {}", player.hp, player.max_hp)),
                    TextFont {
                        font: chinese_font.clone(),
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.6, 0.6)),
                ));
            });
    }
}

/// 设置渡劫按钮（仅在可突破时显示）
fn setup_breakthrough_button(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cultivation_query: Query<&Cultivation>,
) {
    if let Ok(cultivation) = cultivation_query.get_single() {
        if cultivation.can_breakthrough() {
            info!("【UI】检测到可突破，创建引动雷劫按钮");
            
            commands
                .spawn((
                    Button,
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(40.0),
                        right: Val::Px(40.0),
                        width: Val::Px(240.0),
                        height: Val::Px(90.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                    ZIndex(100), // 确保在最顶层，防止被地图节点遮挡
                    BorderColor(Color::srgb(1.0, 0.8, 0.2)),
                    BackgroundColor(Color::srgba(0.1, 0.05, 0.2, 0.95)),
                    BreakthroughButtonMarker,
                    MapUiRoot,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("引动雷劫"),
                        TextFont {
                            font: asset_server.load("fonts/Arial Unicode.ttf"),
                            font_size: 36.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.9, 0.5)),
                    ));
                })
                                .observe(|_entity: Trigger<Pointer<Click>>, mut next_state: ResMut<NextState<GameState>>| {
                                    info!("【点击测试】点击了引动雷劫按钮！尝试进入 Tribulation 状态");
                                    next_state.set(GameState::Tribulation);
                                });
                        }
                    }
                }
                
/// 在连接区域生成路径导向点
fn spawn_path_indicator(
    parent: &mut ChildBuilder,
    from_node: &MapNode,
    to_node: &MapNode,
    from_count: f32,
    to_count: f32,
    _progress: &MapProgress,
) {
    let is_path_unlocked = from_node.completed;
    
    // --- 极致对位算法：匹配 SpaceEvenly ---
    // 公式: (index + 1) / (total + 1)
    let get_x_percent = |idx: i32, total: f32| -> f32 {
        ((idx as f32 + 1.0) / (total + 1.0)) * 100.0
    };

    let start_x = get_x_percent(from_node.position.1, from_count);
    let end_x = get_x_percent(to_node.position.1, to_count);

    // 提升密度：使用 12 个点形成紧密的虚线感
    let point_count = 12;
    for i in 1..=point_count {
        let t = i as f32 / (point_count as f32 + 1.0); 
        let current_x = start_x + (end_x - start_x) * t;
        
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(current_x),
                margin: UiRect::left(Val::Px(-3.0)), // 补偿点自身宽度 (6px) 以对准圆心
                top: Val::Percent(t * 100.0),
                width: Val::Px(6.0), // 稍微缩小点的大小，使其更精致
                height: Val::Px(6.0),
                ..default()
            },
            BackgroundColor(if is_path_unlocked {
                Color::srgba(0.8, 0.9, 1.0, 0.9) // 已解锁：高亮蓝白
            } else {
                Color::srgba(0.3, 0.3, 0.4, 0.15) // 未解锁：极淡阴影
            }),
            BorderRadius::all(Val::Px(3.0)),
            ConnectorDot {
                offset: (from_node.id as f32 * 0.7) + (i as f32 * 0.2), // 错落有致的流动感
            },
        ));
    }
}
                
                
                