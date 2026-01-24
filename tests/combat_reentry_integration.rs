//! 战斗重入集成测试
//! 模拟用户真实行为：战斗 -> 返回地图 -> 重新进入战斗

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::text::TextPlugin;
use bevy::sprite::TextureAtlasLayout;
use bevy_card_battler::components::*;
use bevy_card_battler::plugins::CorePlugin;
use bevy_card_battler::states::GameState;
use bevy_card_battler::systems::{RelicPlugin, CombatStartProcessed};
use bevy::state::state::NextState;

/// 创建完整测试应用
fn create_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .add_plugins(TextPlugin::default())
        .add_plugins(StatesPlugin)
        .add_plugins(CorePlugin)
        .add_plugins(RelicPlugin)
        .init_state::<GameState>()
        .init_asset::<Font>()
        .init_asset::<Image>()
        .init_asset::<TextureAtlasLayout>();

    let map_config = MapConfig { layers: 15, nodes_per_layer: 3, node_spacing: 150.0 };
    app.insert_resource(MapProgress::new(&map_config));
    app.insert_resource(PlayerDeck::new());
    app.insert_resource(CombatState::default());

    // 运行 Startup 初始化遗物
    app.update();

    app
}

#[test]
fn test_relic_effects_first_combat() {
    // 测试：第一场战斗中遗物效果正常
    let mut app = create_test_app();

    info!("=== 第一场战斗 ===");
    let enemy_hp_before = 30;
    app.world_mut().spawn(Enemy::new(0, "敌人1", enemy_hp_before));

    app.insert_state(GameState::Combat);
    app.update();
    app.update();

    let enemy = app.world_mut().query::<&Enemy>().single(app.world());
    let damage = enemy_hp_before - enemy.hp;
    info!("第一场战斗：敌人受到 {} 点伤害", damage);
    assert_eq!(damage, 3, "第一场战斗应该有3点遗物伤害");
}

#[test]
fn test_combat_to_map_and_back_to_combat() {
    // 测试：战斗 -> 返回地图 -> 重新进入战斗，遗物效果应该仍然生效
    let mut app = create_test_app();

    // === 第一场战斗 ===
    info!("=== 第一场战斗 ===");
    let enemy1_hp_before = 30;
    app.world_mut().spawn(Enemy::new(0, "敌人1", enemy1_hp_before));

    app.insert_state(GameState::Combat);
    app.update();
    app.update();

    let enemy1 = app.world_mut().query::<&Enemy>().single(app.world());
    let damage1 = enemy1_hp_before - enemy1.hp;
    info!("第一场战斗：敌人受到 {} 点伤害", damage1);
    assert_eq!(damage1, 3, "第一场战斗应该有3点遗物伤害");

    // 检查 CombatStartProcessed 标志
    let processed_after_first_combat = app.world()
        .get_resource::<CombatStartProcessed>()
        .unwrap()
        .processed;
    info!("第一场战斗后，CombatStartProcessed = {}", processed_after_first_combat);
    assert!(processed_after_first_combat, "第一场战斗后标志应该被设置");

    // === 返回地图 ===
    info!("=== 返回地图 ===");
    // 使用 NextState 触发状态转换（就像游戏中一样）
    app.world_mut().insert_resource(NextState::Pending(GameState::Map));
    app.update(); // 触发状态转换和 OnExit 系统
    app.update(); // 确保所有系统完成

    // 清理敌人实体（模拟退出战斗）
    app.world_mut().clear_entities();

    // 检查标志是否被重置
    let processed_after_map = app.world()
        .get_resource::<CombatStartProcessed>()
        .unwrap()
        .processed;
    info!("返回地图后，CombatStartProcessed = {}", processed_after_map);

    // === 第二场战斗 ===
    info!("=== 第二场战斗（从地图重新进入）===");

    // 检查进入第二场战斗前的状态
    let processed_before_second_combat = app.world()
        .get_resource::<CombatStartProcessed>()
        .unwrap()
        .processed;
    info!("第二场战斗前，CombatStartProcessed = {}", processed_before_second_combat);

    let enemy2_hp_before = 30;
    app.world_mut().spawn(Enemy::new(0, "敌人2", enemy2_hp_before));

    app.insert_state(GameState::Combat);
    app.update();
    app.update();

    let enemy2 = app.world_mut().query::<&Enemy>().single(app.world());
    let damage2 = enemy2_hp_before - enemy2.hp;
    info!("第二场战斗：敌人受到 {} 点伤害", damage2);

    // 这个断言会失败，因为标志没有被重置！
    assert_eq!(damage2, 3, "第二场战斗也应该有3点遗物伤害，但实际: {}", damage2);
}

#[test]
fn test_manual_reset_processed_flag() {
    // 测试：手动重置标志后，遗物效果应该生效
    let mut app = create_test_app();

    // === 第一场战斗 ===
    info!("=== 第一场战斗 ===");
    app.world_mut().spawn(Enemy::new(0, "敌人1", 30));
    app.insert_state(GameState::Combat);
    app.update();
    app.update();

    // === 返回地图 ===
    app.insert_state(GameState::Map);
    app.update();
    app.world_mut().clear_entities();

    // === 手动重置标志（模拟应该发生的行为）===
    info!("=== 手动重置 CombatStartProcessed ===");
    let mut processed = app.world_mut().get_resource_mut::<CombatStartProcessed>().unwrap();
    processed.processed = false;
    info!("已手动重置标志为 false");

    // === 第二场战斗 ===
    info!("=== 第二场战斗 ===");
    app.world_mut().spawn(Enemy::new(0, "敌人2", 30));
    app.insert_state(GameState::Combat);
    app.update();
    app.update();

    let enemy = app.world_mut().query::<&Enemy>().single(app.world());
    let damage = 30 - enemy.hp;
    info!("第二场战斗：敌人受到 {} 点伤害", damage);

    assert_eq!(damage, 3, "重置标志后，遗物效果应该生效");
    info!("✅ 修复方案验证：重置标志后遗物效果恢复正常！");
}
