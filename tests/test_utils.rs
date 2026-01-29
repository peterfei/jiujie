//! 集成测试辅助工具
//!
//! 提供设置 Bevy App 测试环境的通用功能
//!
//! # 测试继承规范 (commit 2a818d5)
//!
//! 本模块是项目集成测试的标准框架。所有新的集成测试都应该：
//! 1. 使用 `use crate::test_utils::*;` 导入本模块
//! 2. 使用 `create_test_app()` 创建测试环境
//! 3. 使用提供的场景设置函数
//! 4. 遵循现有的测试模式
//!
//! # 扩展指南
//!
//! 如果需要新的辅助函数：
//! - 首先考虑能否扩展现有函数
//! - 如确需新函数，添加到本模块并更新文档
//! - 保持函数职责单一，便于复用

use bevy::prelude::*;
use bevy::text::TextPlugin;
use bevy_card_battler::plugins::{CorePlugin, MenuPlugin};
use bevy_card_battler::components::sprite::CharacterAssets;
use bevy_card_battler::systems::{AnimationPlugin, SpritePlugin, ParticlePlugin, ScreenEffectPlugin, ShopPlugin, RestPlugin, MapPlugin};
use bevy_card_battler::states::GameState;
use bevy_card_battler::components::{VictoryDelay, Enemy, Player, PlayerDeck, MapProgress, CombatConfig, RelicCollection};
use bevy_card_battler::components::shop::CurrentShopItems;
use bevy_card_battler::components::particle::SpawnEffectEvent;
use bevy_card_battler::components::screen_effect::ScreenEffectEvent;
use bevy_card_battler::components::EnemyAttackEvent;

/// 创建用于测试的 Bevy App
///
/// 包含所有必要的插件和最小资源配置
///
/// # 示例
///
/// ```rust
/// let mut app = create_test_app();
/// advance_frames(&mut app, 1);
/// ```
pub fn create_test_app() -> App {
    let mut app = App::new();

    // 使用无头模式运行，避免窗口创建问题
    app.add_plugins(MinimalPlugins)
        // 添加资产插件（无头模式）
        .add_plugins(AssetPlugin::default())
        .add_plugins(TextPlugin::default())
        // 添加状态插件
        .add_plugins(bevy::state::app::StatesPlugin);

    // 手动初始化需要的资产类型
    app.init_asset::<Image>();
    app.init_asset::<Font>();
    app.init_asset::<TextureAtlasLayout>();
    app.init_asset::<Mesh>();
    app.init_asset::<StandardMaterial>();

    // 初始化游戏状态
    app.init_state::<GameState>();

    // 添加所需的事件
    app.add_event::<SpawnEffectEvent>();
    app.add_event::<ScreenEffectEvent>();
    app.add_event::<EnemyAttackEvent>();
    app.add_event::<bevy_card_battler::components::PlaySfxEvent>();
    app.add_event::<bevy_card_battler::components::DamageEffectEvent>();
    app.add_event::<bevy_card_battler::components::StatusEffectEvent>();

    // 注册核心插件
    app.add_plugins(CorePlugin);
    app.add_plugins(MenuPlugin);
    app.add_plugins(ShopPlugin);
    app.add_plugins(RestPlugin);
    app.add_plugins(MapPlugin);

    // 注册特效插件
    app.add_plugins(AnimationPlugin);
    app.add_plugins(SpritePlugin);
    app.add_plugins(ParticlePlugin);
    app.add_plugins(ScreenEffectPlugin);

    // 设置测试资源
    app.insert_resource(bevy_card_battler::components::CombatState::default());
    app.insert_resource(VictoryDelay::new(0.1));
    app.insert_resource(CharacterAssets::default());
    app.insert_resource(bevy_card_battler::components::Environment::default());
    app.insert_resource(Player::default());
    app.insert_resource(PlayerDeck::default());
    app.insert_resource(MapProgress::default());
    app.insert_resource(CombatConfig::default());
    app.insert_resource(RelicCollection::default());

    // 初始化输入资源
    app.init_resource::<ButtonInput<KeyCode>>();
    app.init_resource::<ButtonInput<MouseButton>>();

    app
}

/// 设置战斗场景
///
/// 切换到Combat状态（init_player 会创建玩家实体，setup_combat_ui 会创建敌人）
///
/// 注意：init_player 系统会确保玩家实体存在
/// setup_combat_ui 会创建敌人（1-3个）
pub fn setup_combat_scene(app: &mut App) -> Entity {
    // 切换到Combat状态（init_player 会创建玩家，setup_combat_ui 会创建敌人）
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);

    // 运行状态转换 schedule 以应用状态更改
    app.world_mut().run_schedule(StateTransition);

    // 查找并返回 setup_combat_ui 创建的第一个敌人实体
    let world = app.world_mut();
    let mut enemy_query = world.query::<(Entity, &Enemy)>();
    let enemy_entity = enemy_query.iter(world).next().expect("战斗场景初始化失败：找不到敌人实体").0;

    enemy_entity
}

/// 模拟敌人受到致命伤害
///
/// 对敌人实体造成超过其生命值的伤害
pub fn kill_enemy(app: &mut App, enemy_entity: Entity) {
    if let Some(mut enemy) = app.world_mut().get_mut::<Enemy>(enemy_entity) {
        enemy.take_damage(999);
    }
}

/// 模拟时间流逝
///
/// 运行指定次数的帧更新
pub fn advance_frames(app: &mut App, frame_count: u32) {
    for _ in 0..frame_count {
        // 手动推进时间，以确保依赖时间的系统正常运行
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(std::time::Duration::from_millis(16));
        
        app.update();
    }
}

/// 模拟时间流逝（秒）
///
/// 假设每秒60帧
pub fn advance_seconds(app: &mut App, seconds: f32) {
    let frame_count = (seconds * 60.0) as u32;
    advance_frames(app, frame_count);
}

/// 验证粒子实体数量
pub fn count_particles(app: &mut App) -> usize {
    let world = app.world_mut();
    world.query_filtered::<Entity, With<bevy_card_battler::components::ParticleMarker>>()
        .iter(world)
        .count()
}

/// 验证发射器实体数量
pub fn count_emitters(app: &mut App) -> usize {
    let world = app.world_mut();
    world.query_filtered::<Entity, With<bevy_card_battler::components::EmitterMarker>>()
        .iter(world)
        .count()
}

/// 验证屏幕效果实体数量
pub fn count_screen_effects(app: &mut App) -> usize {
    let world = app.world_mut();
    world.query_filtered::<Entity, With<bevy_card_battler::components::ScreenEffectMarker>>()
        .iter(world)
        .count()
}

/// 获取当前游戏状态
///
/// 在 Bevy 0.15 中，状态通过 State 资源访问
pub fn get_current_state(app: &App) -> GameState {
    match app.world().get_resource::<State<GameState>>() {
        Some(state) => *state.get(),
        None => GameState::default(),
    }
}

/// 检查胜利延迟是否激活
pub fn is_victory_delay_active(app: &App) -> bool {
    app.world().resource::<VictoryDelay>().active
}

/// 获取胜利延迟经过时间
pub fn get_victory_delay_elapsed(app: &App) -> f32 {
    app.world().resource::<VictoryDelay>().elapsed
}

// ============================================================================
// 商店测试辅助函数
// ============================================================================

/// 设置商店场景
///
/// 切换到商店状态（init_player 会创建玩家实体）
///
/// # 示例
///
/// ```rust
/// let mut app = create_test_app();
/// setup_shop_scene(&mut app);
/// advance_frames(&mut app, 1);
///
/// // 验证商店UI已创建
/// let shop_ui_count = count_shop_ui(&app);
/// assert!(shop_ui_count > 0);
/// ```
pub fn setup_shop_scene(app: &mut App) {
    // 初始化商店物品资源
    app.world_mut().insert_resource(CurrentShopItems::default());

    // 切换到商店状态（init_player 会创建玩家实体）
    let world = app.world_mut();
    if let Some(mut next_state) = world.get_resource_mut::<NextState<GameState>>() {
        next_state.set(GameState::Shop);
    }
}

/// 统计商店UI根节点数量
pub fn count_shop_ui(app: &mut App) -> usize {
    use bevy_card_battler::components::shop::ShopUiRoot;
    let mut query = app.world_mut().query::<(Entity, &ShopUiRoot)>();
    query.iter(&app.world()).count()
}

/// 统计商店卡牌按钮数量
pub fn count_shop_card_buttons(app: &mut App) -> usize {
    use bevy_card_battler::components::shop::ShopCardButton;
    let mut query = app.world_mut().query::<(Entity, &ShopCardButton)>();
    query.iter(&app.world()).count()
}

/// 获取玩家当前金币
pub fn get_player_gold(app: &mut App) -> i32 {
    let mut query = app.world_mut().query::<&Player>();
    query.iter(&app.world()).next().map(|p| p.gold).unwrap_or(0)
}

// ============================================================================
// 状态转换辅助函数
// ============================================================================

/// 切换到指定状态
///
/// # 示例
///
/// ```rust
/// transition_to_state(&mut app, GameState::Combat);
/// advance_frames(&mut app, 1);
/// ```
pub fn transition_to_state(app: &mut App, state: GameState) {
    let world = app.world_mut();
    if let Some(mut next_state) = world.get_resource_mut::<NextState<GameState>>() {
        next_state.set(state);
    }
}
