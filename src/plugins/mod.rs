//! 游戏插件定义

use bevy::prelude::*;
use crate::states::GameState;
use crate::components::{
    MapNode, NodeType, MapProgress, Player, Enemy, EnemyIntent, CombatState, TurnPhase, 
    Hand, DrawPile, DiscardPile, DeckConfig, CardEffect, Card, CardType, CardRarity, 
    CardPool, PlayerDeck, EnemyUiMarker, PlayerUiMarker, EnemyAttackEvent, CharacterType, 
    SpriteMarker, ParticleMarker, EmitterMarker, EffectType, SpawnEffectEvent, 
    ScreenEffectEvent, ScreenEffectMarker, VictoryEvent, EnemyDeathAnimation, 
    EnemySpriteMarker, VictoryDelay, RelicCollection, Relic, RelicId,
    RelicObtainedEvent, RelicTriggeredEvent
};
use crate::systems::sprite::spawn_character_sprite;

/// 核心游戏插件
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GameState>();
        // 应用启动时设置2D相机（用于渲染UI）
        app.add_systems(Startup, setup_camera);
        // 初始化胜利延迟计时器
        app.insert_resource(VictoryDelay::new(2.0)); // 延迟2.0秒让粒子特效播放

        // 玩家实体初始化系统 - 在所有OnEnter系统之前运行
        // 使用world_mut().spawn()确保实体立即可用，避免重复创建
        app.add_systems(OnEnter(GameState::MainMenu), init_player);
        app.add_systems(OnEnter(GameState::Map), init_player);
        app.add_systems(OnEnter(GameState::Combat), init_player);
        app.add_systems(OnEnter(GameState::Shop), init_player);
        app.add_systems(OnEnter(GameState::Rest), init_player);
        app.add_systems(OnEnter(GameState::Reward), init_player);
    }
}

/// 初始化玩家实体（如果不存在）
///
/// 此函数使用 world_mut().spawn() 而不是 commands.spawn()
/// 在 Bevy 0.15 中，world_mut().spawn() 是立即生效的，而 commands.spawn() 是延迟的
/// 这确保玩家实体在当前帧立即可用，避免重复创建
fn init_player(mut commands: Commands) {
    // 使用 Deferred 视图访问 World 进行立即 spawn
    commands.queue(move |world: &mut World| {
        let mut player_query = world.query_filtered::<Entity, With<Player>>();
        let player_entity = player_query.iter(world).next();

        if let Some(entity) = player_entity {
            // 玩家已存在，检查是否缺少 Cultivation 组件
            if world.get::<crate::components::Cultivation>(entity).is_none() {
                info!("init_player: 玩家实体已存在，但缺少修为，正在补全...");
                world.entity_mut(entity).insert(crate::components::Cultivation::new());
            }
        } else {
            // 玩家不存在，创建全新修仙者
            world.spawn((
                Player { gold: 100, ..Default::default() },
                crate::components::Cultivation::new(),
            ));
            info!("init_player: 创建全新修仙者实体（初始境界：炼气）");
        }
    });
}

/// 主菜单UI插件
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        // 注册胜利事件
        app.add_event::<VictoryEvent>();

        // 初始化悬停状态资源
        app.init_resource::<HoveredCard>();
        app.init_resource::<HoveredRelic>();
        app.init_resource::<CurrentRewardCards>();
        app.init_resource::<CurrentRewardRelic>();
        app.init_resource::<MousePosition>();


        // 在进入MainMenu状态时设置主菜单
        app.add_systems(OnEnter(GameState::MainMenu), setup_main_menu);
        // 在退出MainMenu状态时清理主菜单
        app.add_systems(OnExit(GameState::MainMenu), cleanup_main_menu);
        // 处理按钮点击
        app.add_systems(Update, handle_button_clicks.run_if(in_state(GameState::MainMenu)));

        // 在进入Map状态时设置地图UI
        app.add_systems(OnEnter(GameState::Map), (setup_map_ui, setup_breakthrough_button, setup_cultivation_status_ui));
        // 在退出Map状态时清理地图UI
        app.add_systems(OnExit(GameState::Map), cleanup_map_ui);
        // 处理地图界面按钮点击
        app.add_systems(Update, handle_map_button_clicks.run_if(in_state(GameState::Map)));

        // 在进入Combat状态时设置战斗UI
        app.add_systems(OnEnter(GameState::Combat), setup_combat_ui);
        // 在进入Combat状态时重置玩家状态（能量、护甲等）
        app.add_systems(OnEnter(GameState::Combat), reset_player_on_combat_start);
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
        // 处理胜利延迟计时器
        app.add_systems(Update, update_victory_delay.run_if(in_state(GameState::Combat)));
        // 更新敌人死亡动画
        app.add_systems(Update, update_enemy_death_animation.run_if(in_state(GameState::Combat)));

        // 在进入Reward状态时设置奖励UI
        app.add_systems(OnEnter(GameState::Reward), setup_reward_ui);
        // 在退出Reward状态时清理奖励UI
        app.add_systems(OnExit(GameState::Reward), cleanup_reward_ui);
        // 处理奖励界面点击
        app.add_systems(Update, handle_reward_clicks.run_if(in_state(GameState::Reward)));
        // 处理卡牌/遗物悬停显示详情
        app.add_systems(Update, handle_card_hover.run_if(in_state(GameState::Reward)));
        app.add_systems(Update, handle_relic_hover.run_if(in_state(GameState::Reward)));
        // 更新鼠标位置
        app.add_systems(Update, update_mouse_position.run_if(in_state(GameState::Reward)));
        // 清理悬停面板（鼠标移开时）
        app.add_systems(Update, cleanup_hover_panels.run_if(in_state(GameState::Reward)));

        // 在进入GameOver状态时设置游戏结束UI
        app.add_systems(OnEnter(GameState::GameOver), setup_game_over_ui);
        // 在退出GameOver状态时清理游戏结束UI
        app.add_systems(OnExit(GameState::GameOver), cleanup_game_over_ui);
        // 处理游戏结束界面按钮点击
        app.add_systems(Update, handle_game_over_clicks.run_if(in_state(GameState::GameOver)));

        // 注意：商店和休息系统现在由独立的 ShopPlugin 和 RestPlugin 管理
        // 不要在这里重复注册，否则会导致系统重复注册错误
    }
}

/// 渡劫计时器
#[derive(Resource)]
struct TribulationTimer {
    /// 渡劫总时长
    total_timer: Timer,
    /// 天雷间隔
    strike_timer: Timer,
    /// 已降下天雷次数
    strikes_count: u32,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            crate::systems::animation::AnimationPlugin,
            crate::systems::particle::ParticlePlugin,
            crate::systems::screen_effect::ScreenEffectPlugin,
            crate::systems::sprite::SpritePlugin,
        ))
        .init_state::<GameState>()
        .insert_resource(VictoryDelay::new(2.0))
        .init_resource::<CurrentRewardCards>()
        .init_resource::<CurrentRewardRelic>()
        .init_resource::<HoveredCard>()
        .init_resource::<HoveredRelic>()
        .init_resource::<MousePosition>()
        .insert_resource(TribulationTimer {
            total_timer: Timer::from_seconds(10.0, TimerMode::Once),
            strike_timer: Timer::from_seconds(1.5, TimerMode::Repeating),
            strikes_count: 0,
        })
        .add_event::<EnemyAttackEvent>()
        .add_event::<RelicObtainedEvent>()
        .add_event::<RelicTriggeredEvent>()
        // ... (其他系统注册)
        .add_systems(OnEnter(GameState::Tribulation), setup_tribulation)
        .add_systems(Update, update_tribulation.run_if(in_state(GameState::Tribulation)))
        .add_systems(OnExit(GameState::Tribulation), teardown_tribulation);
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
    // 加载中文字体和Logo图片
    let chinese_font: Handle<Font> = asset_server.load("fonts/Arial Unicode.ttf");
    let logo_handle: Handle<Image> = asset_server.load("textures/logo.png");

    // 创建根节点（全屏容器）
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
            BackgroundColor(Color::srgb(0.05, 0.05, 0.05)),
        ))
        .with_children(|parent| {
            // 游戏 Logo 背景图片 - 铺满全屏
            parent.spawn((
                ImageNode::new(logo_handle),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ZIndex(-1), // 确保在最底层
            ));

            // 开始游戏按钮 - 绝对定位在屏幕中下方
            parent.spawn((
                Node {
                    width: Val::Px(220.0),
                    height: Val::Px(60.0),
                    position_type: PositionType::Absolute,
                    bottom: Val::Percent(15.0), // 距离底部 15%
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                ZIndex(1), // 确保在最顶层
                BorderColor(Color::srgba(1.0, 1.0, 1.0, 0.3)),
                BackgroundColor(Color::srgba(0.1, 0.2, 0.1, 0.85)), // 墨绿色半透明
                Button,
                StartGameButton,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("进入九界"),
                    TextFont {
                        font: chinese_font.clone(),
                        font_size: 32.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
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

/// 标记渡劫按钮，便于清理
#[derive(Component)]
struct BreakthroughButtonMarker;

/// 设置修为状态显示（左上角）
fn setup_cultivation_status_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<(&Player, &crate::components::Cultivation)>,
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
    cultivation_query: Query<&crate::components::Cultivation>,
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
                    BorderColor(Color::srgb(1.0, 0.8, 0.2)),
                    BackgroundColor(Color::srgba(0.1, 0.05, 0.2, 0.95)),
                    BreakthroughButtonMarker,
                    MapUiRoot, // 借用MapUiRoot标记，这样原有的清理逻辑能顺便清理掉它
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
                    info!("🌩️ 玩家引动九天雷劫！");
                    next_state.set(GameState::Tribulation);
                });
        }
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
pub struct MapUiRoot;

/// 地图节点按钮标记
#[derive(Component)]
pub struct MapNodeButton {
    pub node_id: u32,
}

/// 返回主菜单按钮标记
#[derive(Component)]
struct BackToMenuButton;

// ============================================================================
// 按钮交互系统
// ============================================================================

/// 处理按钮点击事件
fn handle_button_clicks(
    mut commands: Commands,
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

            // 初始化玩家牌组（如果不存在）
            commands.init_resource::<PlayerDeck>();

            // 初始化地图进度（如果不存在）
            // 注意：使用 init_resource 而不是 insert_resource，这样不会覆盖现有进度
            commands.init_resource::<MapProgress>();

            info!("开始游戏 - 玩家牌组和地图进度已初始化");

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
    mut map_progress: Option<ResMut<MapProgress>>,
    mut button_queries: ParamSet<(
        Query<(&Interaction, &MapNodeButton, &mut BackgroundColor)>,
        Query<(&Interaction, &mut BackgroundColor), With<BackToMenuButton>>,
    )>,
) {
    // 处理地图节点点击
    for (interaction, node_btn, mut color) in button_queries.p0().iter_mut() {
        if matches!(interaction, Interaction::Pressed) {
            // 获取节点类型
            let node_type: NodeType = if let Some(ref progress) = map_progress {
                progress.nodes.iter()
                    .find(|n| n.id == node_btn.node_id)
                    .map(|n| n.node_type)
                    .unwrap_or(NodeType::Normal)
            } else {
                NodeType::Normal
            };

            info!("地图节点 {} 被点击，节点类型: {:?}", node_btn.node_id, node_type);

            // 根据节点类型进入不同状态
            match node_type {
                NodeType::Shop => {
                    // 进入商店
                    if let Some(ref mut progress) = map_progress {
                        progress.set_current_node(node_btn.node_id);
                    }
                    next_state.set(GameState::Shop);
                }
                NodeType::Rest => {
                    // 休息点 - 恢复生命值
                    if let Some(ref mut progress) = map_progress {
                        progress.set_current_node(node_btn.node_id);
                    }
                    next_state.set(GameState::Rest);
                }
                NodeType::Treasure | NodeType::Normal | NodeType::Elite | NodeType::Boss => {
                    // 战斗节点
                    if let Some(ref mut progress) = map_progress {
                        progress.set_current_node(node_btn.node_id);
                    }
                    next_state.set(GameState::Combat);
                }
                NodeType::Unknown => {
                    info!("未知节点类型，忽略");
                }
            }
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
fn setup_map_ui(mut commands: Commands, asset_server: Res<AssetServer>, map_progress: Option<Res<MapProgress>>) {
    // 加载中文字体
    let chinese_font: Handle<Font> = asset_server.load("fonts/Arial Unicode.ttf");

    // 如果没有地图进度，创建新的
    let progress = if let Some(p) = map_progress {
        p.clone()
    } else {
        info!("创建新地图进度");
        MapProgress::default()
    };

    let nodes = progress.nodes.clone();

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
                    // 计算最大层数
                    let max_layer = nodes.iter().map(|n| n.position.0).max().unwrap_or(0);

                    // 按层显示节点
                    for layer in 0..=max_layer {
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
                                    if node.position.0 == layer {
                                        spawn_map_node(
                                            layer_parent,
                                            node,
                                            &chinese_font,
                                            &progress,
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
    map_progress: &MapProgress,
) {
    // 检查是否是当前节点
    let is_current = map_progress.current_node_id == Some(node.id);

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

    // 根据节点状态计算显示颜色
    let display_color = if node.completed {
        // 已完成：灰色
        Color::srgb(0.3, 0.3, 0.3)
    } else if is_current {
        // 当前节点：高亮（黄色发光效果）
        Color::srgb(1.0, 0.9, 0.3)
    } else if node.unlocked {
        // 已解锁但未访问：正常颜色
        node_color
    } else {
        // 未解锁：暗色
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

    // 边框效果
    if is_current {
        entity.insert(BorderColor(Color::srgb(1.0, 1.0, 0.0)));
        entity.insert(Node {
            border: UiRect::all(Val::Px(4.0)),
            ..default()
        });
    } else if node.completed {
        entity.insert(BorderColor(Color::srgb(0.5, 0.5, 0.5)));
        entity.insert(Node {
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        });
    }

    // 如果节点已解锁，添加按钮组件
    if node.unlocked && !node.completed {
        entity.insert(Button);
    }

    // 显示状态标记
    let status_mark = if node.completed {
        "✓"
    } else if is_current {
        "→"
    } else {
        ""
    };

    entity.with_children(|node_parent| {
            // 状态标记
            if !status_mark.is_empty() {
                node_parent.spawn((
                    Text::new(status_mark),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(5.0),
                        right: Val::Px(5.0),
                        ..default()
                    },
                ));
            }

            // 节点类型图标（用文字表示）
            node_parent.spawn((
                Text::new(get_node_icon(node.node_type)),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // 节点名称
            node_parent.spawn((
                Text::new(format!("{}{}", node_name, if node.completed { "(已完成)" } else { "" })),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::top(Val::Px(3.0)),
                    ..default()
                },
            ));

            // 节点描述（帮助用户理解节点功能）
            if node.unlocked && !node.completed {
                let node_desc = get_node_description(node.node_type);
                node_parent.spawn((
                    Text::new(node_desc),
                    TextFont {
                        font: font.clone(),
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    Node {
                        margin: UiRect::top(Val::Px(1.0)),
                        ..default()
                    },
                ));
            }
        });
}

/// 获取节点描述文字
fn get_node_description(node_type: NodeType) -> &'static str {
    match node_type {
        NodeType::Normal => "战斗",
        NodeType::Elite => "强敌",
        NodeType::Boss => "首领",
        NodeType::Rest => "恢复HP",
        NodeType::Shop => "购买",
        NodeType::Treasure => "奖励",
        NodeType::Unknown => "???",
    }
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
pub struct CombatUiRoot;

/// 结束回合按钮标记
#[derive(Component)]
struct EndTurnButton;

/// 返回地图按钮标记（战斗结束）
#[derive(Component)]
struct ReturnToMapButton;

/// 奖励UI根节点标记
#[derive(Component)]
struct RewardUiRoot;

/// 奖励卡牌按钮标记
#[derive(Component)]
pub struct RewardCardButton {
    pub card_id: u32,
}

/// 奖励遗物按钮标记
#[derive(Component)]
pub struct RewardRelicButton {
    pub relic_id: RelicId,
}

/// 卡牌悬停详情面板标记
#[derive(Component)]
struct CardHoverPanelMarker;

/// 遗物悬停详情面板标记
#[derive(Component)]
struct RelicHoverPanelMarker;

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
pub struct HandArea;

/// 设置战斗UI
fn setup_combat_ui(mut commands: Commands, asset_server: Res<AssetServer>, player_deck: Res<PlayerDeck>, mut victory_delay: ResMut<VictoryDelay>) {
    // 进入战斗时确保胜利延迟被重置（防止上一场战斗的状态泄漏）
    if victory_delay.active {
        info!("进入战斗时检测到胜利延迟仍然激活，强制重置");
        victory_delay.active = false;
        victory_delay.elapsed = 0.0;
    }
    let chinese_font: Handle<Font> = asset_server.load("fonts/Arial Unicode.ttf");

    // 注意：玩家实体由 init_player 系统统一管理，不再在此创建

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

    // 创建牌组（使用持久化的玩家牌组）
    let deck_cards = player_deck.cards.clone();
    commands.insert_resource(DeckConfig { starting_deck: deck_cards.clone(), ..default() });

    // 计算初始抽牌后剩余的卡牌
    let initial_draw = 5.min(deck_cards.len());
    let drawn_cards: Vec<Card> = deck_cards.iter().take(initial_draw).cloned().collect();
    let remaining_deck: Vec<Card> = deck_cards.iter().skip(initial_draw).cloned().collect();

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
pub fn cleanup_combat_ui(
    mut commands: Commands,
    ui_query: Query<Entity, With<CombatUiRoot>>,
    player_query: Query<Entity, With<Player>>,
    enemy_query: Query<Entity, With<Enemy>>,
    _sprite_query: Query<Entity, With<SpriteMarker>>,
    particle_query: Query<Entity, With<ParticleMarker>>,
    emitter_query: Query<Entity, With<EmitterMarker>>,
    screen_effect_query: Query<Entity, With<ScreenEffectMarker>>,
    draw_pile_query: Query<Entity, With<DrawPile>>,
    discard_pile_query: Query<Entity, With<DiscardPile>>,
    hand_query: Query<Entity, With<Hand>>,
    hand_area_query: Query<Entity, With<HandArea>>,
) {
    // 清理UI
    for entity in ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    // 注意：不再在这里清理玩家实体，玩家需要持久化以保留修仙境界
    
    // 清理敌人实体
    for entity in enemy_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    // 清理精灵实体
    for entity in _sprite_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    // 清理粒子实体
    for entity in particle_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    // 清理发射器实体
    for entity in emitter_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    // 清理屏幕特效实体
    for entity in screen_effect_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    // 牌组实体
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
    _commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut combat_state: ResMut<CombatState>,
    mut player_query: Query<&mut Player>,
    mut enemy_query: Query<&mut Enemy>,
    mut attack_events: EventWriter<EnemyAttackEvent>,
    mut effect_events: EventWriter<SpawnEffectEvent>,
    mut screen_events: EventWriter<ScreenEffectEvent>,
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

            // 敌人AI行动逻辑
            if let Ok(mut enemy) = enemy_query.get_single_mut() {
                // 先让敌人选择新意图（清空护甲并选择新行动）
                enemy.start_turn();
                info!("敌人意图: {:?}", enemy.intent);

                // 然后执行意图
                let executed_intent = enemy.execute_intent();

                match executed_intent {
                    EnemyIntent::Attack { damage } => {
                        if let Ok(mut player) = player_query.get_single_mut() {
                            // 检查是否破甲（护甲被完全击破）
                            let block_broken = player.block > 0 && damage >= player.block;

                            player.take_damage(damage);
                            info!("玩家受到{}点伤害，剩余HP: {}", damage, player.hp);

                            // 发送攻击事件，触发动画
                            attack_events.send(EnemyAttackEvent::new(damage, block_broken));

                            // 触发粒子特效（火焰+受击）
                            effect_events.send(SpawnEffectEvent {
                                effect_type: EffectType::Fire,
                                position: Vec3::new(0.0, 100.0, 999.0),
                                burst: true,
                                count: 30,
                            });
                            effect_events.send(SpawnEffectEvent {
                                effect_type: EffectType::Hit,
                                position: Vec3::new(0.0, -200.0, 999.0),
                                burst: true,
                                count: 20,
                            });

                            // 触发屏幕特效（震动+红色闪光）
                            screen_events.send(ScreenEffectEvent::Shake {
                                trauma: 0.4,
                                decay: 4.0,
                            });
                            screen_events.send(ScreenEffectEvent::Flash {
                                color: Color::srgba(1.0, 0.0, 0.0, 0.6),
                                duration: 0.15,
                            });
                        }
                    }
                    EnemyIntent::Defend { block } => {
                        info!("{} 获得了 {} 点护甲", enemy.name, block);
                        // 触发冰霜特效
                        effect_events.send(SpawnEffectEvent {
                            effect_type: EffectType::Ice,
                            position: Vec3::new(0.0, 100.0, 999.0),
                            burst: true,
                            count: 25,
                        });
                    }
                    EnemyIntent::Buff { strength } => {
                        info!("{} 获得了 {} 点攻击力", enemy.name, strength);
                        // 触发强化特效（紫色）
                        effect_events.send(SpawnEffectEvent {
                            effect_type: EffectType::Victory,
                            position: Vec3::new(0.0, 100.0, 999.0),
                            burst: true,
                            count: 20,
                        });
                    }
                    EnemyIntent::Wait => {
                        info!("{} 等待中", enemy.name);
                    }
                }

                // 检查战斗是否结束
                if let Ok(player) = player_query.get_single() {
                    if player.hp <= 0 {
                        info!("玩家败北！");
                        // TODO: 游戏结束逻辑
                    }
                }
            }

            // 新回合开始
            if let Ok(mut player) = player_query.get_single_mut() {
                player.start_turn();
                info!("玩家新回合：护甲清零");
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
// 战斗开始初始化系统
// ============================================================================

/// 战斗开始时重置玩家状态
fn reset_player_on_combat_start(mut player_query: Query<&mut Player>) {
    info!("reset_player_on_combat_start 被调用");
    if let Ok(mut player) = player_query.get_single_mut() {
        player.energy = player.max_energy; // 重置能量
        player.block = 0; // 清除护甲
        player.turn = 1; // 重置回合数
        info!("战斗开始：重置玩家状态 - 能量: {}/{}, 护甲: {}, 回合: {}",
              player.energy, player.max_energy, player.block, player.turn);
    } else {
        info!("警告：战斗开始时找不到玩家实体");
    }
}
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
    _commands: Commands,
    card_query: Query<(&Interaction, &HandCard), (Changed<Interaction>, With<HandCard>)>,
    mut player_query: Query<&mut Player>,
    mut hand_query: Query<&mut Hand>,
    mut draw_pile_query: Query<&mut DrawPile>,
    mut discard_pile_query: Query<&mut DiscardPile>,
    mut enemy_query: Query<&mut Enemy>,
    mut effect_events: EventWriter<SpawnEffectEvent>,
    mut screen_events: EventWriter<ScreenEffectEvent>,
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
                        &mut player_query,
                        &mut enemy_query,
                        &mut hand_query,
                        &mut draw_pile_query,
                        &mut discard_pile_query,
                        &mut effect_events,
                        &mut screen_events,
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
    player_query: &mut Query<&mut Player>,
    enemy_query: &mut Query<&mut Enemy>,
    hand_query: &mut Query<&mut Hand>,
    draw_pile_query: &mut Query<&mut DrawPile>,
    discard_pile_query: &mut Query<&mut DiscardPile>,
    effect_events: &mut EventWriter<SpawnEffectEvent>,
    screen_events: &mut EventWriter<ScreenEffectEvent>,
) {
    match effect {
        CardEffect::DealDamage { amount } => {
            if let Ok(mut enemy) = enemy_query.get_single_mut() {
                enemy.take_damage(*amount);
                info!("卡牌效果：对敌人造成 {} 点伤害，敌人剩余HP: {}", amount, enemy.hp);
                // 触发火焰特效
                effect_events.send(SpawnEffectEvent {
                    effect_type: EffectType::Fire,
                    position: Vec3::new(0.0, 100.0, 999.0),
                    burst: true,
                    count: 30,
                });
                // 触发屏幕震动（轻）
                screen_events.send(ScreenEffectEvent::Shake {
                    trauma: 0.2,
                    decay: 6.0,
                });
            }
        }
        CardEffect::GainBlock { amount } => {
            if let Ok(mut player) = player_query.get_single_mut() {
                let old_block = player.block;
                player.gain_block(*amount);
                info!("卡牌效果：获得 {} 点护甲，{} -> {}", amount, old_block, player.block);
                // 触发冰霜特效（护甲）
                effect_events.send(SpawnEffectEvent {
                    effect_type: EffectType::Ice,
                    position: Vec3::new(0.0, -200.0, 999.0),
                    burst: true,
                    count: 25,
                });
            }
        }
        CardEffect::Heal { amount } => {
            if let Ok(mut player) = player_query.get_single_mut() {
                let old_hp = player.hp;
                player.hp = (player.hp + amount).min(player.max_hp);
                info!("卡牌效果：回复 {} 点生命，{} -> {}", amount, old_hp, player.hp);
                // 触发治疗粒子特效
                effect_events.send(SpawnEffectEvent {
                    effect_type: EffectType::Heal,
                    position: Vec3::new(0.0, -200.0, 999.0),
                    burst: true,
                    count: 20,
                });
                // 触发白色闪光
                screen_events.send(ScreenEffectEvent::white_flash(0.3));
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
    state: Res<State<GameState>>,
    mut player_query: Query<(&mut Player, &mut crate::components::Cultivation)>,
    enemy_query: Query<&Enemy>,
    _sprite_query: Query<(Entity, &Sprite, &Children)>,
    _enemy_sprite_marker_query: Query<&EnemySpriteMarker>,
    mut next_state: ResMut<NextState<GameState>>,
    _commands: Commands,
    mut effect_events: EventWriter<SpawnEffectEvent>,
    mut screen_events: EventWriter<ScreenEffectEvent>,
    mut victory_events: EventWriter<VictoryEvent>,
    mut victory_delay: ResMut<VictoryDelay>,
) {
    // 额外检查：确保当前确实是Combat状态
    // 防止在同一帧内状态切换后仍执行此系统
    if **state != GameState::Combat {
        return;
    }

    // 检查敌人是否死亡
    if let Ok(enemy) = enemy_query.get_single() {
        if enemy.is_dead() {
            // 检查是否已经触发过胜利（防止重复触发）
            if victory_delay.active {
                // 已经触发，等待延迟结束（每帧都会到这里，不打印日志）
                return;
            }

            info!("敌人被击败！战斗胜利！");

            // 获得感悟（移除自动突破）
            if let Ok((_player, mut cultivation)) = player_query.get_single_mut() {
                let insight_gain = 50; // 基础获得50感悟
                cultivation.gain_insight(insight_gain);
                info!("【修仙】获得 {} 点感悟，当前总感悟: {}/{}", insight_gain, cultivation.insight, cultivation.get_threshold());
                
                if cultivation.can_breakthrough() {
                    info!("✨【机缘已至】感悟已满，可在地图界面开启“渡劫”！");
                }
            } else {
                warn!("⚠️【警告】战斗胜利但未能获取玩家修为数据！请检查 Player 是否正确绑定了 Cultivation 组件。");
            }

            // 触发胜利粒子特效（金色星形）
            effect_events.send(SpawnEffectEvent {
                effect_type: EffectType::Victory,
                position: Vec3::new(0.0, 100.0, 999.0),
                burst: true,
                count: 50,
            });
            // 多次触发形成爆发效果
            effect_events.send(SpawnEffectEvent {
                effect_type: EffectType::Victory,
                position: Vec3::new(-50.0, 80.0, 999.0),
                burst: true,
                count: 30,
            });
            effect_events.send(SpawnEffectEvent {
                effect_type: EffectType::Victory,
                position: Vec3::new(50.0, 80.0, 999.0),
                burst: true,
                count: 30,
            });

            // 触发金色边缘闪光
            screen_events.send(ScreenEffectEvent::Flash {
                color: Color::srgba(1.0, 0.9, 0.3, 0.5),
                duration: 0.4,
            });

            // 触发胜利事件（可用于其他系统）
            victory_events.send(VictoryEvent);

            // 启动胜利延迟计时器，不立即切换状态
            victory_delay.active = true;
            victory_delay.elapsed = 0.0;
            info!("启动胜利延迟，{}秒后进入奖励界面", victory_delay.duration);
            return;
        }
    }

    // 检查玩家是否死亡
    if let Ok(player_data) = player_query.get_single() {
        if player_data.0.hp <= 0 {
            info!("玩家败北！身陨道消...");
            next_state.set(GameState::GameOver);
        }
    }
}

/// 更新敌人死亡动画
fn update_enemy_death_animation(
    mut commands: Commands,
    mut query: Query<(Entity, &mut EnemyDeathAnimation, &mut Sprite)>,
    time: Res<Time>,
) {
    for (entity, mut anim, mut sprite) in query.iter_mut() {
        anim.elapsed += time.delta_secs();
        anim.progress = (anim.elapsed / anim.duration).min(1.0);

        // 淡出效果：减少透明度
        let alpha = 1.0 - anim.progress;
        sprite.color.set_alpha(alpha);

        // 缩放效果：敌人逐渐缩小
        let scale = 1.0 - (anim.progress * 0.3); // 缩小到 70%
        sprite.custom_size = Some(Vec2::new(200.0, 200.0) * scale);

        // 动画完成后移除敌人实体
        if anim.progress >= 1.0 {
            commands.entity(entity).despawn_recursive();
            info!("敌人死亡动画完成，已移除敌人实体");
        }
    }
}

/// 更新胜利延迟计时器
fn update_victory_delay(
    mut victory_delay: ResMut<VictoryDelay>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    ui_query: Query<Entity, With<CombatUiRoot>>,
    _sprite_query: Query<Entity, With<SpriteMarker>>,
) {
    if !victory_delay.active {
        return;
    }

    victory_delay.elapsed += time.delta_secs();

    // 只在激活时输出日志
    info!("胜利延迟进行中: {:.2}/{:.2}", victory_delay.elapsed, victory_delay.duration);

    if victory_delay.elapsed >= victory_delay.duration {
        // 延迟结束，切换到奖励界面
        info!("胜利延迟结束，进入奖励界面！");

        // 先设置 active = false，防止 check_combat_end 再次触发
        victory_delay.active = false;
        victory_delay.elapsed = 0.0;

        // 清理战斗UI，避免遮挡
        let ui_count = ui_query.iter().count();
        info!("找到 {} 个战斗UI实体需要清理", ui_count);

        for entity in ui_query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        // 最后切换状态
        next_state.set(GameState::Reward);
        info!("已切换到 Reward 状态");
    }
}

// ============================================================================
// 奖励系统
// ============================================================================

/// 设置奖励界面
fn setup_reward_ui(mut commands: Commands, asset_server: Res<AssetServer>, relic_collection: Res<RelicCollection>, mut reward_cards_resource: ResMut<CurrentRewardCards>, mut reward_relic_resource: ResMut<CurrentRewardRelic>) {
    info!("设置奖励界面");

    // 生成随机奖励卡牌（3张）
    let reward_cards = CardPool::random_rewards(3);

    // 存储奖励卡牌到资源中（供悬停系统使用）
    reward_cards_resource.cards = reward_cards.clone();

    // 生成随机遗物奖励
    let relic_reward = generate_relic_reward(&relic_collection);
    let show_relic = relic_reward.is_some();

    // 存储遗物奖励到资源中（供悬停系统使用）
    reward_relic_resource.relic = relic_reward.clone();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(30.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.15)),
            RewardUiRoot,
        ))
        .with_children(|parent| {
            // 标题
            parent.spawn((
                Text::new("战斗胜利！选择奖励"),
                TextFont {
                    font: asset_server.load("fonts/Arial Unicode.ttf"),
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.9, 0.3)),
                TextLayout::new_with_justify(JustifyText::Center),
            ));

            // 奖励选项容器
            parent
                .spawn(Node {
                    width: Val::Percent(80.0),
                    height: Val::Auto,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(20.0),
                    ..default()
                })
                .with_children(|parent| {
                    // 卡牌容器
                    parent
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            height: Val::Auto,
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            column_gap: Val::Px(30.0),
                            ..default()
                        })
                        .with_children(|parent| {
                            // 为每张奖励卡创建UI
                            for (index, card) in reward_cards.iter().enumerate() {
                                create_reward_card(parent, card, index, &asset_server);
                            }
                        });

                    // 遗物选项（如果可用）
                    if show_relic {
                        if let Some(relic) = relic_reward {
                            parent.spawn(Node {
                                width: Val::Percent(100.0),
                                height: Val::Auto,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                margin: UiRect::top(Val::Px(20.0)),
                                ..default()
                            })
                            .with_children(|parent| {
                                create_relic_reward_option(parent, relic, &asset_server);
                            });
                        }
                    }
                });

            // 跳过按钮
            parent
                .spawn((
                    Button,
                    BackgroundColor(Color::srgb(0.3, 0.3, 0.4)),
                    BorderColor(Color::srgb(0.5, 0.5, 0.6)),
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(50.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("跳过奖励"),
                        TextFont {
                            font: asset_server.load("fonts/Arial Unicode.ttf"),
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 1.0, 1.0)),
                    ));
                })
                .observe(|_entity: Trigger<Pointer<Click>>, mut next_state: ResMut<NextState<GameState>>, mut map_progress: ResMut<MapProgress>| {
                    info!("跳过奖励");
                    // 标记当前节点为完成，解锁下一层
                    map_progress.complete_current_node();
                    info!("节点已完成，已解锁下一层");
                    next_state.set(GameState::Map);
                });
        }); // commands.with_children
} // setup_reward_ui 函数结束

/// 创建单张奖励卡UI
fn create_reward_card(parent: &mut ChildBuilder, card: &Card, _index: usize, asset_server: &AssetServer) {
    let card_color = match card.card_type {
        CardType::Attack => Color::srgb(0.8, 0.2, 0.2),
        CardType::Defense => Color::srgb(0.2, 0.5, 0.8),
        CardType::Skill => Color::srgb(0.2, 0.7, 0.3),
        CardType::Power => Color::srgb(0.7, 0.3, 0.8),
    };

    let rarity_color = match card.rarity {
        CardRarity::Common => Color::srgb(0.7, 0.7, 0.7),
        CardRarity::Uncommon => Color::srgb(0.3, 0.8, 0.9),
        CardRarity::Rare => Color::srgb(0.9, 0.7, 0.2),
        CardRarity::Special => Color::srgb(0.9, 0.4, 0.9),
    };

    parent
        .spawn((
            Button,
            BackgroundColor(card_color),
            BorderColor(rarity_color),
            Node {
                width: Val::Px(180.0),
                height: Val::Px(260.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(8.0),
                border: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            RewardCardButton { card_id: card.id },
        ))
        .with_children(|parent| {
            // 稀有度标签
            parent.spawn((
                Text::new(format!("{:?}", card.rarity)),
                TextFont {
                    font: asset_server.load("fonts/Arial Unicode.ttf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(rarity_color),
            ));

            // 卡牌名称
            parent.spawn((
                Text::new(card.name.clone()),
                TextFont {
                    font: asset_server.load("fonts/Arial Unicode.ttf"),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                TextLayout::new_with_justify(JustifyText::Center),
            ));

            // 能量消耗
            parent.spawn((
                Text::new(format!("能量: {}", card.cost)),
                TextFont {
                    font: asset_server.load("fonts/Arial Unicode.ttf"),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.9, 0.3)),
            ));

            // 卡牌描述
            parent.spawn((
                Text::new(card.description.clone()),
                TextFont {
                    font: asset_server.load("fonts/Arial Unicode.ttf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
                TextLayout::new_with_justify(JustifyText::Center),
            ));

            // 类型标签
            parent.spawn((
                Text::new(format!("{:?}", card.card_type)),
                TextFont {
                    font: asset_server.load("fonts/Arial Unicode.ttf"),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
            ));
        });
}

/// 清理奖励界面
fn cleanup_reward_ui(
    mut commands: Commands,
    ui_query: Query<Entity, With<RewardUiRoot>>,
    particle_query: Query<Entity, With<ParticleMarker>>,
    emitter_query: Query<Entity, With<EmitterMarker>>,
    screen_effect_query: Query<Entity, With<ScreenEffectMarker>>,
    card_hover_query: Query<Entity, With<CardHoverPanelMarker>>,
    relic_hover_query: Query<Entity, With<RelicHoverPanelMarker>>,
) {
    info!("【清理奖励界面】清理所有奖励相关UI");

    // 清理奖励UI
    for entity in ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    // 清理粒子实体
    for entity in particle_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    // 清理发射器实体
    for entity in emitter_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    // 清理屏幕特效实体
    for entity in screen_effect_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // 清理悬停面板（重要：防止悬停面板在状态切换后残留）
    for entity in card_hover_query.iter() {
        info!("【清理奖励界面】清理卡牌悬停面板");
        commands.entity(entity).despawn_recursive();
    }
    for entity in relic_hover_query.iter() {
        info!("【清理奖励界面】清理遗物悬停面板");
        commands.entity(entity).despawn_recursive();
    }
}

/// 处理奖励卡牌点击
fn handle_reward_clicks(
    interactions: Query<
        (&Interaction, &RewardCardButton),
        (Changed<Interaction>, With<RewardCardButton>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut player_deck: ResMut<PlayerDeck>,
    mut map_progress: ResMut<MapProgress>,
) {
    for (interaction, reward_btn) in interactions.iter() {
        if matches!(interaction, Interaction::Pressed) {
            info!("选择了奖励卡牌 ID: {}", reward_btn.card_id);

            // 从卡牌池找到对应的卡牌
            let all_cards = CardPool::all_cards();
            if let Some(card) = all_cards.iter().find(|c| c.id == reward_btn.card_id) {
                let card_name = card.name.clone();
                // 添加到玩家牌组
                let mut new_card = card.clone();
                new_card.id = 1000 + player_deck.cards.len() as u32;
                player_deck.add_card(new_card);
                info!("卡牌「{}」已加入牌组，当前牌组大小: {}", card_name, player_deck.len());
            }

            // 标记当前节点为完成，解锁下一层
            map_progress.complete_current_node();
            info!("节点已完成，已解锁下一层");

            // 返回地图
            next_state.set(GameState::Map);
        }
    }
}

// ============================================================================
// 游戏结束系统
// ============================================================================

/// 游戏结束UI根节点标记
#[derive(Component)]
struct GameOverUiRoot;

/// 重新开始按钮标记
#[derive(Component)]
struct RestartButton;

/// 设置游戏结束界面
fn setup_game_over_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    map_progress: Res<MapProgress>,
) {
    info!("设置游戏结束界面");

    // 获取当前层数
    let current_layer = map_progress.current_layer;

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(30.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.05, 0.05, 0.1)),
            GameOverUiRoot,
        ))
        .with_children(|parent| {
            // 标题
            parent.spawn((
                Text::new("你败北"),
                TextFont {
                    font: asset_server.load("fonts/Arial Unicode.ttf"),
                    font_size: 64.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.2, 0.2)),
                TextLayout::new_with_justify(JustifyText::Center),
            ));

            // 层数信息
            parent.spawn((
                Text::new(format!("到达层数：{} 层", current_layer)),
                TextFont {
                    font: asset_server.load("fonts/Arial Unicode.ttf"),
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                TextLayout::new_with_justify(JustifyText::Center),
                Node {
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
            ));

            // 按钮容器
            parent
                .spawn(Node {
                    width: Val::Auto,
                    height: Val::Auto,
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(20.0),
                    margin: UiRect::top(Val::Px(40.0)),
                    ..default()
                })
                .with_children(|button_parent| {
                    // 重新开始按钮
                    button_parent
                        .spawn((
                            Button,
                            BackgroundColor(Color::srgb(0.2, 0.5, 0.8)),
                            BorderColor(Color::srgb(0.3, 0.6, 0.9)),
                            Node {
                                width: Val::Px(160.0),
                                height: Val::Px(50.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            RestartButton,
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("重新开始"),
                                TextFont {
                                    font: asset_server.load("fonts/Arial Unicode.ttf"),
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                    // 返回主菜单按钮
                    button_parent
                        .spawn((
                            Button,
                            BackgroundColor(Color::srgb(0.3, 0.3, 0.4)),
                            BorderColor(Color::srgb(0.5, 0.5, 0.6)),
                            Node {
                                width: Val::Px(160.0),
                                height: Val::Px(50.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                        ))
                        .observe(
                            |_entity: Trigger<Pointer<Click>>,
                            mut next_state: ResMut<NextState<GameState>>| {
                                info!("游戏结束：返回主菜单");
                                next_state.set(GameState::MainMenu);
                            },
                        )
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("返回主菜单"),
                                TextFont {
                                    font: asset_server.load("fonts/Arial Unicode.ttf"),
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                });
        });
}

/// 清理游戏结束界面
fn cleanup_game_over_ui(mut commands: Commands, query: Query<Entity, With<GameOverUiRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// ============================================================================
// 遗物奖励辅助函数
// ============================================================================

/// 生成遗物奖励（基于当前拥有的遗物，避免重复）
fn generate_relic_reward(relic_collection: &RelicCollection) -> Option<Relic> {
    use rand::Rng;

    // 获取所有未拥有的遗物
    let all_relics = vec![
        Relic::burning_blood(),
        Relic::bag_of_preparation(),
        Relic::anchor(),
        Relic::strange_spoon(),
    ];

    let available_relics: Vec<_> = all_relics
        .into_iter()
        .filter(|r| !relic_collection.has(r.id))
        .collect();

    if available_relics.is_empty() {
        info!("没有可用的遗物奖励");
        return None;
    }

    // 随机选择一个可用遗物
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..available_relics.len());
    Some(available_relics[index].clone())
}

/// 创建遗物奖励选项UI
fn create_relic_reward_option(parent: &mut ChildBuilder, relic: Relic, asset_server: &AssetServer) {

    let rarity_color = relic.rarity.color();
    let text_color = relic.rarity.text_color();

    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(280.0),
                height: Val::Auto,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(15.0)),
                align_items: AlignItems::Center,
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(rarity_color),
            BorderRadius::all(Val::Px(8.0)),
            BorderColor(Color::srgb(0.3, 0.3, 0.3)),
            RewardRelicButton { relic_id: relic.id },
        ))
        .with_children(|parent| {
            // 稀有度标签
            parent.spawn((
                Text::new(format!("{:?}", relic.rarity)),
                TextFont {
                    font: asset_server.load("fonts/Arial Unicode.ttf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(text_color),
            ));

            // 遗物名称
            parent.spawn((
                Text::new(relic.name.clone()),
                TextFont {
                    font: asset_server.load("fonts/Arial Unicode.ttf"),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(text_color),
                TextLayout::new_with_justify(JustifyText::Center),
            ));

            // 遗物描述
            parent.spawn((
                Text::new(relic.description.clone()),
                TextFont {
                    font: asset_server.load("fonts/Arial Unicode.ttf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(text_color),
                TextLayout::new_with_justify(JustifyText::Center),
            ));
        })
        .observe(move |_entity: Trigger<Pointer<Click>>, mut relic_collection: ResMut<RelicCollection>, mut next_state: ResMut<NextState<GameState>>, mut map_progress: ResMut<MapProgress>| {
            // 添加遗物到背包
            let added = relic_collection.add_relic(relic.clone());
            if added {
                info!("【遗物奖励】获得了遗物: {}", relic.name);
            } else {
                warn!("【遗物奖励】遗物已存在，未能添加: {}", relic.name);
            }

            // 标记当前节点为完成，解锁下一层
            map_progress.complete_current_node();
            next_state.set(GameState::Map);
        });
}

/// 处理游戏结束界面按钮点击
fn handle_game_over_clicks(
    interactions: Query<
        (&Interaction, &RestartButton),
        (Changed<Interaction>, With<RestartButton>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut player_deck: ResMut<PlayerDeck>,
    mut map_progress: ResMut<MapProgress>,
) {
    for (interaction, _restart_btn) in interactions.iter() {
        if matches!(interaction, Interaction::Pressed) {
            info!("游戏结束：重新开始游戏");

            // 重置玩家牌组
            player_deck.reset();
            info!("玩家牌组已重置，大小: {}", player_deck.len());

            // 重置地图进度
            map_progress.reset();
            info!("地图进度已重置，当前层数: {}", map_progress.current_layer);

            // 进入地图状态
            next_state.set(GameState::Map);
        }
    }
}

// ============================================================================
// 渡劫系统 (Tribulation)
// ============================================================================

fn setup_tribulation(
    mut timer: ResMut<TribulationTimer>,
    mut screen_events: EventWriter<ScreenEffectEvent>,
) {
    info!("🌩️ 天地震动，雷劫将至！");
    timer.total_timer.reset();
    timer.strike_timer.reset();
    timer.strikes_count = 0;

    // 初始屏幕变暗特效
    screen_events.send(ScreenEffectEvent::Flash { 
        color: Color::srgba(0.0, 0.0, 0.0, 0.8), 
        duration: 1.0 
    });
}

fn update_tribulation(
    time: Res<Time>,
    mut timer: ResMut<TribulationTimer>,
    mut player_query: Query<&mut Player>,
    mut next_state: ResMut<NextState<GameState>>,
    mut screen_events: EventWriter<ScreenEffectEvent>,
    mut effect_events: EventWriter<SpawnEffectEvent>,
) {
    // 推进总进度
    timer.total_timer.tick(time.delta());
    if timer.total_timer.finished() {
        info!("🌩️ 雷云散去，渡劫成功！");
        next_state.set(GameState::Map);
        return;
    }

    // 推进天雷间隔
    timer.strike_timer.tick(time.delta());
    if timer.strike_timer.just_finished() {
        timer.strikes_count += 1;
        
        if let Ok(mut player) = player_query.get_single_mut() {
            // 天雷伤害：固定伤害或百分比
            let damage = (player.max_hp as f32 * 0.12).max(10.0) as i32;
            player.hp -= damage;
            
            info!("⚡ 第 {} 道天雷落下！造成 {} 点伤害，剩余道行: {}", timer.strikes_count, damage, player.hp);

            // 视觉特效：白光闪烁 + 剧烈震动
            screen_events.send(ScreenEffectEvent::Flash { 
                color: Color::WHITE, 
                duration: 0.2 
            });
            screen_events.send(ScreenEffectEvent::Shake { 
                trauma: 10.0, 
                decay: 0.3 
            });
            
            // 粒子特效：电光火石
            effect_events.send(SpawnEffectEvent {
                effect_type: EffectType::Hit,
                position: Vec3::new(0.0, 0.0, 100.0),
                burst: true,
                count: 30,
            });

            // 检查陨落
            if player.hp <= 0 {
                info!("💀 渡劫失败，身陨道消...");
                next_state.set(GameState::GameOver);
            }
        }
    }
}

fn teardown_tribulation(
    mut player_query: Query<(&mut Player, &mut crate::components::Cultivation)>,
    mut effect_events: EventWriter<SpawnEffectEvent>,
) {
    if let Ok((mut player, mut cultivation)) = player_query.get_single_mut() {
        // 只有在没死的情况下才进入这里（由于 GameOver 也会触发出状态，这里加个判断）
        if player.hp > 0 {
            let old_realm = cultivation.realm;
            if cultivation.breakthrough() {
                let hp_bonus = cultivation.get_hp_bonus();
                player.max_hp += hp_bonus;
                player.hp += hp_bonus;
                
                info!("✨【破境成功】成功晋升至 {:?}！道行大进，上限增加 {} 点", cultivation.realm, hp_bonus);
                
                // 成功的金色光辉
                effect_events.send(SpawnEffectEvent {
                    effect_type: EffectType::Victory,
                    position: Vec3::ZERO,
                    burst: true,
                    count: 150,
                });
            }
        }
    }
}

/// 当前奖励的卡牌列表
#[derive(Resource, Default)]
struct CurrentRewardCards {
    cards: Vec<Card>,
}

/// 当前奖励的遗物
#[derive(Resource, Default)]
struct CurrentRewardRelic {
    relic: Option<Relic>,
}

/// 当前悬停的卡牌数据
#[derive(Resource, Default)]
pub struct HoveredCard {
    pub card_id: Option<u32>,
}

/// 当前悬停的遗物数据
#[derive(Resource, Default)]
pub struct HoveredRelic {
    pub relic_id: Option<RelicId>,
}

/// 鼠标位置（用于悬停面板定位）
#[derive(Resource, Default)]
struct MousePosition {
    x: f32,
    y: f32,
}

/// 处理卡牌悬停
fn handle_card_hover(
    interactions: Query<(&Interaction, &RewardCardButton), Changed<Interaction>>,
    mut hovered_card: ResMut<HoveredCard>,
    mut commands: Commands,
    reward_cards: Res<CurrentRewardCards>,
    asset_server: Res<AssetServer>,
    mouse_position: Res<MousePosition>,
    existing_panels: Query<Entity, With<CardHoverPanelMarker>>,
) {
    for (interaction, card_button) in interactions.iter() {
        match interaction {
            Interaction::Hovered => {
                info!("【悬停】卡牌 ID: {}", card_button.card_id);

                // 更新悬停状态
                hovered_card.card_id = Some(card_button.card_id);

                // 清除旧面板
                for panel in existing_panels.iter() {
                    commands.entity(panel).despawn_recursive();
                }

                // 从当前奖励卡牌中查找对应的卡牌
                if let Some(card) = reward_cards.cards.iter().find(|c| c.id == card_button.card_id) {
                    spawn_card_hover_panel(&mut commands, card, &asset_server, &mouse_position);
                }
            }
            Interaction::None => {
                // 鼠标移开，直接清理面板
                if hovered_card.card_id == Some(card_button.card_id) {
                    info!("【悬停】鼠标从卡牌 {} 移开，开始清理", card_button.card_id);
                    hovered_card.card_id = None;

                    // 立即清理所有卡牌面板
                    for panel in existing_panels.iter() {
                        info!("【悬停】清理卡牌面板");
                        commands.entity(panel).despawn_recursive();
                    }
                }
            }
            _ => {}
        }
    }
}

/// 处理遗物悬停
fn handle_relic_hover(
    interactions: Query<(&Interaction, &RewardRelicButton), Changed<Interaction>>,
    mut hovered_relic: ResMut<HoveredRelic>,
    mut commands: Commands,
    reward_relic: Res<CurrentRewardRelic>,
    asset_server: Res<AssetServer>,
    mouse_position: Res<MousePosition>,
    existing_panels: Query<Entity, With<RelicHoverPanelMarker>>,
) {
    for (interaction, relic_button) in interactions.iter() {
        match interaction {
            Interaction::Hovered => {
                info!("【悬停】遗物 ID: {:?}", relic_button.relic_id);

                // 更新悬停状态
                hovered_relic.relic_id = Some(relic_button.relic_id);

                // 清除旧面板
                for panel in existing_panels.iter() {
                    commands.entity(panel).despawn_recursive();
                }

                // 从当前奖励遗物中获取数据
                if let Some(relic) = &reward_relic.relic {
                    if relic.id == relic_button.relic_id {
                        spawn_relic_hover_panel(&mut commands, relic, &asset_server, &mouse_position);
                    }
                }
            }
            Interaction::None => {
                // 鼠标移开，直接清理面板
                if hovered_relic.relic_id == Some(relic_button.relic_id) {
                    info!("【悬停】鼠标从遗物 {:?} 移开，开始清理", relic_button.relic_id);
                    hovered_relic.relic_id = None;

                    // 立即清理所有遗物面板
                    for panel in existing_panels.iter() {
                        info!("【悬停】清理遗物面板");
                        commands.entity(panel).despawn_recursive();
                    }
                }
            }
            _ => {}
        }
    }
}

/// 更新鼠标位置
fn update_mouse_position(
    mut mouse_position: ResMut<MousePosition>,
    q_windows: Query<&Window>,
) {
    if let Ok(window) = q_windows.get_single() {
        if let Some(position) = window.cursor_position() {
            mouse_position.x = position.x;
            mouse_position.y = position.y;
        }
    }
}

/// 清理悬停面板
fn cleanup_hover_panels(
    hovered_card: Res<HoveredCard>,
    hovered_relic: Res<HoveredRelic>,
    mut commands: Commands,
    card_panels: Query<Entity, With<CardHoverPanelMarker>>,
    relic_panels: Query<Entity, With<RelicHoverPanelMarker>>,
) {
    // 记录当前状态
    let card_panel_count = card_panels.iter().count();
    let relic_panel_count = relic_panels.iter().count();

    if card_panel_count > 0 || relic_panel_count > 0 {
        info!("【清理系统】检查清理 - 卡牌悬停: {:?}, 遗物悬停: {:?}, 卡牌面板: {}, 遗物面板: {}",
            hovered_card.card_id, hovered_relic.relic_id, card_panel_count, relic_panel_count);
    }

    // 如果没有悬停的卡牌，清理卡牌面板
    if hovered_card.card_id.is_none() {
        if card_panel_count > 0 {
            info!("【清理系统】清理 {} 个卡牌面板", card_panel_count);
        }
        for panel in card_panels.iter() {
            commands.entity(panel).despawn_recursive();
        }
    }

    // 如果没有悬停的遗物，清理遗物面板
    if hovered_relic.relic_id.is_none() {
        if relic_panel_count > 0 {
            info!("【清理系统】清理 {} 个遗物面板", relic_panel_count);
        }
        for panel in relic_panels.iter() {
            commands.entity(panel).despawn_recursive();
        }
    }
}

/// 创建卡牌悬停详情面板
fn spawn_card_hover_panel(commands: &mut Commands, card: &Card, asset_server: &AssetServer, mouse_pos: &MousePosition) {
    let card_color = match card.card_type {
        CardType::Attack => Color::srgb(0.8, 0.2, 0.2),
        CardType::Defense => Color::srgb(0.2, 0.5, 0.8),
        CardType::Skill => Color::srgb(0.2, 0.7, 0.3),
        CardType::Power => Color::srgb(0.7, 0.3, 0.8),
    };

    let rarity_color = match card.rarity {
        CardRarity::Common => Color::srgb(0.7, 0.7, 0.7),
        CardRarity::Uncommon => Color::srgb(0.3, 0.8, 0.9),
        CardRarity::Rare => Color::srgb(0.9, 0.7, 0.2),
        CardRarity::Special => Color::srgb(0.9, 0.4, 0.9),
    };

    // 计算面板位置（跟随鼠标，但避免超出屏幕）
    const PANEL_WIDTH: f32 = 300.0;
    const OFFSET_X: f32 = 20.0;
    const OFFSET_Y: f32 = 20.0;
    const WINDOW_WIDTH: f32 = 1280.0;
    const WINDOW_HEIGHT: f32 = 720.0;

    let mut x = mouse_pos.x + OFFSET_X;
    let mut y = mouse_pos.y + OFFSET_Y;

    // 如果面板超出右边界，从左侧显示
    if x + PANEL_WIDTH > WINDOW_WIDTH {
        x = mouse_pos.x - PANEL_WIDTH - OFFSET_X;
    }

    // 如果面板超出底部，从上方显示
    if y + 200.0 > WINDOW_HEIGHT {  // 假设面板高度约200px
        y = mouse_pos.y - 200.0 - OFFSET_Y;
    }

    let (position_left, position_right) = if x + PANEL_WIDTH > WINDOW_WIDTH {
        (None, Some(Val::Px(WINDOW_WIDTH - x)))
    } else {
        (Some(Val::Px(x)), None)
    };

    let (position_top, position_bottom) = if y + 200.0 > WINDOW_HEIGHT {
        (Some(Val::Px(WINDOW_HEIGHT - y)), None)
    } else {
        (Some(Val::Px(y)), None)
    };

    let mut node = Node {
        position_type: PositionType::Absolute,
        width: Val::Px(PANEL_WIDTH),
        height: Val::Auto,
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(Val::Px(15.0)),
        row_gap: Val::Px(8.0),
        border: UiRect::all(Val::Px(2.0)),
        ..default()
    };

    if let Some(left) = position_left {
        node.left = left;
    }
    if let Some(right) = position_right {
        node.right = right;
    }
    if let Some(top) = position_top {
        node.top = top;
    }
    if let Some(bottom) = position_bottom {
        node.bottom = bottom;
    }

    commands
        .spawn((
            node,
            BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.95)),
            BorderColor(rarity_color),
            CardHoverPanelMarker,
        ))
        .with_children(|parent| {
            // 稀有度标签
            parent.spawn((
                Text::new(format!("{:?}", card.rarity)),
                TextFont {
                    font: asset_server.load("fonts/Arial Unicode.ttf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(rarity_color),
            ));

            // 卡牌名称
            parent.spawn((
                Text::new(card.name.clone()),
                TextFont {
                    font: asset_server.load("fonts/Arial Unicode.ttf"),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(card_color),
                TextLayout::new_with_justify(JustifyText::Center),
            ));

            // 能量消耗
            parent.spawn((
                Text::new(format!("能量: {}", card.cost)),
                TextFont {
                    font: asset_server.load("fonts/Arial Unicode.ttf"),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.9, 0.3)),
            ));

            // 卡牌类型
            parent.spawn((
                Text::new(format!("{:?}", card.card_type)),
                TextFont {
                    font: asset_server.load("fonts/Arial Unicode.ttf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
            ));

            // 卡牌描述
            parent.spawn((
                Text::new(card.description.clone()),
                TextFont {
                    font: asset_server.load("fonts/Arial Unicode.ttf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
                TextLayout::new_with_justify(JustifyText::Center),
            ));
        });

    info!("【悬停面板】已创建卡牌详情面板: {}", card.name);
}

/// 创建遗物悬停详情面板
fn spawn_relic_hover_panel(commands: &mut Commands, relic: &Relic, asset_server: &AssetServer, mouse_pos: &MousePosition) {
    let rarity_color = relic.rarity.color();
    let text_color = relic.rarity.text_color();

    // 计算面板位置（与卡牌相同逻辑）
    const PANEL_WIDTH: f32 = 300.0;
    const OFFSET_X: f32 = 20.0;
    const OFFSET_Y: f32 = 20.0;
    const WINDOW_WIDTH: f32 = 1280.0;
    const WINDOW_HEIGHT: f32 = 720.0;

    let mut x = mouse_pos.x + OFFSET_X;
    let mut y = mouse_pos.y + OFFSET_Y;

    if x + PANEL_WIDTH > WINDOW_WIDTH {
        x = mouse_pos.x - PANEL_WIDTH - OFFSET_X;
    }

    if y + 200.0 > WINDOW_HEIGHT {
        y = mouse_pos.y - 200.0 - OFFSET_Y;
    }

    let (position_left, position_right) = if x + PANEL_WIDTH > WINDOW_WIDTH {
        (None, Some(Val::Px(WINDOW_WIDTH - x)))
    } else {
        (Some(Val::Px(x)), None)
    };

    let (position_top, position_bottom) = if y + 200.0 > WINDOW_HEIGHT {
        (Some(Val::Px(WINDOW_HEIGHT - y)), None)
    } else {
        (Some(Val::Px(y)), None)
    };

    let mut node = Node {
        position_type: PositionType::Absolute,
        width: Val::Px(PANEL_WIDTH),
        height: Val::Auto,
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(Val::Px(15.0)),
        row_gap: Val::Px(8.0),
        border: UiRect::all(Val::Px(2.0)),
        ..default()
    };

    if let Some(left) = position_left {
        node.left = left;
    }
    if let Some(right) = position_right {
        node.right = right;
    }
    if let Some(top) = position_top {
        node.top = top;
    }
    if let Some(bottom) = position_bottom {
        node.bottom = bottom;
    }

    commands
        .spawn((
            node,
            BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.95)),
            BorderColor(rarity_color),
            RelicHoverPanelMarker,
        ))
        .with_children(|parent| {
            // 稀有度标签
            parent.spawn((
                Text::new(format!("{:?}", relic.rarity)),
                TextFont {
                    font: asset_server.load("fonts/Arial Unicode.ttf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(rarity_color),
            ));

            // 遗物名称
            parent.spawn((
                Text::new(relic.name.clone()),
                TextFont {
                    font: asset_server.load("fonts/Arial Unicode.ttf"),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(text_color),
                TextLayout::new_with_justify(JustifyText::Center),
            ));

            // 遗物描述
            parent.spawn((
                Text::new(relic.description.clone()),
                TextFont {
                    font: asset_server.load("fonts/Arial Unicode.ttf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
                TextLayout::new_with_justify(JustifyText::Center),
            ));
        });

    info!("【悬停面板】已创建遗物详情面板: {}", relic.name);
}
