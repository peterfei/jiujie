//! 集成测试辅助工具
//!
//! 提供设置 Bevy App 测试环境的通用功能

use bevy::prelude::*;
use bevy_card_battler::plugins::{CorePlugin, MenuPlugin};
use bevy_card_battler::systems::{AnimationPlugin, SpritePlugin, ParticlePlugin, ScreenEffectPlugin};
use bevy_card_battler::states::GameState;
use bevy_card_battler::components::{VictoryDelay, Enemy, Player, PlayerDeck, MapProgress, CombatConfig};

/// 创建用于测试的 Bevy App
///
/// 包含所有必要的插件和最小资源配置
pub fn create_test_app() -> App {
    let mut app = App::new();

    // 使用无头模式运行，避免窗口创建问题
    app.add_plugins(MinimalPlugins)
        // 添加资产插件（无头模式）
        .add_plugins(AssetPlugin::default())
        // 添加状态插件
        .add_plugins(bevy::state::app::StatesPlugin);

    // 手动初始化需要的资产类型
    app.init_asset::<Image>();
    app.init_asset::<Font>();

    // 初始化游戏状态
    app.init_state::<GameState>();

    // 注册核心插件
    app.add_plugins(CorePlugin);
    app.add_plugins(MenuPlugin);

    // 注册特效插件
    app.add_plugins(AnimationPlugin);
    app.add_plugins(SpritePlugin);
    app.add_plugins(ParticlePlugin);
    app.add_plugins(ScreenEffectPlugin);

    // 设置测试资源（不包含 Player，它是 Component）
    app.insert_resource(VictoryDelay::new(0.8));
    app.insert_resource(PlayerDeck::default());
    app.insert_resource(MapProgress::default());
    app.insert_resource(CombatConfig::default());

    app
}

/// 设置战斗场景
///
/// 创建玩家实体，设置Combat状态（不创建敌人，使用 setup_combat_ui 创建的敌人）
///
/// 注意：由于 setup_combat_ui 在进入 Combat 状态时会创建一个敌人（HP: 30），
/// 我们不在这里创建额外的敌人。测试应该找到并杀死那个敌人。
pub fn setup_combat_scene(app: &mut App) -> Entity {
    // 创建玩家实体（check_combat_end 需要同时存在 Player 和 Enemy）
    app.world_mut().spawn(Player::default());

    // 切换到Combat状态（这会触发 setup_combat_ui，它创建敌人）
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);

    // 运行状态转换 schedule 以应用状态更改
    app.world_mut().run_schedule(StateTransition);

    // 查找并返回 setup_combat_ui 创建的敌人实体
    let world = app.world_mut();
    let mut enemy_query = world.query::<(Entity, &Enemy)>();
    let enemy_entity = enemy_query.get_single(world).unwrap().0;

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
