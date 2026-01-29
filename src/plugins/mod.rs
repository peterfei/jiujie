//! 游戏插件定义
pub mod hand_ui_v2;
use bevy::prelude::*;
use crate::states::GameState;
use crate::components::background_music::{BgmType, PlayBgmEvent, StopBgmEvent};

use crate::components::{
    Player, Enemy, EnemyType, EnemyIntent, Card, CardType, CardEffect, CardRarity, Hand, DrawPile, DiscardPile,
    CombatState, TurnPhase, NodeType, Realm, Cultivation, MapNode, MapProgress, PlayerDeck, DeckConfig, CardPool,
    MapUiRoot, MapNodeButton, RippleEffect, // 新增导入
    CharacterType, EnemyAttackEvent,
    SpriteMarker, ParticleMarker, EmitterMarker, EffectType, SpawnEffectEvent,
    PlayerHpBufferMarker, EnemyHpBarMarker, EnemyHpBufferMarker,

    IntentIconMarker,

    ScreenWarning,

    PlayerHpBarMarker,

    PlayerUiMarker,

    ScreenEffectEvent, ScreenEffectMarker, VictoryEvent, EnemyDeathAnimation,


    EnemySpriteMarker, VictoryDelay, RelicCollection, Relic, RelicId,
    EnemyActionQueue, RelicObtainedEvent, RelicTriggeredEvent, HeavenlyStrikeCinematic,
    ParticleEmitter, PlaySfxEvent, SfxType, CardHoverPanelMarker, RelicHoverPanelMarker, DialogueLine,
    Particle, DamageNumber, DamageEffectEvent, BlockIconMarker, BlockText, StatusIndicator,
    EnemyHpText, EnemyIntentText, EnemyStatusUi, PlayerHpText, PlayerEnergyText, PlayerBlockText,
    TopBar, TopBarHpText, TopBarGoldText, EnergyOrb, EndTurnButton, HandArea, CombatUiRoot,
    StatusEffectEvent, Environment, // 补全导入
};
use crate::components::sprite::{CharacterAssets, Rotating, CharacterAnimationEvent, PlayerSpriteMarker, MagicSealMarker, CharacterSprite};
use crate::systems::sprite::{spawn_character_sprite};

/// 核心游戏插件
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GameState>();
        // 应用启动时设置2D相机（用于渲染UI）
        app.add_systems(Startup, (setup_camera, init_character_assets));
        // 初始化胜利延迟计时器
        app.insert_resource(VictoryDelay::new(1.5)); // 延迟1.5秒让粒子特效播放

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
pub fn init_player(mut commands: Commands, player_data: Option<Res<Player>>) {
    // 关键修复：在闭包外部提取数据，避免生命周期冲突
    let initial_player = player_data.map(|p| p.clone());

    // 使用 Deferred 视图访问 World 进行立即 spawn
    commands.queue(move |world: &mut World| {
        let mut player_query = world.query_filtered::<Entity, With<Player>>();
        let player_entity = player_query.iter(world).next();

        if player_entity.is_none() {
            let player = initial_player.unwrap_or_default();
            
            info!("【持久化】初始化修士: HP={}/{}, 灵石={}", player.hp, player.max_hp, player.gold);
            world.spawn((
                player,
                crate::components::Cultivation::new(),
            ));
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
        app.init_resource::<EnemyActionQueue>();
        app.init_resource::<crate::components::combat::HeavenlyStrikeCinematic>();


        // 在进入MainMenu状态时设置主菜单
        app.add_systems(OnEnter(GameState::MainMenu), setup_main_menu);
        // 在退出MainMenu状态时清理主菜单
        app.add_systems(OnExit(GameState::MainMenu), cleanup_main_menu);
        // 处理按钮点击
        app.add_systems(Update, handle_button_clicks.run_if(in_state(GameState::MainMenu)));

        // 在进入Map状态时设置地图UI - 已迁移至 MapPlugin
        // app.add_systems(OnEnter(GameState::Map)...);
        // 全局牌组查看系统
        app.add_systems(Update, handle_deck_view_toggle);

        // 在进入Combat状态时设置战斗UI
        // 在进入Combat状态时设置战斗UI
        app.add_systems(OnEnter(GameState::Combat), (
            setup_combat_ui,
            setup_combat_environment, // 新增 3D 环境设置
        ));
        // 在进入Combat状态时重置玩家状态（能量、护甲等）
        app.add_systems(OnEnter(GameState::Combat), reset_player_on_combat_start);
        // 在进入Combat状态时抽牌（必须在 setup_combat_ui 之后执行）
        app.add_systems(OnEnter(GameState::Combat), draw_cards_on_combat_start.after(setup_combat_ui));
        // 在退出Combat状态时清理战斗UI
        app.add_systems(OnExit(GameState::Combat), cleanup_combat_ui);
        // 处理战斗界面按钮点击
        app.add_systems(Update, handle_combat_button_clicks.run_if(in_state(GameState::Combat)));
        // 更新战斗UI显示
        app.add_systems(Update, update_combat_ui.run_if(in_state(GameState::Combat)));
        // 回合开始时抽牌
        app.add_systems(Update, draw_cards_on_turn_start.run_if(in_state(GameState::Combat)));
        // 处理手牌中的诅咒效果
        app.add_systems(Update, handle_curse_effects.after(draw_cards_on_turn_start).run_if(in_state(GameState::Combat)));
        // 敌人队列处理系统
        app.add_systems(Update, process_enemy_turn_queue.run_if(in_state(GameState::Combat)));
        // 更新手牌UI
        app.add_systems(Update, hand_ui_v2::update_hand_ui_v2.run_if(in_state(GameState::Combat)));
        // 处理手牌卡片交互（弹起、放大、悬停效果）
        app.add_systems(Update, handle_hand_card_hover.run_if(in_state(GameState::Combat)));
        // 处理出牌
        app.add_systems(Update, handle_card_play.run_if(in_state(GameState::Combat)));
        // 检查战斗结束
        app.add_systems(Update, check_combat_end.run_if(in_state(GameState::Combat)));
        // 处理胜利延迟计时器
        app.add_systems(Update, update_victory_delay.run_if(in_state(GameState::Combat)));
        // 处理天象演出系统
        app.add_systems(Update, process_heavenly_strike_cinematic.run_if(in_state(GameState::Combat)));
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
            crate::systems::screen_effect::ScreenEffectPlugin,
            crate::systems::sprite::SpritePlugin,
            crate::systems::particle::ParticlePlugin, // 补齐粒子插件
            crate::systems::ui::UiPlugin,
            crate::systems::map::MapPlugin,
            crate::systems::background_music::BackgroundMusicPlugin, // 背景音乐插件
            crate::systems::audio::SfxPlugin, // 音效插件
        ))
        .init_state::<GameState>()
        .init_resource::<Player>() // 初始化玩家全局资源
        .insert_resource(VictoryDelay::new(1.5))
        .init_resource::<PlayerDeck>() // 初始化玩家持久化牌组
        .init_resource::<RelicCollection>() // 初始化遗物背包
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
        // 背景音乐触发系统
        .add_systems(OnEnter(GameState::MainMenu), trigger_bgm_main_menu)
        .add_systems(OnEnter(GameState::Prologue), trigger_bgm_map_exploration)
        .add_systems(OnEnter(GameState::Map), trigger_bgm_map_exploration)
        .add_systems(OnEnter(GameState::Combat), trigger_bgm_combat)
        .add_systems(OnEnter(GameState::Shop), trigger_bgm_shop)
        .add_systems(OnEnter(GameState::Rest), trigger_bgm_rest)
        .add_systems(OnEnter(GameState::Reward), trigger_bgm_victory)
        .add_systems(OnEnter(GameState::Tribulation), trigger_bgm_tribulation)
        .add_systems(OnEnter(GameState::Event), trigger_bgm_map_exploration)
        .add_systems(OnEnter(GameState::GameOver), stop_bgm)
        // 其他系统
        .add_systems(OnEnter(GameState::Prologue), setup_prologue)
        .add_systems(Update, update_prologue.run_if(in_state(GameState::Prologue)))
        .add_systems(OnExit(GameState::Prologue), cleanup_prologue)
        .add_systems(OnEnter(GameState::Tribulation), setup_tribulation)
        .add_systems(OnEnter(GameState::Event), setup_event_ui)
        .add_systems(Update, (
            update_tribulation.run_if(in_state(GameState::Tribulation)),
            handle_event_choices.run_if(in_state(GameState::Event)),
        ))
        .add_systems(OnExit(GameState::Tribulation), teardown_tribulation);
    }
}

// ============================================================================
// 背景音乐触发系统
// ============================================================================

/// 背景音乐触发系统 - 主菜单
fn trigger_bgm_main_menu(mut bgm_events: EventWriter<PlayBgmEvent>) {
    bgm_events.send(PlayBgmEvent::new(BgmType::MainMenu));
    info!("【背景音乐】触发播放: {}", BgmType::MainMenu.chinese_name());
}

/// 背景音乐触发系统 - 地图探索
fn trigger_bgm_map_exploration(mut bgm_events: EventWriter<PlayBgmEvent>) {
    bgm_events.send(PlayBgmEvent::new(BgmType::MapExploration));
    info!("【背景音乐】触发播放: {}", BgmType::MapExploration.chinese_name());
}

/// 背景音乐触发系统 - 战斗（根据敌人类型选择）
fn trigger_bgm_combat(
    mut bgm_events: EventWriter<PlayBgmEvent>,
    enemies: Query<&Enemy>,
) {
    // 检查是否有Boss（GreatDemon类型或高血量敌人）
    let has_boss = enemies.iter().any(|e| {
        matches!(e.enemy_type, EnemyType::GreatDemon) || e.max_hp >= 100
    });

    let bgm_type = if has_boss {
        BgmType::BossBattle
    } else {
        BgmType::NormalBattle
    };

    bgm_events.send(PlayBgmEvent::new(bgm_type));
    info!("【背景音乐】触发播放: {} ({})",
        bgm_type.chinese_name(),
        if has_boss { "Boss战" } else { "普通战斗" }
    );
}

/// 背景音乐触发系统 - 商店
fn trigger_bgm_shop(mut bgm_events: EventWriter<PlayBgmEvent>) {
    bgm_events.send(PlayBgmEvent::new(BgmType::Shop));
    info!("【背景音乐】触发播放: {}", BgmType::Shop.chinese_name());
}

/// 背景音乐触发系统 - 休息
fn trigger_bgm_rest(mut bgm_events: EventWriter<PlayBgmEvent>) {
    bgm_events.send(PlayBgmEvent::new(BgmType::Rest));
    info!("【背景音乐】触发播放: {}", BgmType::Rest.chinese_name());
}

/// 背景音乐触发系统 - 胜利
fn trigger_bgm_victory(mut bgm_events: EventWriter<PlayBgmEvent>) {
    bgm_events.send(PlayBgmEvent::new(BgmType::Victory));
    info!("【背景音乐】触发播放: {}", BgmType::Victory.chinese_name());
}

/// 背景音乐触发系统 - 渡劫
fn trigger_bgm_tribulation(mut bgm_events: EventWriter<PlayBgmEvent>) {
    bgm_events.send(PlayBgmEvent::new(BgmType::Tribulation));
    info!("【背景音乐】触发播放: {}", BgmType::Tribulation.chinese_name());
}

/// 停止背景音乐
fn stop_bgm(mut bgm_events: EventWriter<StopBgmEvent>) {
    bgm_events.send(StopBgmEvent::new());
    info!("【背景音乐】停止播放");
}

// ============================================================================
// 核心系统
// ============================================================================

use bevy::core_pipeline::tonemapping::Tonemapping;

/// 相机设置
fn setup_camera(mut commands: Commands) {
    // 2D 相机 (用于 UI 渲染)
    commands.spawn((
        Camera2d,
        Camera {
            order: 10, // 确保 UI 永远在最上层
            clear_color: ClearColorConfig::None,
            ..default()
        },
    ));
    
    // 3. 3D 相机 (大作级视角：低仰角 + 深度雾)
    commands.spawn((
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgba(0.01, 0.005, 0.02, 1.0)),
            order: 0, 
            ..default()
        },
        Tonemapping::None,
        Transform::from_xyz(0.0, 4.5, 10.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
        DistanceFog {
            color: Color::srgba(0.01, 0.005, 0.02, 1.0), // 幽邃的紫色雾气
            falloff: FogFalloff::Linear {
                start: 5.0, 
                end: 25.0,
            },
            ..default()
        },
    ));
}

/// 初始化角色资源
fn init_character_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(CharacterAssets::load(&asset_server));
    info!("CharacterAssets 已加载");
}

/// 设置战斗环境（3D 地板和灯光）
fn setup_combat_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let floor_texture = asset_server.load("textures/magic_circle.png");

    // 1. 聚灵法阵地板 (3D 平面) - 强化发光
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 1.0, 1.0, 0.95),
            base_color_texture: Some(floor_texture.clone()),
            emissive: LinearRgba::new(0.0, 1.2, 0.5, 1.0), // 初始更亮
            perceptual_roughness: 0.8,
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_xyz(0.0, -1.5, 0.0), 
        CombatUiRoot, 
        Rotating { speed: 0.05 },
        MagicSealMarker, // 标记用于脉动效果
    ));

    // 2. 主光源 (模拟太阳/大阵光芒)
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 4000.0,
            ..default()
        },
        Transform::from_xyz(5.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        CombatUiRoot,
    ));

    // 3. 局部氛围光源 - 增加电影感
    // 玩家位光
    commands.spawn((
        PointLight {
            intensity: 150000.0,
            range: 15.0,
            color: Color::srgb(0.3, 0.5, 1.0),
            ..default()
        },
        Transform::from_xyz(-4.5, 3.0, 2.0),
        CombatUiRoot,
    ));

    // 敌人位光
    commands.spawn((
        PointLight {
            intensity: 120000.0,
            range: 12.0,
            color: Color::srgb(1.0, 0.2, 0.3),
            ..default()
        },
        Transform::from_xyz(4.5, 3.0, 2.0),
        CombatUiRoot,
    ));
    
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.1, 0.1, 0.1), 
        brightness: 20.0, // 降低到极低，主要靠点光源和自发光
    });

    // 4. 环境灵气粒子
    commands.spawn((
        ParticleEmitter::new(5.0, EffectType::AmbientSpirit.config()).with_type(EffectType::AmbientSpirit),
        Transform::from_xyz(0.0, 0.0, 0.0),
        CombatUiRoot,
    ));
}

// ============================================================================
// 主菜单系统
// ============================================================================

/// 设置主菜单UI
/// 主菜单根节点标记
#[derive(Component)]
struct MainMenuRoot;

fn setup_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let chinese_font: Handle<Font> = asset_server.load("fonts/Arial Unicode.ttf");
    let logo_handle: Handle<Image> = asset_server.load("textures/logo.png");
    let has_save = crate::resources::save::GameStateSave::exists();

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
            MainMenuRoot, // 添加标记
        ))
        .with_children(|parent| {
            // 背景 Logo
            parent.spawn((
                ImageNode::new(logo_handle),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ZIndex(-1),
            ));

            // 按钮容器
            parent.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(25.0),
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                bottom: Val::Percent(15.0),
                ..default()
            }).with_children(|btn_parent| {
                if has_save {
                    // 继续修行按钮
                    btn_parent.spawn((
                        Node {
                            width: Val::Px(240.0), height: Val::Px(60.0),
                            justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)), ..default()
                        },
                        BorderColor(Color::srgba(0.4, 1.0, 0.4, 0.5)),
                        BackgroundColor(Color::srgba(0.1, 0.3, 0.1, 0.9)),
                        Button,
                        ContinueGameButton,
                    )).with_children(|p| {
                        p.spawn((Text::new("继 续 修 行"), TextFont { font: chinese_font.clone(), font_size: 32.0, ..default() }, TextColor(Color::WHITE)));
                    });
                }

                // 开始修行按钮 (或重塑道基)
                btn_parent.spawn((
                    Node {
                        width: Val::Px(240.0), height: Val::Px(60.0),
                        justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)), ..default()
                    },
                    BorderColor(Color::srgba(1.0, 1.0, 1.0, 0.3)),
                    BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.85)),
                    Button,
                    StartGameButton,
                )).with_children(|p| {
                    let btn_text = if has_save { "重 塑 道 基" } else { "开 始 修 行" };
                    p.spawn((Text::new(btn_text), TextFont { font: chinese_font.clone(), font_size: 32.0, ..default() }, TextColor(Color::WHITE)));
                });
            });
        });
}

/// 清理主菜单UI
fn cleanup_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// cleanup_map_ui 已迁移至 src/systems/map.rs

// BreakthroughButtonMarker 已迁移至 src/components/map.rs

// UI 设置函数已迁移至 src/systems/map.rs

// ============================================================================
// 组件标记
// ============================================================================

#[derive(Component)]
struct StartGameButton;

#[derive(Component)]
struct ContinueGameButton;

#[derive(Component)]
struct QuitGameButton;

// 地图组件已迁移至 src/components/map.rs
// 视觉效果组件已迁移至 src/components/map.rs

// ============================================================================
// 按钮交互系统
// ============================================================================

/// 处理主菜单按钮点击
fn handle_button_clicks(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut button_queries: ParamSet<(
        Query<&Interaction, (Changed<Interaction>, With<StartGameButton>)>,
        Query<&Interaction, (Changed<Interaction>, With<ContinueGameButton>)>,
        Query<&Interaction, (Changed<Interaction>, With<QuitGameButton>)>,
    )>,
    mut exit: EventWriter<AppExit>,
) {
    // 1. 开始修行（重塑道基）
    for interaction in button_queries.p0().iter() {
        if matches!(interaction, Interaction::Pressed) {
            info!("【主菜单】开始新修行，删除旧存档");
            crate::resources::save::GameStateSave::delete_save();
            next_state.set(GameState::Prologue);
            return;
        }
    }

    // 2. 继续修行（读档）
    for interaction in button_queries.p1().iter() {
        if matches!(interaction, Interaction::Pressed) {
            info!("【主菜单】继续修行，尝试加载存档");
            match crate::resources::save::GameStateSave::load_from_disk() {
                Ok(save) => {
                    // 使用 commands.queue 确保资源立即插入，防止 setup_map_ui 读取不到
                    commands.queue(move |world: &mut World| {
                        world.insert_resource(save.player.clone());
                        world.insert_resource(save.cultivation.clone());
                        world.insert_resource(PlayerDeck { cards: save.deck.clone() });
                        world.insert_resource(RelicCollection { relic: save.relics.clone() });
                        world.insert_resource(MapProgress::from_save(
                            save.map_nodes.clone(),
                            save.current_map_node_id,
                            save.current_map_layer,
                        ));
                        
                        // 在同一个 queue 中设置状态，确保顺序
                        if let Some(mut next_state) = world.get_resource_mut::<NextState<GameState>>() {
                            next_state.set(GameState::Map);
                        }
                    });
                    info!("【存档系统】读档操作已排队，即将进入大地图");
                }
                Err(e) => {
                    error!("【存档系统】加载失败: {}", e);
                }
            }
            return;
        }
    }

    // 3. 离开尘世
    for interaction in button_queries.p2().iter() {
        if matches!(interaction, Interaction::Pressed) {
            exit.send(AppExit::Success);
        }
    }
}

/// 处理地图界面按钮点击
fn handle_map_button_clicks(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut map_progress: ResMut<MapProgress>,
    player_query: Query<(&Player, &crate::components::Cultivation)>,
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
                    RippleEffect::new(size * 1.5, 0.6), // 波纹扩散到节点大小的1.5倍，持续0.6秒
                    ZIndex(-1), // 在节点下方
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

            // [同步清理加固] 在切换状态前手动清理地图UI，防止遮挡
            // 注意：虽然 OnExit 也有，但这里手动触发可以消除一帧的延迟
            // 我们通过查询 MapUiRoot 实体进行清理
            // 由于系统无法在这里直接 Query，我们依赖 OnExit 逻辑，但确保状态切换是最高优先级
            
            // --- 执行自动存档 ---
            if let Ok((player, cultivation)) = player_query.get_single() {
                let save = crate::resources::save::GameStateSave {
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

// ============================================================================
// 地图系统
// ============================================================================

// 地图 UI 设置函数已迁移至 src/systems/map.rs

// ============================================================================
// 战斗系统
// ============================================================================

// ============================================================================
// 战斗UI初始化
// ============================================================================

#[derive(Component)]
struct TurnText;

#[derive(Component)]
pub struct DrawPileText;

#[derive(Component)]
pub struct DiscardPileText;

#[derive(Component)]
pub struct HandCountText;

#[derive(Component)]
struct ReturnToMapButton;

#[derive(Component)]
pub struct HandCard {
    pub card_id: u32,
    pub base_bottom: f32,
    pub base_rotation: f32,
    pub index: usize,
}

#[derive(Component)]
struct RewardUiRoot;

#[derive(Component)]
struct RewardCardButton {
    card_id: u32,
}

#[derive(Component)]
struct RewardRelicButton {
    relic_id: RelicId,
}

#[derive(Component)]
struct RestartButton;

#[derive(Component)]
struct BackToMenuButton;

#[derive(Component)]
struct BackToMapButton;

#[derive(Component)]
pub struct EventUiRoot; // 设为 pub

pub fn setup_event_ui_wrapper(commands: Commands, asset_server: Res<AssetServer>) {
    setup_event_ui(commands, asset_server);
}

#[derive(Component)]
enum EventChoiceButton {
    GainGold(i32),
    Heal(i32),
    Leave,
}

/// 设置机缘事件界面
fn setup_event_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    info!("【机缘】进入奇遇事件");
    let font = asset_server.load("fonts/Arial Unicode.ttf");

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(30.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.05, 0.1, 0.9)),
        EventUiRoot,
    )).with_children(|parent| {
        // 标题
        parent.spawn((
            Text::new("古 修 遗 迹"),
            TextFont { font: font.clone(), font_size: 48.0, ..default() },
            TextColor(Color::srgb(0.4, 0.8, 1.0)),
        ));

        // 描述
        parent.spawn((
            Text::new("你在一处断崖下发现了一尊古老的石像，石像手中握着一颗微弱发光的灵石，而基座上似乎刻着某种愈合咒文。"),
            Node { max_width: Val::Px(600.0), ..default() },
            TextFont { font: font.clone(), font_size: 20.0, ..default() },
            TextColor(Color::WHITE),
            TextLayout::new_with_justify(JustifyText::Center),
        ));

        // 选项区
        parent.spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(15.0),
            ..default()
        }).with_children(|choices| {
            // 选项 1: 拿走灵石
            create_event_button(choices, "取走灵石 (+50 灵石)", EventChoiceButton::GainGold(50), font.clone());
            // 选项 2: 虔诚祈祷
            create_event_button(choices, "虔诚祈祷 (回复 20 HP)", EventChoiceButton::Heal(20), font.clone());
            // 选项 3: 离去
            create_event_button(choices, "因果莫测，径直离去", EventChoiceButton::Leave, font.clone());
        });
    });
}

fn create_event_button(parent: &mut ChildBuilder, label: &str, choice: EventChoiceButton, font: Handle<Font>) {
    parent.spawn((
        Button,
        Node {
            width: Val::Px(400.0),
            height: Val::Px(50.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.2, 0.2, 0.3, 0.8)),
        BorderRadius::all(Val::Px(8.0)),
        choice,
    )).with_children(|p| {
        p.spawn((
            Text::new(label),
            TextFont { font, font_size: 18.0, ..default() },
            TextColor(Color::WHITE),
        ));
    });
}

/// 处理机缘事件选择
fn handle_event_choices(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut player_query: Query<(&mut Player, &crate::components::Cultivation)>,
    mut map_progress: ResMut<MapProgress>,
    deck: Res<PlayerDeck>,
    relics: Res<RelicCollection>,
    button_query: Query<(&Interaction, &EventChoiceButton), Changed<Interaction>>,
    ui_query: Query<Entity, With<EventUiRoot>>,
) {
    for (interaction, choice) in button_query.iter() {
        if *interaction == Interaction::Pressed {
            let mut player_modified = false;
            if let Ok((mut player, _)) = player_query.get_single_mut() {
                match choice {
                    EventChoiceButton::GainGold(amt) => {
                        player.gold += *amt;
                        info!("【机缘】获得灵石 {}", amt);
                        player_modified = true;
                    }
                    EventChoiceButton::Heal(amt) => {
                        player.hp = (player.hp + *amt).min(player.max_hp);
                        info!("【机缘】回复生命 {}", amt);
                        player_modified = true;
                    }
                    EventChoiceButton::Leave => {
                        info!("【机缘】悄然离去");
                    }
                }
            }

            // 完成当前节点，防止重复进入
            map_progress.complete_current_node();
            info!("【机缘】事件节点已完成，下一层已解锁");

            // --- [优化] 移除同步阻塞存档，依赖内存状态传递和状态机自动处理 ---
            // 避免在此处进行 IO 操作，防止与状态切换产生竞争

            // 清理并退出
            for e in ui_query.iter() {
                commands.entity(e).despawn_recursive();
            }
            next_state.set(GameState::Map);
        }
    }
}

/// 设置战斗UI
fn setup_combat_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    character_assets: Res<CharacterAssets>,
    player_deck: Res<PlayerDeck>,
    relic_collection: Res<RelicCollection>,
    enemy_query: Query<(Entity, &Enemy)>,
    mut victory_delay: ResMut<VictoryDelay>,
    player_query: Query<(Entity, &Player, &crate::components::Cultivation)>,
    map_progress: Res<MapProgress>,
    existing_ui: Query<Entity, With<CombatUiRoot>>,
    map_ui: Query<Entity, With<MapUiRoot>>, // 新增：清理地图残留
) {
    info!("【战斗】进入战场，众妖环伺");
    
    // 防御性清理：确保没有任何残留的 UI (包括战斗和地图)
    for entity in existing_ui.iter() { commands.entity(entity).despawn_recursive(); }
    for entity in map_ui.iter() { commands.entity(entity).despawn_recursive(); }
    
    if victory_delay.active { victory_delay.active = false; victory_delay.elapsed = 0.0; }
    let chinese_font: Handle<Font> = asset_server.load("fonts/Arial Unicode.ttf");

    // 检查当前节点是否为 Boss
    let is_boss_node = map_progress.is_at_boss();

    // 创建根容器
        let root_entity = commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            ZIndex(200), // 全局最高层级
            PickingBehavior::IGNORE, CombatUiRoot,
        )).id();

    // --- 多敌人生成 ---
    if enemy_query.is_empty() {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        let current_layer = map_progress.current_layer;
        let hp_scaling = 1.0 + (current_layer as f32 * 0.15);
        let extra_strength = (current_layer / 3) as i32;

        // 如果是 Boss 节点，固定生成 1 个 BOSS；否则随机生成 1~3 个小怪
        let num_enemies = if is_boss_node { 1 } else { rng.gen_range(1..=3) };

        for i in 0..num_enemies {
            let enemy_id = i as u32;
            let (name, base_hp, e_type) = if is_boss_node {
                ("幽冥白虎", 150, EnemyType::GreatDemon)
            } else {
                match rng.gen_range(0..3) {
                    0 => ("嗜血妖狼", 30, EnemyType::DemonicWolf),
                    1 => ("剧毒蛛", 20, EnemyType::PoisonSpider),
                    _ => ("怨灵", 40, EnemyType::CursedSpirit),
                }
            };

            let scaled_hp = (base_hp as f32 * hp_scaling) as i32;
            let mut enemy = Enemy::with_type(enemy_id, name, scaled_hp, e_type);
            
            // 随着层级提升增加初始力量和护甲
            enemy.strength = extra_strength;
            enemy.block = (current_layer / 2) as i32 * 2; // 每2层增加2点初始护甲
            
            // 动态调整 AI 动作强度范围
            enemy.ai_pattern.damage_range.0 += extra_strength;
            enemy.ai_pattern.damage_range.1 += extra_strength;
            enemy.ai_pattern.block_range.0 += (current_layer / 2) as i32;
            enemy.ai_pattern.block_range.1 += (current_layer / 2) as i32;

            if is_boss_node {
                enemy.strength += 2; // Boss 额外强度
                enemy.block += 5;    // Boss 额外护甲
                enemy.ai_pattern.damage_range.0 += 5;
                enemy.ai_pattern.damage_range.1 += 5;
            }

            let x_world = 250.0 + (i as f32 - (num_enemies as f32 - 1.0) / 2.0) * 220.0;
            let enemy_entity = commands.spawn(enemy).id();

            // 根据妖兽类型选择渲染类型与尺寸 (大作级体型压制)
            let (char_type, size) = match e_type {
                EnemyType::DemonicWolf => (CharacterType::DemonicWolf, Vec2::new(100.0, 120.0)),
                EnemyType::PoisonSpider => (CharacterType::PoisonSpider, Vec2::new(100.0, 120.0)),
                EnemyType::CursedSpirit => (CharacterType::CursedSpirit, Vec2::new(120.0, 160.0)),
                EnemyType::GreatDemon => (CharacterType::GreatDemon, Vec2::new(180.0, 240.0)),
            };
            spawn_character_sprite(&mut commands, &character_assets, char_type, Vec3::new(x_world, 50.0, 10.0), size, Some(enemy_id));

            let ui_left = 640.0 + x_world - 80.0;
            commands.entity(root_entity).with_children(|root| {
                root.spawn((
                    Node { position_type: PositionType::Absolute, left: Val::Px(ui_left), bottom: Val::Px(480.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, ..default() },
                    EnemyStatusUi { owner: enemy_entity },
                    ZIndex(150), // 提升层级，确保在特效上方
                )).with_children(|p| {
                    p.spawn((Text::new(name), TextFont { font: chinese_font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));
                    
                    // HP & Block 栏 (三层血条重构)
                    p.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(8.0),
                        ..default()
                    }).with_children(|row| {
                        row.spawn((
                            Text::new(format!("{}/{}", scaled_hp, scaled_hp)),
                            TextFont { font: chinese_font.clone(), font_size: 14.0, ..default() },
                            TextColor(Color::WHITE),
                            EnemyHpText { owner: enemy_entity },
                        ));

                        // 血条主体
                        row.spawn((
                            Node {
                                width: Val::Px(120.0),
                                height: Val::Px(12.0),
                                border: UiRect::all(Val::Px(1.5)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                            BorderColor(Color::srgb(0.4, 0.4, 0.4)),
                        )).with_children(|bar| {
                            bar.spawn((
                                Node { position_type: PositionType::Absolute, left: Val::Px(0.0), top: Val::Px(0.0), width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
                                BackgroundColor(Color::srgb(0.6, 0.2, 0.2)),
                                EnemyHpBufferMarker { owner: enemy_entity },
                            ));
                            bar.spawn((
                                Node { position_type: PositionType::Absolute, left: Val::Px(0.0), top: Val::Px(0.0), width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
                                BackgroundColor(Color::srgb(0.9, 0.1, 0.1)),
                                EnemyHpBarMarker { owner: enemy_entity },
                            ));
                        });
                        
                        // 护甲图标容器 (使用 Display 控制)
                        row.spawn((
                            Node {
                                display: Display::None, // 初始隐藏
                                width: Val::Px(28.0), height: Val::Px(28.0),
                                justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                                margin: UiRect::left(Val::Px(-10.0)),
                                ..default()
                            },
                            ImageNode::new(asset_server.load("textures/cards/defense.png")).with_color(Color::srgb(0.4, 0.7, 1.0)), // 蓝色护盾图标
                            BlockIconMarker { owner: enemy_entity },
                        )).with_children(|shield| {
                            shield.spawn((
                                Text::new("0"),
                                TextFont { font: chinese_font.clone(), font_size: 14.0, ..default() },
                                TextColor(Color::WHITE),
                                BlockText,
                            ));
                        });
                    });

                    // 状态显示行
                    p.spawn((
                        Text::new(""),
                        TextFont { font: chinese_font.clone(), font_size: 12.0, ..default() },
                        TextColor(Color::srgb(0.7, 0.4, 1.0)), // 紫色，醒目且符合状态色彩
                        StatusIndicator { owner: enemy_entity },
                    ));

                    // [大作级] 意图显示容器
                    p.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(4.0),
                            margin: UiRect::top(Val::Px(4.0)),
                            ..default()
                        },
                    )).with_children(|row| {
                        row.spawn((
                            Node { width: Val::Px(24.0), height: Val::Px(24.0), ..default() },
                            ImageNode::new(asset_server.load("textures/cards/attack.png")), // 默认攻击
                            IntentIconMarker { owner: enemy_entity },
                        ));
                        row.spawn((
                            Text::new(""),
                            TextFont { font: chinese_font.clone(), font_size: 16.0, ..default() },
                            TextColor(Color::srgb(1.0, 0.8, 0.4)),
                            EnemyIntentText { owner: enemy_entity }
                        ));
                    });
                });
            });
        }
    } else {
        info!("【战斗】检测到预设妖兽，跳过随机生成");
        for (enemy_entity, enemy) in enemy_query.iter() {
            let x_world = 250.0; // 简化位置
            let ui_left = 640.0 + x_world - 80.0;
            
            let char_type = match enemy.enemy_type {
                EnemyType::DemonicWolf => CharacterType::DemonicWolf,
                EnemyType::PoisonSpider => CharacterType::PoisonSpider,
                EnemyType::CursedSpirit => CharacterType::CursedSpirit,
                EnemyType::GreatDemon => CharacterType::GreatDemon,
            };
            spawn_character_sprite(&mut commands, &character_assets, char_type, Vec3::new(x_world, 50.0, 10.0), Vec2::new(100.0, 120.0), Some(enemy.id));

            commands.entity(root_entity).with_children(|root| {
                root.spawn((
                    Node { position_type: PositionType::Absolute, left: Val::Px(ui_left), bottom: Val::Px(480.0), flex_direction: FlexDirection::Column, align_items: AlignItems::Center, ..default() },
                    EnemyStatusUi { owner: enemy_entity },
                    ZIndex(150),
                )).with_children(|p| {
                    p.spawn((Text::new(enemy.name.clone()), TextFont { font: chinese_font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));
                    
                    // HP & Block 栏 (三层血条重构)
                    p.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(8.0),
                        ..default()
                    }).with_children(|row| {
                        row.spawn((
                            Text::new(format!("{}/{}", enemy.hp, enemy.max_hp)),
                            TextFont { font: chinese_font.clone(), font_size: 14.0, ..default() },
                            TextColor(Color::WHITE),
                            EnemyHpText { owner: enemy_entity },
                        ));

                        // 血条主体
                        row.spawn((
                            Node {
                                width: Val::Px(120.0),
                                height: Val::Px(12.0),
                                border: UiRect::all(Val::Px(1.5)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                            BorderColor(Color::srgb(0.4, 0.4, 0.4)),
                        )).with_children(|bar| {
                            bar.spawn((
                                Node { position_type: PositionType::Absolute, left: Val::Px(0.0), top: Val::Px(0.0), width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
                                BackgroundColor(Color::srgb(0.6, 0.2, 0.2)),
                                EnemyHpBufferMarker { owner: enemy_entity },
                            ));
                            bar.spawn((
                                Node { position_type: PositionType::Absolute, left: Val::Px(0.0), top: Val::Px(0.0), width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
                                BackgroundColor(Color::srgb(0.9, 0.1, 0.1)),
                                EnemyHpBarMarker { owner: enemy_entity },
                            ));
                        });
                        
                        // 护甲图标容器
                        row.spawn((
                            Node {
                                display: Display::None,
                                width: Val::Px(28.0), height: Val::Px(28.0),
                                justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                                margin: UiRect::left(Val::Px(-10.0)),
                                ..default()
                            },
                            ImageNode::new(asset_server.load("textures/cards/defense.png")).with_color(Color::srgb(0.4, 0.7, 1.0)),
                            BlockIconMarker { owner: enemy_entity },
                        )).with_children(|shield| {
                            shield.spawn((
                                Text::new("0"),
                                TextFont { font: chinese_font.clone(), font_size: 14.0, ..default() },
                                TextColor(Color::WHITE),
                                BlockText,
                            ));
                        });
                    });

                    // 状态显示行
                    p.spawn((
                        Text::new(""),
                        TextFont { font: chinese_font.clone(), font_size: 12.0, ..default() },
                        TextColor(Color::srgb(0.7, 0.4, 1.0)), // 紫色，醒目且符合状态色彩
                        StatusIndicator { owner: enemy_entity },
                    ));

                    // [大作级] 意图显示容器
                    p.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(4.0),
                            margin: UiRect::top(Val::Px(4.0)),
                            ..default()
                        },
                    )).with_children(|row| {
                        row.spawn((
                            Node { width: Val::Px(24.0), height: Val::Px(24.0), ..default() },
                            ImageNode::new(asset_server.load("textures/cards/attack.png")), // 默认攻击
                            IntentIconMarker { owner: enemy_entity },
                        ));
                        row.spawn((
                            Text::new(""),
                            TextFont { font: chinese_font.clone(), font_size: 16.0, ..default() },
                            TextColor(Color::srgb(1.0, 0.8, 0.4)),
                            EnemyIntentText { owner: enemy_entity }
                        ));
                    });
                });
            });
        }
    }

    spawn_character_sprite(&mut commands, &character_assets, CharacterType::Player, Vec3::new(-350.0, -80.0, 10.0), Vec2::new(100.0, 140.0), None);

    // --- 法宝 3D 视觉生成 ---
    for (i, relic) in relic_collection.relic.iter().enumerate() {
        // 大作级环绕布局：半径 1.8 米，高度 1.5 米
        let angle = i as f32 * 1.2; // 角度间隔
        let x_offset = angle.cos() * 1.8;
        let z_offset = angle.sin() * 0.8;
        let base_pos = Vec3::new(-3.5 + x_offset, 1.5, 1.0 + z_offset); // Z 轴提前到 1.0

        commands.spawn((
            crate::components::sprite::RelicVisualMarker { 
                relic_id: relic.id.clone(),
                base_y: base_pos.y,
            },
            SpriteMarker,
            CombatUiRoot,
            Rotating { speed: 1.5 }, // 旋转快一点更灵动
            CharacterSprite::new(asset_server.load("textures/relics/default.png"), Vec2::new(45.0, 45.0)),
            Transform::from_translation(base_pos),
        ));
    }
    commands.insert_resource(CombatState::default());
    
    // 准备全量牌组
    let deck_cards = player_deck.cards.clone();
    info!("【战斗】准备战斗牌组: {} 张", deck_cards.len());
    
    // 仅初始化空的堆栈，将抽牌权交给专门的 draw_cards 系统
    commands.spawn(DrawPile::new(deck_cards));
    commands.spawn(DiscardPile::new());
    commands.spawn(Hand::new(10));
    let player_data = player_query.get_single().ok();
    let player_entity = player_data.map(|(e, _, _)| e);

    commands.entity(root_entity).with_children(|root| {
        // --- [大作级] 屏幕预警层 (最底层) ---
        root.spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.8, 0.0, 0.0, 0.0)), // 初始透明
            Visibility::Hidden,
            ScreenWarning,
            ZIndex(-1), // 放在所有战斗 UI 之下
        ));

        root.spawn((
            Node { position_type: PositionType::Absolute, top: Val::Px(0.0), width: Val::Percent(100.0), height: Val::Px(45.0), flex_direction: FlexDirection::Row, align_items: AlignItems::Center, padding: UiRect::horizontal(Val::Px(20.0)), column_gap: Val::Px(30.0), ..default() },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)), TopBar,
        )).with_children(|bar| {
            if let Some((_, p, c)) = player_data {
                let r_name = match c.realm { crate::components::cultivation::Realm::QiRefining => "炼气期", crate::components::cultivation::Realm::FoundationEstablishment => "筑基期", crate::components::cultivation::Realm::GoldenCore => "金丹期", _ => "修仙者" };
                bar.spawn((Text::new(format!("境界: {}", r_name)), TextFont { font: chinese_font.clone(), font_size: 20.0, ..default() }, TextColor(Color::srgb(0.4, 1.0, 0.4))));
                bar.spawn((Text::new(format!("道行: {}/{}", p.hp, p.max_hp)), TextFont { font: chinese_font.clone(), font_size: 20.0, ..default() }, TextColor(Color::srgb(1.0, 0.4, 0.4)), TopBarHpText));
                bar.spawn((Text::new(format!("灵石: {}", p.gold)), TextFont { font: chinese_font.clone(), font_size: 20.0, ..default() }, TextColor(Color::srgb(1.0, 0.8, 0.2)), TopBarGoldText));
            }
        });
        root.spawn((
            Node { 
                position_type: PositionType::Absolute, 
                left: Val::Px(150.0), 
                bottom: Val::Px(280.0), 
                flex_direction: FlexDirection::Column, 
                align_items: AlignItems::Start, // 改为左对齐
                ..default() 
            },
            ZIndex(150),
        )).with_children(|p| {
            if let Some((_, player, _)) = player_data {
                // --- [大作级] 增强版血条容器 ---
                p.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(5.0),
                    ..default()
                }).with_children(|row| {
                    // A. 护甲图标 (半悬浮于血条左侧)
                    if let Some(entity) = player_entity {
                        row.spawn((
                            Node {
                                display: Display::None,
                                width: Val::Px(36.0), height: Val::Px(36.0),
                                justify_content: JustifyContent::Center, align_items: AlignItems::Center,
                                margin: UiRect::right(Val::Px(-15.0)), // 向右偏移覆盖血条边缘
                                ..default()
                            },
                            ImageNode::new(asset_server.load("textures/cards/defense.png")).with_color(Color::srgb(1.0, 0.6, 0.0)),
                            BlockIconMarker { owner: entity },
                            ZIndex(10), // 确保在血条之上
                        )).with_children(|shield| {
                            shield.spawn((
                                Text::new("0"),
                                TextFont { font: chinese_font.clone(), font_size: 18.0, ..default() },
                                TextColor(Color::WHITE),
                                BlockText,
                            ));
                        });
                    }

                    // B. 血条主体 (三层结构：深灰槽、亮红缓冲、鲜红当前)
                    row.spawn((
                        Node {
                            width: Val::Px(200.0),
                            height: Val::Px(18.0),
                            border: UiRect::all(Val::Px(1.5)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.15, 0.15, 0.15)), // 深灰色槽，消除“黑影”感
                        BorderColor(Color::srgb(0.4, 0.4, 0.4)), // 灰色边框
                    )).with_children(|bar| {
                        let hp_percent = (player.hp as f32 / player.max_hp as f32) * 100.0;
                        
                        // 1. 缓冲层
                        bar.spawn((
                            Node { position_type: PositionType::Absolute, left: Val::Px(0.0), top: Val::Px(0.0), width: Val::Percent(hp_percent), height: Val::Percent(100.0), ..default() },
                            BackgroundColor(Color::srgb(0.6, 0.2, 0.2)), // 缓冲暗红
                            PlayerHpBufferMarker,
                        ));

                        // 2. 当前血量
                        bar.spawn((
                            Node { position_type: PositionType::Absolute, left: Val::Px(0.0), top: Val::Px(0.0), width: Val::Percent(hp_percent), height: Val::Percent(100.0), ..default() },
                            BackgroundColor(Color::srgb(0.9, 0.1, 0.1)), // 鲜亮红
                            PlayerHpBarMarker,
                        ));
                    });

                    row.spawn((Text::new(format!("{}/{}", player.hp, player.max_hp)), TextFont { font: chinese_font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE), PlayerHpText));
                });

                // 玩家状态显示行
                p.spawn((
                    Text::new(""),
                    TextFont { font: chinese_font.clone(), font_size: 14.0, ..default() },
                    TextColor(Color::srgb(0.7, 0.4, 1.0)),
                    StatusIndicator { owner: player_entity.unwrap_or(Entity::PLACEHOLDER) },
                )).insert(PlayerUiMarker);
            }
        });
        root.spawn((
            Node { position_type: PositionType::Absolute, left: Val::Px(100.0), bottom: Val::Px(120.0), width: Val::Px(90.0), height: Val::Px(90.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
            ZIndex(10), BackgroundColor(Color::srgba(0.1, 0.2, 0.5, 0.9)), EnergyOrb
        )).with_children(|orb| {
            if let Some((_, p, _)) = player_data {
                orb.spawn((Text::new(format!("{}/{}", p.energy, p.max_energy)), TextFont { font: chinese_font.clone(), font_size: 32.0, ..default() }, TextColor(Color::WHITE), PlayerEnergyText));
            }
        });
        root.spawn((
            Button, Node { position_type: PositionType::Absolute, right: Val::Px(100.0), bottom: Val::Px(140.0), width: Val::Px(160.0), height: Val::Px(50.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() },
            BackgroundColor(Color::srgb(0.2, 0.4, 0.2)), BorderColor(Color::BLACK), EndTurnButton
        )).with_children(|btn| { btn.spawn((Text::new("结束回合"), TextFont { font: chinese_font.clone(), font_size: 24.0, ..default() }, TextColor(Color::WHITE))); });
        root.spawn((
            Node { 
                position_type: PositionType::Absolute, 
                left: Val::Percent(0.0), 
                bottom: Val::Px(0.0), 
                width: Val::Percent(100.0), 
                height: Val::Px(250.0), 
                justify_content: JustifyContent::Center, 
                ..default() 
            }, 
            HandArea,
            ZIndex(50), // 确保在粒子(5)之上
        )).with_children(|parent| {
            parent.spawn((Text::new("手牌: 0/10"), TextFont { font: chinese_font.clone(), font_size: 18.0, ..default() }, TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)), Node { position_type: PositionType::Absolute, top: Val::Px(10.0), ..default() }, HandCountText));
        });
        root.spawn(Node { position_type: PositionType::Absolute, left: Val::Px(30.0), bottom: Val::Px(30.0), ..default() }).with_children(|p| {
            p.spawn((Text::new("剑冢: 0"), TextFont { font: chinese_font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE), DrawPileText));
        });
        root.spawn(Node { position_type: PositionType::Absolute, right: Val::Px(30.0), bottom: Val::Px(30.0), ..default() }).with_children(|p| {
            p.spawn((Text::new("归墟: 0"), TextFont { font: chinese_font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE), DiscardPileText));
        });
    });
}

/// 天象打击演出处理系统
pub fn process_heavenly_strike_cinematic(
    mut commands: Commands,
    time: Res<Time>,
    mut cinematic: ResMut<HeavenlyStrikeCinematic>,
    mut player_query: Query<(&mut Player, &crate::components::Cultivation)>,
    mut enemy_query: Query<&mut Enemy>,
    mut effect_events: EventWriter<SpawnEffectEvent>,
    mut screen_events: EventWriter<ScreenEffectEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
    mut anim_events: EventWriter<CharacterAnimationEvent>, // 新增
    enemy_sprite_query: Query<(Entity, &EnemySpriteMarker, &Transform)>,
    env: Option<Res<Environment>>,
) {
    let enemy_sprite_query_with_markers = &enemy_sprite_query;
    if !cinematic.active { return; }

    cinematic.timer.tick(time.delta());
    cinematic.effect_timer.tick(time.delta());

    let elapsed = cinematic.timer.elapsed_secs();
    let env_ref = env.as_ref().map(|r| r.as_ref());

    // --- 阶段 A: 持续预兆 (0.0s - 2.2s) ---
    if elapsed < 2.2 {
        if cinematic.effect_timer.just_finished() {
            // 预兆期不再降下真实闪电，改为灵气涌动和轻微震动
            screen_events.send(ScreenEffectEvent::Shake { trauma: 0.1, decay: 5.0 });
            sfx_events.send(PlaySfxEvent::with_volume(SfxType::LightningStrike, 0.2));
        }
    }

    // --- 阶段 B: 三连天罚闪击 (2.2s / 2.5s / 2.8s) ---
    let flash_times = [2.2, 2.5, 2.8];
    for (i, &strike_time) in flash_times.iter().enumerate() {
        if elapsed >= strike_time && cinematic.flash_count == i as u32 {
            info!("【天象演出】闪击 {}！", i + 1);
            
            // 1. 全屏视觉反馈
            screen_events.send(ScreenEffectEvent::Flash { 
                color: Color::srgba(0.9, 0.9, 1.0, 0.7 + (i as f32 * 0.15)), 
                duration: 0.2 
            });
            screen_events.send(ScreenEffectEvent::Shake { 
                trauma: 0.5 + (i as f32 * 0.25), 
                decay: 2.5 
            });
            sfx_events.send(PlaySfxEvent::with_volume(SfxType::LightningStrike, 1.0));

            // 2. [关键修复] 在所有存活敌人位置降下真实折线闪电
            for (_, _, transform) in enemy_sprite_query.iter() {
                effect_events.send(SpawnEffectEvent::new(EffectType::Lightning, transform.translation));
            }

            // 3. 只有最后一次闪击 (2.8s) 执行最终伤害与环境切换
            if i == 2 && !cinematic.damage_applied {
                if cinematic.environment_name == "雷暴" {
                    commands.insert_resource(Environment::thunder_storm());
                }

                let final_damage = if let Ok((player, _)) = player_query.get_single() {
                    player.calculate_outgoing_damage_with_env(cinematic.pending_damage, env_ref)
                } else {
                    cinematic.pending_damage
                };

                for mut enemy in enemy_query.iter_mut() {
                    if enemy.hp > 0 {
                        enemy.take_damage_with_env(final_damage, env_ref);
                        let is_dead = enemy.hp <= 0;
                        
                        // 查找对应的 3D 渲染实体位置并触发动画
                        for (render_entity, marker, transform) in enemy_sprite_query_with_markers.iter() {
                            if marker.id == enemy.id {
                                // 触发受击特效
                                effect_events.send(SpawnEffectEvent::new(EffectType::Hit, transform.translation));
                                
                                // [关键修复] 触发死亡或受击动画
                                if is_dead {
                                    anim_events.send(CharacterAnimationEvent { 
                                        target: render_entity, 
                                        animation: crate::components::sprite::AnimationState::Death 
                                    });
                                } else {
                                    anim_events.send(CharacterAnimationEvent { 
                                        target: render_entity, 
                                        animation: crate::components::sprite::AnimationState::Hit 
                                    });
                                }
                            }
                        }
                    }
                }
                info!("【天象演出】最终天罚降临，造成 {} 点伤害", final_damage);
                cinematic.damage_applied = true;
            }

            cinematic.flash_count += 1;
        }
    }

    // --- 演出结束 (4.0s) ---
    if cinematic.timer.finished() {
        cinematic.active = false;
        cinematic.flash_count = 0;
        info!("【天象演出】圆满结束");
    }
}

/// 处理手牌中的诅咒效果
fn handle_curse_effects(
    mut player_query: Query<&mut Player>,
    hand_query: Query<&Hand, Changed<Hand>>,
    mut effect_events: EventWriter<SpawnEffectEvent>,
    mut screen_events: EventWriter<ScreenEffectEvent>,
    env: Option<Res<Environment>>,
) {
    if let Ok(hand) = hand_query.get_single() {
        if let Ok(mut player) = player_query.get_single_mut() {
            let env_ref = env.as_ref().map(|r| r.as_ref());
            for card in &hand.cards {
                match &card.effect {
                    CardEffect::CurseDamage { amount } => {
                        info!("【诅咒】{} 抽动，造成 {} 点伤害", card.name, amount);
                        player.take_damage_with_env(*amount, env_ref);
                        effect_events.send(SpawnEffectEvent::new(EffectType::Slash, Vec3::new(-3.5, 0.0, 0.5)));
                        screen_events.send(ScreenEffectEvent::Shake { trauma: 0.3, decay: 4.0 });
                    }
                    CardEffect::CurseWeakness => {
                        info!("【诅咒】{} 侵蚀，施加虚弱", card.name);
                        player.weakness += 1;
                    }
                    _ => {}
                }
            }
        }
    }
}

pub fn cleanup_combat_ui(
    mut commands: Commands,
    query: Query<Entity, With<CombatUiRoot>>,
    player_query: Query<&Player>,
    player_resource: Option<ResMut<Player>>,
    // 增加对残留实体的查询
    enemy_query: Query<Entity, With<Enemy>>,
    sprite_query: Query<Entity, With<SpriteMarker>>,
    particle_query: Query<Entity, With<ParticleMarker>>,
    emitter_query: Query<Entity, With<EmitterMarker>>,
    piles_query: Query<Entity, Or<(With<DrawPile>, With<DiscardPile>, With<Hand>)>>,
) {
    // 1. 持久化同步
    if let Ok(player) = player_query.get_single() {
        if let Some(mut player_res) = player_resource {
            *player_res = player.clone();
            info!("【持久化】修士状态已同步：HP={}, 灵石={}", player_res.hp, player_res.gold);
        } else {
            warn!("【持久化】同步失败：未找到 Player 资源");
        }
    }

    // 2. 彻底肃清战斗实体
    for entity in query.iter() { commands.entity(entity).despawn_recursive(); }
    for entity in enemy_query.iter() { commands.entity(entity).despawn_recursive(); }
    for entity in sprite_query.iter() { commands.entity(entity).despawn_recursive(); }
    for entity in particle_query.iter() { commands.entity(entity).despawn_recursive(); }
    for entity in emitter_query.iter() { commands.entity(entity).despawn_recursive(); }
    for entity in piles_query.iter() { commands.entity(entity).despawn_recursive(); }
    
    info!("【战斗清理】已彻底销毁所有战斗相关实体");
}

/// 处理战斗界面按钮点击
fn handle_combat_button_clicks(
    mut commands: Commands, // 改为 mut
    mut next_state: ResMut<NextState<GameState>>,
    mut combat_state: ResMut<CombatState>,
    mut enemy_query: Query<(Entity, &Enemy)>,
    mut queue: ResMut<EnemyActionQueue>,
    mut hand_query: Query<&mut Hand>,
    mut discard_pile_query: Query<&mut DiscardPile>,
    hand_area_query: Query<Entity, With<HandArea>>, // 引入 UI 查询
    mut button_queries: ParamSet<(
        Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<EndTurnButton>)>,
        Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<ReturnToMapButton>)>,
    )>,
) {
    for (interaction, mut _color) in button_queries.p0().iter_mut() {
        if matches!(interaction, Interaction::Pressed) {
            info!("【战斗】玩家结束回合，队列初始化开始行动");

            // 1. 立即清空手牌并进入弃牌堆
            if let Ok(mut hand) = hand_query.get_single_mut() {
                if let Ok(mut discard_pile) = discard_pile_query.get_single_mut() {
                    while let Some(card) = hand.remove_card(0) {
                        discard_pile.add_card(card);
                    }
                    info!("【战斗】手牌已清空至弃牌堆");
                }
            }

            // [关键修复] 强制销毁 UI，避免视觉残留或闪烁
            if let Ok(hand_area) = hand_area_query.get_single() {
                commands.entity(hand_area).despawn_descendants();
            }
            
            // 2. 搜集所有存活敌人进入行动队列
            let mut enemies: Vec<Entity> = enemy_query.iter()
                .filter(|(_, e)| e.hp > 0)
                .map(|(entity, _)| entity)
                .collect();
            
            // 排序，确保从左到右行动
            enemies.sort(); 

            queue.enemies = enemies;
            queue.current_index = 0;
            queue.timer = Timer::from_seconds(0.1, TimerMode::Once); // 立即开始第一个动作
            queue.processing = true;

            combat_state.phase = TurnPhase::EnemyTurn;
        }
    }

    // 处理返回地图
    for (interaction, _) in button_queries.p1().iter() {
        if matches!(interaction, Interaction::Pressed) {
            next_state.set(GameState::Map);
        }
    }
}

/// 核心系统：逐个处理敌人回合动作
pub fn process_enemy_turn_queue(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut queue: ResMut<EnemyActionQueue>,
    combat_state_opt: Option<ResMut<CombatState>>,
    mut player_query: Query<(&mut Player, &crate::components::Cultivation)>,
    mut enemy_query: Query<&mut Enemy>,
    mut hand_query: Query<&mut Hand>,
    mut discard_pile_query: Query<&mut DiscardPile>,
    events: (
        EventWriter<CharacterAnimationEvent>,
        EventWriter<SpawnEffectEvent>,
        EventWriter<ScreenEffectEvent>,
        EventWriter<EnemyAttackEvent>,
    ),
    enemy_sprite_query: Query<(Entity, &crate::components::sprite::EnemySpriteMarker, &Transform)>,
    time: Res<Time>,
    env: Option<Res<Environment>>,
) {
    let (mut anim_events, mut effect_events, mut screen_events, mut attack_events) = events;
    let Some(mut combat_state) = combat_state_opt else { return; };
    if !queue.processing || combat_state.phase != TurnPhase::EnemyTurn {
        return;
    }

    queue.timer.tick(time.delta());

    if queue.timer.finished() {
        if queue.current_index < queue.enemies.len() {
            let enemy_entity = queue.enemies[queue.current_index];
            
            if let Ok(mut enemy) = enemy_query.get_mut(enemy_entity) {
                if enemy.hp <= 0 {
                    queue.current_index += 1;
                    queue.timer = Timer::from_seconds(0.1, TimerMode::Once);
                    return;
                }

                enemy.start_turn();
                let intent = enemy.execute_intent();
                let enemy_id = enemy.id;

                // [关键修复] 恢复敌人攻击动画与特效的隔离匹配
                for (render_entity, marker, transform) in enemy_sprite_query.iter() {
                    if marker.id == enemy_id {
                        let anim = match intent {
                            EnemyIntent::Attack { .. } => {
                                match enemy.enemy_type {
                                    EnemyType::DemonicWolf => {
                                        effect_events.send(SpawnEffectEvent::new(EffectType::SwordEnergy, transform.translation).burst(15));
                                        crate::components::sprite::AnimationState::WolfAttack
                                    },
                                    EnemyType::PoisonSpider => {
                                        // 恢复吐丝特效
                                        effect_events.send(SpawnEffectEvent::new(EffectType::WebShot, transform.translation).burst(30));
                                        
                                        // 恢复覆盖全屏的实体蛛网 Mesh
                                        let web_texture = asset_server.load("textures/web_effect.png");
                                        commands.spawn((
                                            crate::components::sprite::Ghost { ttl: 1.2 },
                                            Mesh3d(meshes.add(Rectangle::new(2.5, 2.5))), 
                                            MeshMaterial3d(materials.add(StandardMaterial {
                                                base_color: Color::srgba(1.0, 1.0, 1.0, 0.8),
                                                base_color_texture: Some(web_texture),
                                                alpha_mode: AlphaMode::Blend,
                                                unlit: true,
                                                ..default()
                                            })),
                                            Transform::from_xyz(-3.5, 0.0, 0.5).with_rotation(Quat::from_rotation_z(0.3)),
                                            CombatUiRoot,
                                        ));
                                        crate::components::sprite::AnimationState::SpiderAttack
                                    },
                                    EnemyType::CursedSpirit => {
                                        // 怨灵增加幽冥粒子
                                        effect_events.send(SpawnEffectEvent::new(EffectType::AmbientSpirit, transform.translation).burst(25));
                                        crate::components::sprite::AnimationState::SpiritAttack
                                    },
                                    EnemyType::GreatDemon => {
                                        // 首领攻击增加雷光
                                        effect_events.send(SpawnEffectEvent::new(EffectType::Lightning, transform.translation).burst(15));
                                        crate::components::sprite::AnimationState::DemonCast
                                    },
                                    _ => crate::components::sprite::AnimationState::DemonAttack,
                                }
                            },
                            EnemyIntent::Seal { .. } | EnemyIntent::Curse { .. } | EnemyIntent::Debuff { .. } => {
                                match enemy.enemy_type {
                                    EnemyType::PoisonSpider => {
                                        effect_events.send(SpawnEffectEvent::new(EffectType::WebShot, transform.translation).burst(20));
                                        crate::components::sprite::AnimationState::SpiderAttack
                                    },
                                    EnemyType::CursedSpirit | EnemyType::GreatDemon => {
                                        // 诅咒或施法时产生暗影效果
                                        effect_events.send(SpawnEffectEvent::new(EffectType::AmbientSpirit, transform.translation).burst(30));
                                        crate::components::sprite::AnimationState::DemonCast
                                    }
                                    _ => crate::components::sprite::AnimationState::DemonCast,
                                }
                            }
                            _ => crate::components::sprite::AnimationState::DemonCast,
                        };
                        anim_events.send(CharacterAnimationEvent { target: render_entity, animation: anim });
                    }
                }

                match intent {
                    EnemyIntent::Attack { damage } => {
                        let final_damage = enemy.calculate_outgoing_damage_with_env(damage, env.as_ref().map(|r| r.as_ref()));
                        if let Ok((mut player, _)) = player_query.get_single_mut() {
                            player.take_damage_with_env(final_damage, env.as_ref().map(|r| r.as_ref()));
                            attack_events.send(EnemyAttackEvent::new(final_damage, false));
                            screen_events.send(ScreenEffectEvent::Shake { trauma: 0.6, decay: 6.0 });
                            if player.hp <= 0 {
                                next_state.set(GameState::GameOver);
                                queue.processing = false;
                                return; 
                            }
                        }
                    }
                    EnemyIntent::Defend { .. } => {
                        effect_events.send(SpawnEffectEvent::new(EffectType::Shield, Vec3::new(2.5, 0.5, 0.5)));
                    }
                    EnemyIntent::Curse { card_id } => {
                        if let Ok(mut discard_pile) = discard_pile_query.get_single_mut() {
                            // 创建诅咒卡并加入弃牌堆
                            let curse_card = Card::new(
                                card_id, "心魔干扰", "【诅咒】干扰心神，难以自拔。",
                                CardType::Curse, 0, CardEffect::CurseDamage { amount: 2 },
                                CardRarity::Special, "textures/cards/special.png"
                            );
                            discard_pile.add_card(curse_card);
                            info!("【战斗】敌人向你的归墟注入了心魔！");
                        }
                    }
                    EnemyIntent::Seal { slot_index, duration } => {
                        if let Ok(mut hand) = hand_query.get_single_mut() {
                            hand.seal_slot(slot_index, duration);
                            info!("【战斗】你的气穴被封印了！");
                        }
                    }
                    _ => {}
                }
            }
            
            queue.current_index += 1;
            queue.timer = Timer::from_seconds(1.2, TimerMode::Once);
        } else {
            // (玩家回合开始代码省略)
            queue.processing = false;
            if let Ok((mut player, _)) = player_query.get_single_mut() {
                player.start_turn();
            }
            combat_state.cards_drawn_this_turn = false;
            combat_state.phase = TurnPhase::PlayerAction;
        }
    }
}

/// 更新战斗UI显示
fn update_combat_ui(
    mut commands: Commands,
    time: Res<Time>,
    player_query: Query<&Player>,
    enemy_query: Query<&Enemy>,
    mut hp_bar_query: Query<(&mut Node, Has<PlayerHpBarMarker>, Has<PlayerHpBufferMarker>, Option<&EnemyHpBarMarker>, Option<&EnemyHpBufferMarker>)>,
    mut intent_icon_query: Query<(&IntentIconMarker, &mut ImageNode, &mut Visibility)>,
    ui_query: Query<(Entity, &EnemyStatusUi)>,
    asset_server: Res<AssetServer>,
    env: Option<Res<Environment>>, // 新增环境资源
    mut text_queries: ParamSet<(
        Query<&mut Text, With<PlayerHpText>>,
        Query<&mut Text, With<PlayerEnergyText>>,
        Query<&mut Text, With<PlayerBlockText>>,
        Query<&mut Text, With<TopBarHpText>>,
        Query<&mut Text, With<TopBarGoldText>>,
        Query<(&EnemyHpText, &mut Text)>,
        Query<(&EnemyIntentText, &mut Text)>,
    )>,
) {
    if let Ok(p) = player_query.get_single() {
        if let Ok(mut t) = text_queries.p0().get_single_mut() { t.0 = format!("{}/{}", p.hp, p.max_hp); }
        if let Ok(mut t) = text_queries.p1().get_single_mut() { t.0 = format!("{}/{}", p.energy, p.max_energy); }
        if let Ok(mut t) = text_queries.p2().get_single_mut() { t.0 = format!("护甲: {}", p.block); }
        if let Ok(mut t) = text_queries.p3().get_single_mut() { t.0 = format!("道行: {}/{}", p.hp, p.max_hp); }
        if let Ok(mut t) = text_queries.p4().get_single_mut() { t.0 = format!("灵石: {}", p.gold); }
    }

    // ... (血条同步逻辑省略)
    let p_data = player_query.get_single().ok();
    
    for (mut node, is_p_bar, is_p_buf, e_bar_opt, e_buf_opt) in hp_bar_query.iter_mut() {
        if is_p_bar || is_p_buf {
            if let Some(p) = p_data {
                let hp_percent = (p.hp as f32 / p.max_hp as f32) * 100.0;
                if is_p_bar {
                    node.width = Val::Percent(hp_percent.clamp(0.0, 100.0));
                } else {
                    if let Val::Percent(curr_w) = node.width {
                        if curr_w > hp_percent {
                            let new_w = (curr_w - 40.0 * time.delta_secs()).max(hp_percent);
                            node.width = Val::Percent(new_w);
                        } else { node.width = Val::Percent(hp_percent); }
                    }
                }
            }
        } else if let Some(e_bar) = e_bar_opt {
            if let Ok(enemy) = enemy_query.get(e_bar.owner) {
                let hp_percent = (enemy.hp as f32 / enemy.max_hp as f32) * 100.0;
                node.width = Val::Percent(hp_percent.clamp(0.0, 100.0));
            }
        } else if let Some(e_buf) = e_buf_opt {
            if let Ok(enemy) = enemy_query.get(e_buf.owner) {
                let hp_percent = (enemy.hp as f32 / enemy.max_hp as f32) * 100.0;
                if let Val::Percent(curr_w) = node.width {
                    if curr_w > hp_percent {
                        let new_w = (curr_w - 40.0 * time.delta_secs()).max(hp_percent);
                        node.width = Val::Percent(new_w);
                    } else { node.width = Val::Percent(hp_percent); }
                }
            }
        }
    }

    for (marker, mut text) in text_queries.p5().iter_mut() {
        if let Ok(enemy) = enemy_query.get(marker.owner) {
            text.0 = format!("{}/{}", enemy.hp, enemy.max_hp);
        }
    }

    let player_data = player_query.get_single().ok();
    let env_ref = env.as_ref().map(|r| r.as_ref());
    
    for (marker, mut text) in text_queries.p6().iter_mut() {
        if let Ok(enemy) = enemy_query.get(marker.owner) {
            match &enemy.intent {
                EnemyIntent::Attack { damage } => {
                    let after_weakness = enemy.calculate_outgoing_damage_with_env(*damage, env_ref);
                    let final_val = if let Some(p) = player_data {
                        p.calculate_incoming_damage_with_env(after_weakness, env_ref)
                    } else {
                        after_weakness
                    };
                    text.0 = format!("攻击 {}", final_val);
                }
                EnemyIntent::Defend { block } => {
                    text.0 = format!("防御 {}", block);
                }
                EnemyIntent::Debuff { poison, weakness } => {
                    text.0 = format!("邪术(毒{}/弱{})", poison, weakness);
                }
                _ => {
                    text.0 = "观察中...".to_string();
                }
            }
        }
    }

    // 3. 同步意图图标
    for (marker, mut img, mut vis) in intent_icon_query.iter_mut() {
        if let Ok(enemy) = enemy_query.get(marker.owner) {
            let (tex, visible) = match &enemy.intent {
                EnemyIntent::Attack { .. } => ("textures/cards/attack.png", Visibility::Visible),
                EnemyIntent::Defend { .. } => ("textures/cards/defense.png", Visibility::Visible),
                EnemyIntent::Debuff { .. } => ("textures/cards/special.png", Visibility::Visible),
                _ => ("textures/cards/default.png", Visibility::Hidden),
            };
            img.image = asset_server.load(tex);
            *vis = visible;
        }
    }

    // [关键修复] 自动清理已死亡敌人的 UI
    // 现在直接通过 owner 实体引用检查，无需遍历匹配 ID，效率和安全性大幅提升
    for (ui_entity, status_ui) in ui_query.iter() {
        if let Ok(enemy) = enemy_query.get(status_ui.owner) {
            // 实体存在，检查生命值
            if enemy.hp <= 0 {
                commands.entity(ui_entity).despawn_recursive();
            }
        } else {
            // 实体已不存在 (被 despawn) -> 清理 UI
            commands.entity(ui_entity).despawn_recursive();
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
    combat_state_opt: Option<ResMut<CombatState>>, // 关键修复：改为 Option 防止系统参数未就绪闪退
) {
    let Some(mut combat_state) = combat_state_opt else { return; };
    if combat_state.cards_drawn_this_turn { return; }

    if let Ok(mut draw_pile) = draw_pile_query.get_single_mut() {
        if let Ok(mut hand) = hand_query.get_single_mut() {
            info!("【战斗】初始洗牌并抽取 5 张机缘");
            
            // 1. 全量洗牌
            use rand::seq::SliceRandom;
            draw_pile.cards.shuffle(&mut rand::thread_rng());
            
            // 2. 抽取 5 张
            let to_draw = 5.min(draw_pile.cards.len());
            for _ in 0..to_draw {
                if let Some(card) = draw_pile.draw_card() {
                    hand.add_card(card);
                }
            }
            
            combat_state.cards_drawn_this_turn = true;
        }
    }
}

/// 回合开始时抽牌
fn draw_cards_on_turn_start(
    mut draw_pile_query: Query<&mut DrawPile>,
    mut hand_query: Query<&mut Hand>,
    mut discard_pile_query: Query<&mut DiscardPile>,
    player_query: Query<&Player>,
    combat_state_opt: Option<ResMut<CombatState>>, // 关键修复：改为 Option
) {
    let Some(mut combat_state) = combat_state_opt else { return; };
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
            // 关键：更新封印状态（每回合减少持续时间）
            hand.update_seals();
            
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

fn handle_hand_card_hover(
    mut query: Query<(&Interaction, &HandCard, &mut ZIndex, &mut Transform), (With<HandCard>, Changed<Interaction>)>,
) {
    for (interaction, hand_card, mut z_index, mut transform) in query.iter_mut() {
        match interaction {
            Interaction::Hovered => {
                // 悬停：视觉上浮、放大、置顶
                // 注意：修改 transform.translation.y 而非 node.bottom，防止 Picking 区域随之位移导致闪烁
                transform.translation.y = 40.0; 
                *z_index = ZIndex(100);
                transform.scale = Vec3::splat(1.25); // 放大 25%
                transform.rotation = Quat::IDENTITY; // 摆正
            }
            Interaction::None => {
                // 恢复：回位、缩小
                transform.translation.y = 0.0;
                *z_index = ZIndex(hand_card.index as i32);
                transform.scale = Vec3::ONE;
                transform.rotation = Quat::from_rotation_z(hand_card.base_rotation);
            }
            _ => {}
        }
    }
}

// ============================================================================
// 出牌系统
// ============================================================================

/// 处理卡牌点击事件
fn handle_card_play(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    card_query: Query<(&Interaction, &HandCard), (Changed<Interaction>, With<HandCard>)>,
    mut player_query: Query<(&mut Player, &crate::components::Cultivation)>,
    mut hand_query: Query<&mut Hand>,
    mut draw_pile_query: Query<&mut DrawPile>,
    mut discard_pile_query: Query<&mut DiscardPile>,
    mut enemy_query: Query<&mut Enemy>,
    mut events: (
        EventWriter<SpawnEffectEvent>,
        EventWriter<ScreenEffectEvent>,
        EventWriter<PlaySfxEvent>,
        EventWriter<CharacterAnimationEvent>,
        EventWriter<DamageEffectEvent>,
        EventWriter<StatusEffectEvent>,
    ),
    env: Option<Res<Environment>>,
    mut heavenly_cinematic: ResMut<HeavenlyStrikeCinematic>, 
    victory_delay: Res<VictoryDelay>, // 引入资源
    queries: (
        Query<Entity, With<PlayerSpriteMarker>>,
        Query<(Entity, &crate::components::sprite::EnemySpriteMarker, &Transform)>,
        Query<(Entity, &crate::components::sprite::EnemySpriteMarker, &crate::components::sprite::PhysicalImpact)>,
        Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    ),
) {
    // [安全门禁] 如果已经处于胜利结算阶段，禁止打牌
    if victory_delay.active { return; }

    let (mut effect_events, mut screen_events, mut sfx_events, mut anim_events, mut damage_events, mut status_events) = events;
    let (player_sprite_query, enemy_sprite_query, enemy_impact_query, camera_query) = queries;
    for (interaction, hand_card) in card_query.iter() {
        if matches!(interaction, Interaction::Pressed) {
            // ... (能量检查省略)
            let player_energy = if let Ok((p, _)) = player_query.get_single() {
                p.energy
            } else {
                0
            };

            let card_opt = if let Ok(hand) = hand_query.get_single() {
                let card_index = hand.cards.iter().position(|c| c.id == hand_card.card_id);
                card_index.map(|i| hand.cards[i].clone())
            } else {
                None
            };

            if let Some(card) = card_opt {
                if player_energy >= card.cost {
                    info!("打出卡牌: {} (消耗: {})", card.name, card.cost);
                    
                    // 1. 触发玩家动画 (精准隔离：御剑冲刺，天象原地)
                    if let Ok(player_entity) = player_sprite_query.get_single() {
                        if card.card_type == CardType::Attack {
                            let anim = if card.name.contains("御剑术") {
                                // 真正的御剑术：回旋冲刺
                                effect_events.send(SpawnEffectEvent::new(EffectType::SwordEnergy, Vec3::new(-3.5, 1.0, 0.2)));
                                crate::components::sprite::AnimationState::ImperialSword
                            } else if card.name.contains("天象") {
                                // 天象法术：原地施法 (不再即时产生雷击)
                                crate::components::sprite::AnimationState::HeavenCast
                            } else {
                                // 近战类执行冲刺
                                crate::components::sprite::AnimationState::Attack
                            };

                            anim_events.send(CharacterAnimationEvent {
                                target: player_entity,
                                animation: anim,
                            });
                        } else if card.card_type == CardType::Defense {
                            // 防御功法：彻底原地不动
                            anim_events.send(CharacterAnimationEvent {
                                target: player_entity,
                                animation: crate::components::sprite::AnimationState::Defense,
                            });
                        }
                    }

                    sfx_events.send(PlaySfxEvent::new(SfxType::CardPlay));
                    if let Ok((mut player, _)) = player_query.get_single_mut() {
                        player.energy -= card.cost;
                    }

                        apply_card_effect(
                            &card,
                            &mut commands,
                            &asset_server,
                            &mut player_query,
                            &mut enemy_query,
                            &mut draw_pile_query,
                            &mut discard_pile_query,
                            &mut hand_query,
                            &mut effect_events,
                            &mut screen_events,
                            &mut anim_events,
                            &mut damage_events,
                            &mut status_events,
                            &enemy_sprite_query,
                            &enemy_impact_query,
                            &camera_query,
                            env.as_ref().map(|r| r.as_ref()),
                            &mut heavenly_cinematic, // 传递演出资源
                        );

                    // 3. 移出手牌
                    if let Ok(mut hand) = hand_query.get_single_mut() {
                        if let Some(index) = hand.cards.iter().position(|c| c.id == card.id) {
                            let played_card = hand.remove_card(index).unwrap();
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
    card: &crate::components::cards::Card,
    _commands: &mut Commands,
    _asset_server: &Res<AssetServer>,
    player_query: &mut Query<(&mut Player, &crate::components::Cultivation)>,
    enemy_query: &mut Query<&mut Enemy>,
    draw_pile_query: &mut Query<&mut DrawPile>,
    discard_pile_query: &mut Query<&mut DiscardPile>,
    hand_query: &mut Query<&mut Hand>,
    effect_events: &mut EventWriter<SpawnEffectEvent>,
    screen_events: &mut EventWriter<ScreenEffectEvent>,
    anim_events: &mut EventWriter<CharacterAnimationEvent>,
    damage_events: &mut EventWriter<DamageEffectEvent>,
    status_events: &mut EventWriter<StatusEffectEvent>,
    enemy_sprite_query: &Query<(Entity, &crate::components::sprite::EnemySpriteMarker, &Transform)>,
    enemy_impact_query: &Query<(Entity, &crate::components::sprite::EnemySpriteMarker, &crate::components::sprite::PhysicalImpact)>,
    _camera_query: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    environment: Option<&Environment>,
    heavenly_cinematic: &mut HeavenlyStrikeCinematic, // 新增
) {
    let card_name = card.name.clone();
    match &card.effect {
        // ... (状态施加省略)
        CardEffect::ApplyStatus { status, count } => {
            if let Some(mut enemy) = enemy_query.iter_mut().find(|e| e.hp > 0) {
                let target_id = enemy.id;
                match status {
                    crate::components::cards::StatusType::Weakness => {
                        enemy.weakness += *count;
                    }
                    crate::components::cards::StatusType::Vulnerable => {
                        enemy.vulnerable += *count;
                    }
                    crate::components::cards::StatusType::Poison => {
                        enemy.poison += *count;
                    }
                }

                for (entity, marker, _) in enemy_sprite_query.iter() {
                    if marker.id == target_id {
                        let (msg, color) = match status {
                            crate::components::cards::StatusType::Weakness => ("虚弱！".to_string(), Color::srgb(0.7, 0.4, 1.0)),
                            crate::components::cards::StatusType::Vulnerable => ("易伤！".to_string(), Color::srgb(1.0, 0.3, 0.3)),
                            crate::components::cards::StatusType::Poison => ("中毒！".to_string(), Color::srgb(0.3, 0.8, 0.3)),
                        };
                        status_events.send(StatusEffectEvent { target: entity, msg, color });
                    }
                }
            }
        }
        CardEffect::DealDamage { amount } => {
            if let Ok((player, _)) = player_query.get_single() {
                let final_damage = player.calculate_outgoing_damage_with_env(*amount, environment);
                
                if let Some(mut enemy) = enemy_query.iter_mut().find(|e| e.hp > 0) {
                    let target_id = enemy.id;
                    enemy.take_damage_with_env(final_damage, environment);
                    let is_dead = enemy.hp <= 0;
                
                    for (entity, marker, transform) in enemy_sprite_query.iter() {
                        if marker.id == target_id {
                            effect_events.send(SpawnEffectEvent::new(EffectType::Fire, transform.translation));
                            if let Some((_, _, impact)) = enemy_impact_query.iter().find(|(_, m, _)| m.id == target_id) {
                                let x_world = impact.home_position.x * 100.0;
                                let y_world = (impact.home_position.z - 0.1) * 100.0;
                                damage_events.send(DamageEffectEvent { position: Vec2::new(x_world, y_world), amount: final_damage });
                            }
                            if is_dead {
                                anim_events.send(CharacterAnimationEvent { target: entity, animation: crate::components::sprite::AnimationState::Death });
                            } else {
                                anim_events.send(CharacterAnimationEvent { target: entity, animation: crate::components::sprite::AnimationState::Hit });
                            }
                        }
                    }
                    effect_events.send(SpawnEffectEvent::new(EffectType::Slash, Vec3::new(0.0, 0.0, 5.0)));
                    screen_events.send(ScreenEffectEvent::Shake { trauma: 0.5, decay: 8.0 });
                }
            }
        }
        CardEffect::DealAoEDamage { amount } => {
            let mut hit_count = 0;
            let final_damage = if let Ok((player, _)) = player_query.get_single() {
                player.calculate_outgoing_damage_with_env(*amount, environment)
            } else {
                *amount
            };

            for mut enemy in enemy_query.iter_mut() {
                if enemy.hp <= 0 { continue; }
                enemy.take_damage_with_env(final_damage, environment);
                let is_dead = enemy.hp <= 0;
                hit_count += 1;

                // 针对每个被击中的敌人，触发其渲染实体的动画
                for (render_entity, marker, _) in enemy_sprite_query.iter() {
                    if marker.id == enemy.id {
                        if is_dead {
                            anim_events.send(CharacterAnimationEvent { target: render_entity, animation: crate::components::sprite::AnimationState::Death });
                        } else {
                            anim_events.send(CharacterAnimationEvent { target: render_entity, animation: crate::components::sprite::AnimationState::Hit });
                        }
                    }
                }
            }
            
            if hit_count > 0 {
                // ... (后面是特效和飘字，保持不变，但删掉末尾统一发的 Hit 动画)
                for (_, _marker, impact) in enemy_impact_query.iter() {
                    let x_world = impact.home_position.x * 100.0;
                    let y_world = (impact.home_position.z - 0.1) * 100.0;
                    damage_events.send(DamageEffectEvent { position: Vec2::new(x_world, y_world), amount: final_damage });
                }

                if card_name.contains("万剑归宗") {
                    screen_events.send(ScreenEffectEvent::Shake { trauma: 1.0, decay: 0.45 });
                    let mut alive_enemies: Vec<(Entity, Vec2)> = Vec::new();
                    for (entity, marker, impact) in enemy_impact_query.iter() {
                        let world_pos_3d = impact.home_position;
                        let x_world = world_pos_3d.x * 100.0;
                        let y_world = (world_pos_3d.z - 0.1) * 100.0;
                        alive_enemies.push((entity, Vec2::new(x_world, y_world)));
                    }

                    if !alive_enemies.is_empty() {
                        let total_swords = 80;
                        let swords_per_enemy = total_swords / alive_enemies.len();
                        for (idx, (entity, _)) in alive_enemies.iter().enumerate() {
                            effect_events.send(
                                SpawnEffectEvent::new(EffectType::WanJian, Vec3::new(-350.0, -80.0, 0.5))
                                    .burst(swords_per_enemy)
                                    .with_target(alive_enemies[idx].1)
                                    .with_target_entity(*entity)
                                    .with_target_group(alive_enemies.clone())
                                    .with_target_index(idx)
                            );
                        }
                    }
                                } else {
                                    screen_events.send(ScreenEffectEvent::Shake { trauma: 0.5, decay: 4.0 });
                                }
                            }
                        }
                
        CardEffect::GainBlock { amount } => {
            if let Ok((mut player, _)) = player_query.get_single_mut() {
                player.gain_block_with_env(*amount, environment);
                info!("【卡牌】获得 {} 点护甲 (受环境修正)", amount);
                effect_events.send(SpawnEffectEvent::new(EffectType::Shield, Vec3::new(-3.5, 0.5, 0.5)));
            }
        }
        CardEffect::Heal { amount } => {
            if let Ok((mut player, _)) = player_query.get_single_mut() {
                player.heal(*amount);
                info!("【卡牌】回复 {} 点生命", amount);
                effect_events.send(SpawnEffectEvent::new(EffectType::Heal, Vec3::new(-3.5, -0.5, 0.5)));
            }
        }
        CardEffect::ChangeEnvironment { name } => {
            if card_name.contains("引雷术") {
                info!("【卡牌】引动九天雷霆演出开始...");
                // 仅启动演出，不立即扣血或切换环境，基础伤害提升至 20
                heavenly_cinematic.start(20, name.clone());
            } else {
                info!("【卡牌】天象异变！环境变为: {}", name);
                if name == "浓雾" {
                    _commands.insert_resource(Environment::thick_fog());
                    screen_events.send(ScreenEffectEvent::Flash { color: Color::srgba(0.7, 0.7, 0.7, 0.4), duration: 0.5 });
                } else {
                    _commands.insert_resource(Environment::default());
                }
                if let Ok((mut player, _)) = player_query.get_single_mut() {
                    player.gain_block_with_env(5, environment);
                }
            }
        }

        CardEffect::GainEnergy { amount } => {
            if let Ok((mut player, _)) = player_query.get_single_mut() {
                player.gain_energy(*amount);
                info!("【卡牌】获得 {} 点灵力", amount);
                effect_events.send(SpawnEffectEvent::new(EffectType::AmbientSpirit, Vec3::new(-3.5, 0.0, 0.5)).burst(20));
            }
        }
        CardEffect::AttackAndDraw { damage, cards } => {
            // 伤害部分
            if let Ok((player, _)) = player_query.get_single() {
                let final_damage = player.calculate_outgoing_damage_with_env(*damage, environment);
                if let Some(mut enemy) = enemy_query.iter_mut().find(|e| e.hp > 0) {
                    enemy.take_damage_with_env(final_damage, environment);
                    effect_events.send(SpawnEffectEvent::new(EffectType::Slash, Vec3::new(0.0, 0.0, 5.0)));
                }
            }
            // 抽牌部分
            let mut drawn = 0;
            if let Ok(mut draw_pile) = draw_pile_query.get_single_mut() {
                for _ in 0..*cards {
                    if let Some(card) = draw_pile.draw_card() {
                        if let Ok(mut hand) = hand_query.get_single_mut() {
                            if hand.add_card(card) { drawn += 1; }
                        }
                    }
                }
            }
            if drawn > 0 { info!("【卡牌】造成伤害并抽了 {} 张牌", drawn); }
        }
        CardEffect::MultiAttack { damage, times } => {
            if let Ok((player, _)) = player_query.get_single() {
                let final_damage = player.calculate_outgoing_damage_with_env(*damage, environment);
                if let Some(mut enemy) = enemy_query.iter_mut().find(|e| e.hp > 0) {
                    let target_id = enemy.id;
                    let mut total_damage = 0;
                    
                    for _ in 0..*times {
                        enemy.take_damage_with_env(final_damage, environment);
                        total_damage += final_damage;
                        // 触发多次斩击特效
                        effect_events.send(SpawnEffectEvent::new(EffectType::Slash, Vec3::new(0.0, 0.0, 5.0))); 
                    }
                    
                    let is_dead = enemy.hp <= 0;
                    info!("【卡牌】{} 次攻击，每次 {} 点伤害，共 {} 点，敌人剩余HP: {}", times, damage, total_damage, enemy.hp);

                    // [关键修复] 补充动画和飘字反馈
                    for (entity, marker, _) in enemy_sprite_query.iter() {
                        if marker.id == target_id {
                            // 飘字 (显示总伤害)
                            if let Some((_, _, impact)) = enemy_impact_query.iter().find(|(_, m, _)| m.id == target_id) {
                                let x_world = impact.home_position.x * 100.0;
                                let y_world = (impact.home_position.z - 0.1) * 100.0;
                                damage_events.send(DamageEffectEvent { position: Vec2::new(x_world, y_world), amount: total_damage });
                            }
                            
                            // 死亡/受击动画
                            if is_dead {
                                anim_events.send(CharacterAnimationEvent { target: entity, animation: crate::components::sprite::AnimationState::Death });
                            } else {
                                anim_events.send(CharacterAnimationEvent { target: entity, animation: crate::components::sprite::AnimationState::Hit });
                            }
                        }
                    }
                }
            }
        }
        CardEffect::DrawCards { amount } => {
            let mut drawn = 0;
            if let Ok(mut draw_pile) = draw_pile_query.get_single_mut() {
                for _ in 0..*amount {
                    if let Some(card) = draw_pile.draw_card() {
                        if let Ok(mut hand) = hand_query.get_single_mut() {
                            if hand.add_card(card) { drawn += 1; }
                        }
                    } else { break; }
                }
            }
            if drawn > 0 { info!("【卡牌】抽了 {} 张牌", drawn); }
        }
        _ => {}
    }
}

// ============================================================================
// 战斗结算与延迟系统
// ============================================================================

/// 战斗结算包装器 (仅供集成测试使用)
pub fn check_combat_end_wrapper(
    state: Res<State<GameState>>,
    player_query: Query<(&mut Player, &mut crate::components::Cultivation)>,
    enemy_query: Query<&Enemy>,
    next_state: ResMut<NextState<GameState>>,
    victory_events: EventWriter<VictoryEvent>,
    victory_delay: ResMut<VictoryDelay>,
    asset_server: Res<AssetServer>,
    commands: Commands,
) {
    check_combat_end(state, player_query, enemy_query, next_state, victory_events, victory_delay, asset_server, commands);
}

/// 检查战斗是否结束
fn check_combat_end(
    state: Res<State<GameState>>,
    mut player_query: Query<(&mut Player, &mut crate::components::Cultivation)>,
    enemy_query: Query<&Enemy>,
    mut next_state: ResMut<NextState<GameState>>,
    mut victory_events: EventWriter<VictoryEvent>,
    mut victory_delay: ResMut<VictoryDelay>,
    asset_server: Res<AssetServer>, // 新增参数
    mut commands: Commands, // 确保有 commands
) {
    if **state != GameState::Combat { return; }

    // 1. 检查玩家是否败北
    if let Ok((player, _)) = player_query.get_single() {
        if player.hp <= 0 {
            info!("【战斗】身陨道消，由于道行耗尽...");
            next_state.set(GameState::GameOver);
            return;
        }
    }

    // 2. 检查众妖是否伏诛 (全歼判定)
    let any_alive = enemy_query.iter().any(|e| e.hp > 0);
    
    if !any_alive && !enemy_query.is_empty() {
        if victory_delay.active { return; }

        info!("【战斗】众妖肃清，机缘显现！");
        let chinese_font = asset_server.load("fonts/Arial Unicode.ttf");
        
        // 1. 获得感悟
        if let Ok((mut player, mut cultivation)) = player_query.get_single_mut() {
            let insight_gain = 50;
            cultivation.gain_insight(insight_gain);
            info!("【修仙】获得 {} 点感悟，当前: {}/{}", insight_gain, cultivation.insight, cultivation.get_threshold());

            // 2. [新增] 战后搜刮：获得灵石掉落 (10-25 随机)
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let gold_drop = rng.gen_range(10..26);
            player.gold += gold_drop;
            info!("【战斗】搜刮战场，获得 {} 块灵石！当前持有: {}", gold_drop, player.gold);
        }

        victory_events.send(VictoryEvent);
        victory_delay.active = true;
        victory_delay.elapsed = 0.0;

        // --- [新增] 胜利横幅演出 ---
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Px(120.0),
                top: Val::Percent(40.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ZIndex(300), 
            CombatUiRoot, 
        )).with_children(|banner| {
            banner.spawn((
                Text::new("众 妖 伏 诛"),
                TextFont {
                    font: chinese_font,
                    font_size: 80.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.9, 0.3)), // 亮金色
                crate::components::map::EntranceAnimation::new(0.5),
            ));
        });
    }
}

/// 更新敌人死亡动画
pub fn update_enemy_death_animation(
    mut commands: Commands,
    mut query: Query<(Entity, &mut EnemyDeathAnimation, &mut Sprite)>,
    time: Res<Time>,
) {
    for (entity, mut anim, mut sprite) in query.iter_mut() {
        anim.elapsed += time.delta_secs();
        anim.progress = (anim.elapsed / anim.duration).min(1.0);
        
        info!("死亡动画更新中: 实体={:?}, 进度={:.2}", entity, anim.progress);

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
    particle_query: Query<Entity, With<ParticleMarker>>,
    mouse_button: Res<ButtonInput<MouseButton>>, // 监听鼠标输入
) {
    if !victory_delay.active {
        return;
    }

    // 允许点击跳过：如果按下左键，直接将进度设满
    if mouse_button.just_pressed(MouseButton::Left) {
        victory_delay.elapsed = victory_delay.duration;
        info!("【交互】玩家点击，跳过胜利演出");
    } else {
        victory_delay.elapsed += time.delta_secs();
    }

    // 只在激活时输出日志
    if victory_delay.elapsed < victory_delay.duration {
        info!("胜利延迟进行中: {:.2}/{:.2}", victory_delay.elapsed, victory_delay.duration);
    }

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
        
        // 额外清理所有剩余粒子
        for entity in particle_query.iter() {
            if let Some(mut e) = commands.get_entity(entity) {
                e.despawn_recursive();
            }
        }

        // 最后切换状态
        next_state.set(GameState::Reward);
        info!("已切换到 Reward 状态");
    }
}

// ============================================================================
// 奖励系统
// ============================================================================

/// 设置机缘奖励界面
fn setup_reward_ui(
    mut commands: Commands, 
    asset_server: Res<AssetServer>, 
    relic_collection: Res<RelicCollection>, 
    mut reward_cards_resource: ResMut<CurrentRewardCards>, 
    mut reward_relic_resource: ResMut<CurrentRewardRelic>,
    player_query: Query<&Player>,
) {
    info!("【天道机缘】展现机缘界面");

    let reward_cards = CardPool::random_rewards(3);
    reward_cards_resource.cards = reward_cards.clone();

    let relic_reward = generate_relic_reward(&relic_collection);
    let show_relic = relic_reward.is_some();
    reward_relic_resource.relic = relic_reward.clone();

    let chinese_font: Handle<Font> = asset_server.load("fonts/Arial Unicode.ttf");
    let player_gold = player_query.get_single().map(|p| p.gold).unwrap_or(0);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(40.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.01, 0.01, 0.03)), // 深邃虚空背景
            RewardUiRoot,
        ))
        .with_children(|parent| {
            // 标题区
            parent.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(10.0),
                ..default()
            }).with_children(|header| {
                header.spawn((
                    Text::new("天 道 机 缘"),
                    TextFont { font: chinese_font.clone(), font_size: 56.0, ..default() },
                    TextColor(Color::srgb(0.9, 0.8, 0.4)), // 金色标题
                ));
                header.spawn((
                    Text::new("冥冥之中，自有天定。择一而行，莫失机缘。"),
                    TextFont { font: chinese_font.clone(), font_size: 18.0, ..default() },
                    TextColor(Color::srgb(0.6, 0.6, 0.6)),
                ));
            });

            // 奖励卡牌区
            parent.spawn(Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(40.0),
                ..default()
            }).with_children(|card_area| {
                for (index, card) in reward_cards.iter().enumerate() {
                    create_reward_card(card_area, card, index, &asset_server);
                }
            });

            // 底部操作区
            parent.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
                ..default()
            }).with_children(|footer| {
                // 灵石显示
                footer.spawn((
                    Text::new(format!("持有灵石: {}", player_gold)),
                    TextFont { font: chinese_font.clone(), font_size: 20.0, ..default() },
                    TextColor(Color::srgb(1.0, 0.8, 0.2)),
                ));

                // 放弃按钮
                footer.spawn((
                    Button,
                    Node {
                        width: Val::Px(180.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.5)),
                    BorderRadius::all(Val::Px(5.0)),
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new("因缘未至 (离去)"),
                        TextFont { font: chinese_font.clone(), font_size: 18.0, ..default() },
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    ));
                })
                .observe(|_trigger: Trigger<Pointer<Click>>, mut next_state: ResMut<NextState<GameState>>, mut map_progress: ResMut<MapProgress>| {
                    info!("【机缘】玩家选择放弃，因缘未至");
                    map_progress.complete_current_node();
                    next_state.set(GameState::Map);
                });
            });
        });
}

/// 创建单张奖励卡UI
fn create_reward_card(parent: &mut ChildBuilder, card: &Card, _index: usize, asset_server: &AssetServer) {
    let chinese_font = asset_server.load("fonts/Arial Unicode.ttf");
    let card_color = match card.card_type {
        CardType::Attack => Color::srgba(0.15, 0.05, 0.05, 0.9),
        CardType::Defense => Color::srgba(0.05, 0.05, 0.15, 0.9),
        CardType::Skill => Color::srgba(0.05, 0.15, 0.05, 0.9),
        CardType::Power => Color::srgba(0.15, 0.05, 0.15, 0.9),
        CardType::Curse => Color::srgba(0.1, 0.1, 0.1, 0.9),
    };
    let rarity_color = match card.rarity {
        CardRarity::Common => Color::srgb(0.6, 0.6, 0.6),
        CardRarity::Uncommon => Color::srgb(0.2, 0.8, 0.6),
        CardRarity::Rare => Color::srgb(0.9, 0.7, 0.2),
        CardRarity::Special => Color::srgb(0.9, 0.3, 0.9),
    };
    parent.spawn((
        Button,
        BackgroundColor(card_color),
        BorderColor(rarity_color),
        Node {
            width: Val::Px(200.0), height: Val::Px(280.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(15.0)),
            row_gap: Val::Px(12.0),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BorderRadius::all(Val::Px(8.0)),
        RewardCardButton { card_id: card.id },
    )).with_children(|parent| {
        parent.spawn((Text::new(card.rarity.get_chinese_name()), TextFont { font: chinese_font.clone(), font_size: 16.0, ..default() }, TextColor(rarity_color)));
        parent.spawn((Text::new(card.name.clone()), TextFont { font: chinese_font.clone(), font_size: 24.0, ..default() }, TextColor(Color::WHITE)));
        parent.spawn((Text::new(format!("灵力消耗: {}", card.cost)), TextFont { font: chinese_font.clone(), font_size: 16.0, ..default() }, TextColor(Color::srgb(0.4, 0.8, 1.0))));
        
        // 卡牌插画 (Nano Banana 风格优化)
        parent.spawn((
            ImageNode::new(asset_server.load(card.image_path.clone())),
            Node {
                width: Val::Px(160.0),
                height: Val::Px(120.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BorderColor(Color::srgba(1.0, 1.0, 1.0, 0.2)),
        ));

        parent.spawn((Text::new(card.description.clone()), TextFont { font: chinese_font.clone(), font_size: 14.0, ..default() }, TextColor(Color::srgb(0.8, 0.8, 0.8)), TextLayout::new_with_justify(JustifyText::Center), Node { max_width: Val::Px(170.0), ..default() }));
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
    mut hovered_card: ResMut<HoveredCard>, // 增加资源清理
    mut hovered_relic: ResMut<HoveredRelic>,
) {
    info!("【清理奖励界面】清理所有奖励相关UI");

    // 1. 首先清理逻辑状态，防止竞争
    hovered_card.card_id = None;
    hovered_relic.relic_id = None;

    // 2. 然后清理实体
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

fn handle_reward_clicks(
    interactions: Query<
        (&Interaction, &RewardCardButton),
        (Changed<Interaction>, With<RewardCardButton>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut player_deck: ResMut<PlayerDeck>,
    mut map_progress: ResMut<MapProgress>,
    relics: Res<RelicCollection>, // 引入法宝资源
    player_query: Query<(&Player, &crate::components::Cultivation)>, // 引入玩家查询
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

            // --- [优化] 移除同步阻塞存档，交给状态机自然同步或异步处理 ---
            // 之前的同步 save_to_disk 是导致卡死的嫌疑点，因为下一帧会立即执行大规模 UI 销毁

            // 返回地图
            next_state.set(GameState::Map);
            return; // 立即返回，防止处理同一帧的其他点击
        }
    }
}

/// 处理牌组查看交互
fn handle_deck_view_toggle(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    deck: Res<PlayerDeck>,
    view_btn_query: Query<&Interaction, (Changed<Interaction>, With<crate::components::cards::ViewDeckButton>)>,
    close_btn_query: Query<&Interaction, (Changed<Interaction>, With<crate::components::cards::CloseDeckButton>)>,
    deck_ui_query: Query<Entity, With<crate::components::cards::DeckUiRoot>>,
) {
    let chinese_font = asset_server.load("fonts/Arial Unicode.ttf");

    // 1. 处理打开面板
    for interaction in view_btn_query.iter() {
        if matches!(interaction, Interaction::Pressed) {
            info!("【藏经阁】开启功法查看面板");
            
            // 创建全屏半透明遮罩
            commands.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
                ZIndex(1000), // 覆盖一切
                crate::components::cards::DeckUiRoot,
            )).with_children(|parent| {
                // 标题
                parent.spawn((
                    Text::new("藏 经 阁 (功法一览)"),
                    TextFont { font: chinese_font.clone(), font_size: 48.0, ..default() },
                    TextColor(Color::srgb(0.6, 0.9, 0.6)),
                    Node { margin: UiRect::bottom(Val::Px(30.0)), ..default() },
                ));

                // 滚动区域容器
                parent.spawn(Node {
                    width: Val::Percent(85.0),
                    height: Val::Percent(70.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(20.0),
                    row_gap: Val::Px(25.0),
                    ..default()
                }).with_children(|grid| {
                    // 渲染所有已掌握的功法
                    for card in &deck.cards {
                        create_static_card_ui(grid, card, &chinese_font, &asset_server);
                    }
                });

                // 关闭按钮
                parent.spawn((
                    Button,
                    Node {
                        width: Val::Px(150.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(30.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.4, 0.2, 0.2)),
                    BorderRadius::all(Val::Px(8.0)),
                    crate::components::cards::CloseDeckButton,
                )).with_children(|btn| {
                    btn.spawn((
                        Text::new("合上经卷"),
                        TextFont { font: chinese_font.clone(), font_size: 20.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });
            });
        }
    }

    // 2. 处理关闭面板
    for interaction in close_btn_query.iter() {
        if matches!(interaction, Interaction::Pressed) {
            for entity in deck_ui_query.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

/// 辅助函数：创建一个不可交互的展示用卡牌 UI
fn create_static_card_ui(parent: &mut ChildBuilder, card: &crate::components::cards::Card, font: &Handle<Font>, asset_server: &AssetServer) {
    parent.spawn((
        Node {
            width: Val::Px(120.0),
            height: Val::Px(170.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::all(Val::Px(8.0)),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(card.get_color()),
        BorderColor(Color::BLACK),
        BorderRadius::all(Val::Px(8.0)),
    )).with_children(|card_ui| {
        // 名称
        card_ui.spawn((
            Text::new(card.name.clone()),
            TextFont { font: font.clone(), font_size: 16.0, ..default() },
            TextColor(Color::WHITE),
        ));
        
        // 插画
        card_ui.spawn((
            ImageNode::new(asset_server.load(card.image_path.clone())),
            Node { width: Val::Percent(95.0), height: Val::Percent(55.0), ..default() },
        ));

        // 描述
        card_ui.spawn((
            Text::new(card.description.clone()),
            TextFont { font: font.clone(), font_size: 10.0, ..default() },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            Node { max_width: Val::Px(100.0), ..default() },
        ));
    });
}

#[derive(Component)]
struct GameOverUiRoot;

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
        .observe(move |_entity: Trigger<Pointer<Click>>, 
                       mut relic_collection: ResMut<RelicCollection>, 
                       mut next_state: ResMut<NextState<GameState>>, 
                       mut map_progress: ResMut<MapProgress>,
                       player_deck: Res<PlayerDeck>,
                       player_query: Query<(&Player, &crate::components::Cultivation)>| {
            info!("获得了法宝: {}", relic.name);
            let added = relic_collection.add_relic(relic.clone());
            if added {
                info!("法宝已加入收藏");
            }
            
            // 标记当前节点为完成
            map_progress.complete_current_node();
            
            // --- [优化] 移除同步阻塞存档，防止 UI 销毁时的竞态卡死 ---
            // 存档将由状态机在进入 Map 状态时自动处理

            next_state.set(GameState::Map);
        });
}

/// 处理游戏结束界面按钮点击
fn handle_game_over_clicks(
    mut next_state: ResMut<NextState<GameState>>,
    mut player_query: Query<&mut Player>,
    mut cultivation_query: Query<&mut crate::components::Cultivation>,
    mut deck: ResMut<PlayerDeck>,
    mut relics: ResMut<RelicCollection>,
    mut map_progress: ResMut<MapProgress>, // 新增
    button_queries: Query<(&Interaction, &RestartButton), Changed<Interaction>>,
) {
    for (interaction, _) in button_queries.iter() {
        if matches!(interaction, Interaction::Pressed) {
            info!("【游戏结束】点击重新开始，正在重塑道基...");
            
            // 1. 彻底删除旧存档
            crate::resources::save::GameStateSave::delete_save();

            // 2. 重置玩家数值
            if let Ok(mut player) = player_query.get_single_mut() {
                *player = Player::default(); // 恢复 80 HP
            }
            
            // 3. 重置修为
            if let Ok(mut cultivation) = cultivation_query.get_single_mut() {
                *cultivation = crate::components::Cultivation::new();
            }

            // 4. 重置牌组与遗物
            deck.reset(); 
            relics.relic.clear();
            relics.add_relic(crate::components::relic::Relic::burning_blood());

            // 5. 重置地图进度 (关键修复)
            map_progress.reset();

            next_state.set(GameState::Prologue);
            return;
        }
    }
}

// ============================================================================
// 序章系统 (Prologue)
// ============================================================================

#[derive(Component)]
struct PrologueUiMarker;

#[derive(Component)]
struct PrologueText;

fn setup_prologue(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    info!("序章：开启九界传说...");

    // 初始化序章台词
    let dialogue = crate::components::dialogue::Dialogue::new(vec![
        DialogueLine::new("世界", "混沌初开，九界并立..."),
        DialogueLine::new("世界", "然万载光阴，末法降临，灵气枯竭。"),
        DialogueLine::new("世界", "诸神陨落，众生涂炭，九界命悬一线。"),
        DialogueLine::new("天道", "你，本是凡间一缕残魂..."),
        DialogueLine::new("天道", "唯有逆天渡劫，重铸金身，方能挽狂澜于既倒。"),
        DialogueLine::new("天道", "九界门启，渡劫开始！"),
    ]);

    commands.insert_resource(dialogue.clone());

    // 创建纯黑背景
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::BLACK),
        PrologueUiMarker,
    )).with_children(|parent| {
        // 台词文本
        parent.spawn((
            Text::new(dialogue.current_line().unwrap().text.clone()),
            TextFont {
                font: asset_server.load("fonts/Arial Unicode.ttf"),
                font_size: 40.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            PrologueText,
        ));

        // 提示文本
        parent.spawn((
            Text::new("—— 点击屏幕继续 ——"),
            TextFont {
                font: asset_server.load("fonts/Arial Unicode.ttf"),
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.4)),
            Node {
                margin: UiRect::top(Val::Px(50.0)),
                ..default()
            },
        ));
    });
}

fn update_prologue(
    mut next_state: ResMut<NextState<GameState>>,
    mut dialogue: ResMut<crate::components::dialogue::Dialogue>,
    mut query: Query<&mut Text, With<PrologueText>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
) {
    // 检测点击 (鼠标或触摸)
    if mouse_button.just_pressed(MouseButton::Left) || touches.any_just_pressed() {
        dialogue.next();

        if dialogue.is_finished() {
            info!("序章播放结束，进入地图");
            next_state.set(GameState::Map);
        } else if let Some(line) = dialogue.current_line() {
            if let Ok(mut text) = query.get_single_mut() {
                text.0 = line.text.clone();
            }
        }
    }
}

fn cleanup_prologue(
    mut commands: Commands,
    query: Query<Entity, With<PrologueUiMarker>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<crate::components::dialogue::Dialogue>();
}


#[derive(Component)]
struct TribulationUiMarker;

fn setup_tribulation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut timer: ResMut<TribulationTimer>,
    mut screen_events: EventWriter<ScreenEffectEvent>,
) {
    info!("🌩️ 天地震动，雷劫将至！");
    timer.total_timer.reset();
    timer.strike_timer.reset();
    timer.strikes_count = 0;

    let chinese_font = asset_server.load("fonts/Arial Unicode.ttf");

    // 创建渡劫背景（半透明深紫色遮罩，允许 3D 特效透出）
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.01, 0.0, 0.02, 0.65)),
        ZIndex(-10), 
        TribulationUiMarker,
    ));

    // 渡劫标题与倒计时容器
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            position_type: PositionType::Absolute,
            ..default()
        },
        ZIndex(50), // 文字悬浮
        TribulationUiMarker,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("正在渡劫..."),
            TextFont {
                font: chinese_font.clone(),
                font_size: 72.0,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 1.0)),
        ));

        parent.spawn((
            Text::new("承受九天雷霆洗礼，存活即成大道"),
            TextFont {
                font: chinese_font.clone(),
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::srgb(0.6, 0.6, 0.8)),
        ));
    });

    // 初始屏幕闪光（紫色）
    screen_events.send(ScreenEffectEvent::Flash { 
        color: Color::srgba(0.5, 0.2, 0.8, 0.5), 
        duration: 0.5 
    });
}

fn update_tribulation(
    time: Res<Time>,
    mut timer: ResMut<TribulationTimer>,
    mut player_query: Query<&mut Player>,
    mut next_state: ResMut<NextState<GameState>>,
    mut screen_events: EventWriter<ScreenEffectEvent>,
    mut effect_events: EventWriter<SpawnEffectEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    // 持续环境氛围演出：轻微震动与灵气扰动
    screen_events.send(ScreenEffectEvent::Shake { trauma: 0.05, decay: 10.0 });
    if time.elapsed_secs() as u32 % 2 == 0 {
        effect_events.send(SpawnEffectEvent::new(EffectType::AmbientSpirit, Vec3::new(0.0, 0.0, 5.0)).burst(2));
    }

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
        
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        if let Ok(mut player) = player_query.get_single_mut() {
            // 天雷伤害
            let damage = (player.max_hp as f32 * 0.12).max(10.0) as i32;
            player.hp -= damage;
            
            info!("⚡ 第 {} 道天雷落下！造成 {} 点伤害，剩余道行: {}", timer.strikes_count, damage, player.hp);

            // 播放天雷音效
            sfx_events.send(PlaySfxEvent::new(SfxType::LightningStrike));

            // 视觉特效：强力白光闪烁 + 剧烈震动
            screen_events.send(ScreenEffectEvent::Flash { 
                color: Color::srgba(1.0, 1.0, 1.0, 0.8), // 提高不透明度
                duration: 0.15 
            });
            screen_events.send(ScreenEffectEvent::Shake { 
                trauma: 25.0, // 显著增加震动强度
                decay: 0.5 
            });
            
            // [关键修复] 使用真实的 3D 世界坐标 (而非 2D 像素坐标)
            // 确保雷霆劈在 3D 摄像机的视野范围内 (X: -10..10, Z: -5..5)
            for _ in 0..3 {
                let x = rng.gen_range(-12.0..12.0);
                let z = rng.gen_range(-8.0..8.0);
                // 3D 空间的落点 Y 轴应为 0 附近
                effect_events.send(SpawnEffectEvent::new(EffectType::Lightning, Vec3::new(x, 0.0, z)));
            }

            // 增强瞬间白闪强度，覆盖 2D 背景，产生致盲感
            screen_events.send(ScreenEffectEvent::Flash { 
                color: Color::srgba(1.5, 1.5, 2.0, 0.95), 
                duration: 0.12 
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
    mut commands: Commands,
    ui_query: Query<Entity, With<TribulationUiMarker>>,
    mut player_query: Query<(&mut Player, &mut crate::components::Cultivation)>,
    mut deck: ResMut<PlayerDeck>,
    mut map_progress: ResMut<MapProgress>,
    mut effect_events: EventWriter<SpawnEffectEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    // 清理渡劫专用UI
    for entity in ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    if let Ok((mut player, mut cultivation)) = player_query.get_single_mut() {
        // 只有在没死的情况下才处理突破（避免 GameOver 状态逻辑冲突）
        if player.hp > 0 {
            if cultivation.breakthrough() {
                // 1. 属性质变
                let (hp_bonus, stone_bonus) = match cultivation.realm {
                    crate::components::cultivation::Realm::FoundationEstablishment => (50, 100),
                    crate::components::cultivation::Realm::GoldenCore => (100, 200),
                    _ => (20, 50),
                };
                player.max_hp += hp_bonus;
                player.hp = player.max_hp; // 状态全回满
                player.gold += stone_bonus; // 天道赏赐灵石
                
                info!("✨【破境成功】成功晋升至 {:?}！道行大进，上限增加 {} 点，获灵石 {} 块", cultivation.realm, hp_bonus, stone_bonus);

                // --- 2. 开启新征程：重新生成地图 ---
                let map_config = crate::components::map::MapConfig::default();
                map_progress.nodes = crate::components::map::generate_map_nodes(&map_config, 0);
                map_progress.current_node_id = None;
                map_progress.current_layer = 0;
                map_progress.refresh_unlocks();
                info!("🗺️【天道演化】新的地图已生成，开启下一境界的修行！");
                
                // 3. 功法质变：发放本命功法
                if cultivation.realm == crate::components::cultivation::Realm::FoundationEstablishment {
                    let innate_spell = crate::components::cards::CardPool::get_innate_spell();
                    deck.add_card(innate_spell.clone());
                    info!("📖【本命功法】获得筑基期本命功法：{}", innate_spell.name);
                } else if cultivation.realm == crate::components::cultivation::Realm::GoldenCore {
                    // 金丹期自动领悟万剑归宗
                    let aoe_spell = Card::new(
                        151, "万剑归宗", "金丹大能之怒！对全场造成10点伤害",
                        CardType::Attack, 2, CardEffect::DealAoEDamage { amount: 10 },
                        CardRarity::Rare,
                        "textures/cards/attack.png"
                    );
                    deck.add_card(aoe_spell.clone());
                    info!("📖【大能神通】晋升金丹，领悟群体攻伐：{}", aoe_spell.name);
                }

                // 3. 视听反馈
                sfx_events.send(PlaySfxEvent::new(SfxType::BreakthroughSuccess));
                effect_events.send(SpawnEffectEvent::new(EffectType::Victory, Vec3::new(0.0, 0.0, 999.0)).burst(100));
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
    next_state: Res<NextState<GameState>>, // 增加状态检查
) {
    // 如果状态即将切换，不要再处理悬停逻辑
    // 如果状态即将切换，不要再处理悬停逻辑
    if !matches!(next_state.as_ref(), NextState::Unchanged) { return; }

    for (interaction, card_button) in interactions.iter() {
        match interaction {
            Interaction::Hovered => {
                // 如果 ID 没变，说明已经显示了，跳过重建防止闪烁
                if hovered_card.card_id == Some(card_button.card_id) {
                    continue;
                }

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
    next_state: Res<NextState<GameState>>,
) {
    // 如果状态即将切换，不要再处理悬停逻辑
    if !matches!(next_state.as_ref(), NextState::Unchanged) { return; }

    for (interaction, relic_button) in interactions.iter() {
        match interaction {
            Interaction::Hovered => {
                // 防止重复重建
                if hovered_relic.relic_id == Some(relic_button.relic_id) {
                    continue;
                }

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
    // ... (前略：颜色计算逻辑保持不变)
    let card_color = match card.card_type {
        CardType::Attack => Color::srgb(0.8, 0.2, 0.2),
        CardType::Defense => Color::srgb(0.2, 0.5, 0.8),
        CardType::Skill => Color::srgb(0.2, 0.7, 0.3),
        CardType::Power => Color::srgb(0.7, 0.3, 0.8),
        CardType::Curse => Color::srgb(0.3, 0.3, 0.3),
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
    if y + 400.0 > WINDOW_HEIGHT {  // 假设面板高度约400px (包含插画)
        y = mouse_pos.y - 400.0 - OFFSET_Y;
    }

    let (position_left, position_right) = if x + PANEL_WIDTH > WINDOW_WIDTH {
        (None, Some(Val::Px(WINDOW_WIDTH - x)))
    } else {
        (Some(Val::Px(x)), None)
    };

    let (position_top, position_bottom) = if y + 400.0 > WINDOW_HEIGHT {
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

    if let Some(left) = position_left { node.left = left; }
    if let Some(right) = position_right { node.right = right; }
    if let Some(top) = position_top { node.top = top; }
    if let Some(bottom) = position_bottom { node.bottom = bottom; }

    commands
        .spawn((
            node,
            BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.95)),
            BorderColor(rarity_color),
            CardHoverPanelMarker,
            ZIndex(1000), // 确保在最上层
            CombatUiRoot, // 标记为战斗 UI，支持自动清理
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

            // 卡牌插画 (Nano Banana 风格优化)
            parent.spawn((
                ImageNode::new(asset_server.load(card.image_path.clone())),
                Node {
                    width: Val::Px(270.0),
                    height: Val::Px(180.0),
                    border: UiRect::all(Val::Px(1.0)),
                    margin: UiRect::vertical(Val::Px(5.0)),
                    ..default()
                },
                BorderColor(Color::srgba(1.0, 1.0, 1.0, 0.2)),
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
            ZIndex(1000), // 确保在最上层
            CombatUiRoot, // 标记为战斗 UI
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
