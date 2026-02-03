//! 地图系统
//! 
//! 处理地图界面的生成、交互和状态转换。

use bevy::prelude::*;
use crate::states::GameState;
use crate::components::{
    Player, Cultivation, PlayerDeck,
    PlaySfxEvent, SfxType, CombatUiRoot,
    relic::RelicCollection,
    map::{MapProgress, MapNode, NodeType, MapNodeButton, RippleEffect, MapNodeContainer, MapUiRoot, BreakthroughButtonMarker, BreathingAnimation, OriginalSize, HoverEffect, EntranceAnimation, PulseAnimation, ConnectorDot}
};
use crate::resources::save::GameStateSave;
use crate::plugins::init_player;

/// 地图插件
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        // 初始化地图进度资源（确保重新开始时不闪退）
        app.init_resource::<MapProgress>();
        
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
        app.add_systems(Update, (
            handle_map_button_clicks,
            handle_map_scrolling,
            animate_connector_dots, // 激活灵力流动
        ).run_if(in_state(GameState::Map)));
    }
}

/// 清理地图UI
fn cleanup_map_ui(mut commands: Commands, query: Query<Entity, With<MapUiRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

/// 设置地图UI
pub fn setup_map_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut map_progress: Option<ResMut<MapProgress>>, // 保持 ResMut
    player_query: Query<(&Player, &Cultivation)>,
    player_deck: Res<PlayerDeck>,
    relic_collection: Res<RelicCollection>,
    existing_ui: Query<Entity, With<CombatUiRoot>>, 
) {
    let chinese_font: Handle<Font> = asset_server.load("fonts/Arial Unicode.ttf");

    // --- [统一自动存档点] 进入地图即保存，安全稳健 ---
    if let Some(progress) = &map_progress {
        if let Ok((player, cultivation)) = player_query.get_single() {
            let save = GameStateSave {
                player: player.clone(),
                cultivation: cultivation.clone(),
                deck: player_deck.cards.clone(),
                relics: relic_collection.relic.clone(),
                map_nodes: progress.nodes.clone(),
                current_map_node_id: progress.current_node_id,
                current_map_layer: progress.current_layer,
            };
            
            // [核心修复] 使用线程池进行真正的异步磁盘 IO
            // 彻底解决阻塞主线程导致的卡死问题
            use bevy::tasks::AsyncComputeTaskPool;
            let thread_pool = AsyncComputeTaskPool::get();
            thread_pool.spawn(async move {
                if let Err(e) = save.save_to_disk() {
                    error!("【存档失败】无法持久化识海进度: {}", e);
                } else {
                    // 注意：在异步线程中直接使用 info! 是安全的，日志宏通常是线程安全的
                    // 但不要在里面操作 World
                }
            }).detach();
            
            info!("【自动存档】正在同步进度至识海...");
        }
    }

    // 1. 健壮性处理地图进度 (如果缺失则新建)
    if map_progress.is_none() {
        warn!("【地图】识海中未发现地图进度资源，正在推演全新命途...");
        commands.insert_resource(MapProgress::default());
        // 标记本次 setup 提前结束，依赖下一帧的资源就绪
        return; 
    }
    let mut progress = map_progress.unwrap();

    // [关键修复] 仅清理战斗相关的 UI，不要清理自己的标记
    for entity in existing_ui.iter() {
        commands.entity(entity).despawn_recursive();
    }


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

            // 关键修复：如果 Player 实体还没出生，但我们已有进度，

            // 赋予一个足够大的临时视野，确保读档进来的那一帧不是黑屏。

            if current_layer > 0 { 2 } else { 1 }

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

            // 地图节点容器 - 添加滚动支持与自动聚焦
            parent
                .spawn((
                    Node {
                        width: Val::Percent(90.0),
                        height: Val::Percent(70.0),
                        align_items: AlignItems::Center,  // 水平居中
                        justify_content: JustifyContent::FlexStart,  // 从顶部开始
                        flex_direction: FlexDirection::Column,
                        overflow: Overflow::scroll(),  // 启用滚动
                        // 移除显式的 row_gap，依靠子项自身高度(60px)和连线容器高度(60px)形成间距
                        ..default()
                    },
                    MapNodeContainer,
                    // 自动聚焦：根据当前层级计算初始滚动偏移
                    // 每一层（含连接层）估算高度为 150px
                    ScrollPosition {
                        offset_y: (current_layer as f32 * 150.0).max(0.0),
                        ..default()
                    },
                ))
                .with_children(|map_parent| {
                    let max_layer = nodes.iter().map(|n| n.position.0).max().unwrap_or(0);

                    // [性能优化] 仅渲染当前层附近的节点，防止后期节点过多卡死
                    // 视野窗口：向后看2层，向前看 vision_range 层
                    let min_visible_layer = (current_layer as i32 - 2).max(0);
                    let max_visible_layer = current_layer as i32 + vision_range as i32;

                    for layer in 0..=max_layer {
                        // 视野增强逻辑：
                        // 1. 在可视窗口内的层级
                        // 2. Boss层总是可见 (提供目标感)
                        let is_in_window = layer as i32 >= min_visible_layer && layer as i32 <= max_visible_layer;
                        let is_boss_layer = layer == max_layer as i32;

                        if !is_in_window && !is_boss_layer {
                            // [关键修复] 对于被剔除的层级，渲染一个占位容器，保持滚动条高度和布局位置
                            // 否则中间层级消失会导致 Boss 层直接紧挨着当前层，造成视觉错乱
                            map_parent.spawn(Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(140.0), // 估算高度：节点行(80) + 连线(60)
                                ..default()
                            });
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
    map_progress_opt: Option<ResMut<MapProgress>>,
    player_query: Query<(&Player, &Cultivation)>,
    deck: Res<PlayerDeck>,
    relics: Res<RelicCollection>,
    button_queries: Query<(&Interaction, &MapNodeButton, &Node)>, // 移除 Changed<Interaction>
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    let Some(mut map_progress) = map_progress_opt else { return; };
    for (interaction, node_btn, node) in button_queries.iter() {
        if matches!(interaction, Interaction::Pressed) {
            // 播放音效
            sfx_events.send(PlaySfxEvent::new(SfxType::UiClick));

            // 创建波纹特效
            sfx_events.send(PlaySfxEvent::new(SfxType::UiClick));

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
            
            // --- [优化] 移除此处同步存档，防止跳转时的 IO 阻塞 ---

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

/// 处理地图滚轮滚动
fn handle_map_scrolling(
    mut mouse_wheel_events: EventReader<bevy::input::mouse::MouseWheel>,
    mut query: Query<&mut ScrollPosition, With<MapNodeContainer>>,
) {
    for event in mouse_wheel_events.read() {
        for mut scroll in query.iter_mut() {
            // offset_y 越大，内容向上移动（即看到下方内容）
            // 我们需要反向映射：向下滚轮（y为负）应增加 offset_y
            scroll.offset_y -= event.y * 20.0; 
            
            if scroll.offset_y < 0.0 {
                scroll.offset_y = 0.0;
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
    existing_button: Query<Entity, With<BreakthroughButtonMarker>>,
) {
    // 关键修复：如果按钮已经存在，直接返回，防止死循环
    if !existing_button.is_empty() {
        return;
    }

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
                    BorderColor(Color::srgb(1.0, 0.8, 0.2)),
                    BackgroundColor(Color::srgba(0.1, 0.05, 0.2, 0.95)),
                    BreakthroughButtonMarker,
                    // [彻底修复] 配合动画系统的 Without 过滤，现在可以安全使用 MapUiRoot 保证自动清理
                    MapUiRoot,
                    ZIndex(999), 
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
                .observe(|_trigger: Trigger<Pointer<Over>>, mut query: Query<&mut BackgroundColor, With<BreakthroughButtonMarker>>| {
                    for mut bg in query.iter_mut() {
                        bg.0 = Color::srgba(0.2, 0.1, 0.4, 1.0); // 悬停加亮
                    }
                })
                .observe(|_trigger: Trigger<Pointer<Out>>, mut query: Query<&mut BackgroundColor, With<BreakthroughButtonMarker>>| {
                    for mut bg in query.iter_mut() {
                        bg.0 = Color::srgba(0.1, 0.05, 0.2, 0.95); // 恢复原色
                    }
                })
                .observe(|_entity: Trigger<Pointer<Click>>, mut next_state: ResMut<NextState<GameState>>| {
                    info!("【点击测试】点击了引动雷劫按钮！尝试进入 Tribulation 状态");
                    next_state.set(GameState::Tribulation);
                });
                        }
                    }
                }
                
/// 在连接区域生成路径导向点（大作级视觉增强版）
fn spawn_path_indicator(
    parent: &mut ChildBuilder,
    from_node: &MapNode,
    to_node: &MapNode,
    from_count: f32,
    to_count: f32,
    _progress: &MapProgress,
) {
    let is_path_unlocked = from_node.completed;
    
    // --- 极致对位算法 ---
    let get_x_percent = |idx: i32, total: f32| -> f32 {
        ((idx as f32 + 1.0) / (total + 1.0)) * 100.0
    };

    let start_x = get_x_percent(from_node.position.1, from_count);
    let end_x = get_x_percent(to_node.position.1, to_count);

    // --- 贝塞尔曲线参数 ---
    // 如果是斜向连线，通过中点偏移产生弧度
    let has_horizontal_shift = from_node.position.1 != to_node.position.1;
    let mid_x = if has_horizontal_shift {
        // 弧度提升：从 2.0% 提升至 5.0%，让曲线更优雅
        start_x + (end_x - start_x) * 0.5 + if end_x > start_x { 5.0 } else { -5.0 }
    } else {
        start_x
    };

    // 极致密度：15 个点形成连贯路径
    let point_count = 15;
    for i in 0..=point_count {
        let t = i as f32 / point_count as f32; 
        
        // 二次贝塞尔曲线公式: (1-t)^2 * P0 + 2t(1-t) * P1 + t^2 * P2
        let current_x = (1.0 - t).powi(2) * start_x + 2.0 * t * (1.0 - t) * mid_x + t.powi(2) * end_x;
        
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(current_x),
                margin: UiRect::left(Val::Px(-3.5)), // 精确居中 (点宽 7px)
                top: Val::Percent(t * 100.0),
                width: Val::Px(7.0),
                height: Val::Px(7.0),
                ..default()
            },
            BackgroundColor(if is_path_unlocked {
                Color::srgba(0.85, 0.95, 1.0, 0.95) // 亮蓝色灵力
            } else {
                Color::srgba(0.4, 0.4, 0.5, 0.15) // 暗淡灰色
            }),
            // --- 大作级视觉细节：外发光 ---
            BoxShadow {
                color: if is_path_unlocked {
                    Color::srgba(0.4, 0.7, 1.0, 0.5) // 幽蓝光晕
                } else {
                    Color::srgba(0.0, 0.0, 0.0, 0.0)
                },
                blur_radius: Val::Px(8.0),
                spread_radius: Val::Px(2.0),
                ..default()
            },
            BorderRadius::all(Val::Px(3.5)),
            ConnectorDot {
                offset: (from_node.id as f32 * 1.3) + (i as f32 * 0.25), // 灵动的相位偏移
            },
        ));
    }
}

/// 动画系统：让路径点产生灵动的流动感
pub fn animate_connector_dots(
    time: Res<Time>,
    mut query: Query<(&mut BackgroundColor, &mut Node, &ConnectorDot)>,
) {
    let t = time.elapsed_secs();
    for (mut color, mut node, dot) in query.iter_mut() {
        // 基于时间、位置偏移和 Sine 波计算呼吸相位
        let alpha_pulse = ((t * 2.5 + dot.offset).sin() * 0.4 + 0.6).clamp(0.0, 1.0);
        let scale_pulse = (t * 3.0 + dot.offset).cos() * 0.1 + 1.0;
        
        // 更新颜色透明度
        let current_srgba = color.0.to_srgba();
        color.0 = Color::srgba(current_srgba.red, current_srgba.green, current_srgba.blue, current_srgba.alpha * alpha_pulse);
        
        // 微调点的大小产生呼吸感
        let base_size = 7.0;
        node.width = Val::Px(base_size * scale_pulse);
        node.height = Val::Px(base_size * scale_pulse);
    }
}
                
                
                