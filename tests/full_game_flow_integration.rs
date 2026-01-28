//! 完整游戏流程集成测试
//! 验证从游戏开始到获得遗物再到遗物生效的完整流程

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::text::TextPlugin;
use bevy::sprite::TextureAtlasLayout;
use bevy_card_battler::components::*;
use bevy_card_battler::components::relic::{RelicEffect, RelicRarity, RelicId};
use bevy_card_battler::plugins::CorePlugin;
use bevy_card_battler::states::GameState;
use bevy_card_battler::systems::{RelicPlugin, CombatStartProcessed};

#[test]
fn test_full_game_flow_with_starting_relic() {
    // 测试：游戏开始时自动获得初始遗物（燃烧之血）
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

    // 运行 Startup 系统以初始化遗物
    app.update();

    // 检查遗物背包
    let relic_collection = app.world().get_resource::<RelicCollection>().unwrap();
    assert_eq!(relic_collection.count(), 1, "应该有1个初始遗物");
    assert!(relic_collection.has(RelicId::BurningBlood), "应该拥有燃烧之血遗物");
    info!("✅ 测试通过：游戏开始时自动获得初始遗物");

    // 设置敌人并进入战斗
    let enemy_hp_before = 30;
    app.world_mut().spawn(Enemy::new(0, "测试敌人", enemy_hp_before));
    app.insert_state(GameState::Combat);
    app.update();
    app.update();

    // 检查敌人受到遗物伤害
    let enemy = app.world_mut().query::<&Enemy>().single(app.world());
    let damage_dealt = enemy_hp_before - enemy.hp;
    assert_eq!(damage_dealt, 3, "燃烧之血应该对敌人造成3点伤害");
    info!("✅ 测试通过：初始遗物效果生效！敌人受到 {} 点伤害", damage_dealt);
}

#[test]
fn test_multiple_relics_stack_effects() {
    // 测试：多个遗物效果叠加
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

    // 运行 Startup 系统以初始化遗物
    app.update();

    // 第一场战斗 - 只有初始遗物（燃烧之血）
    info!("=== 第一场战斗（只有初始遗物）===");
    let enemy1_hp_before = 30;
    // 玩家实体已由 CorePlugin 自动创建，无需手动 spawn
    app.world_mut().spawn(Enemy::new(0, "敌人1", enemy1_hp_before));
    app.insert_state(GameState::Combat);
    app.update();
    app.update();

    let enemy1 = app.world_mut().query::<&Enemy>().single(app.world());
    let damage1 = enemy1_hp_before - enemy1.hp;
    let player1_block = app.world_mut().query::<&Player>().single(app.world()).block;
    info!("第一场战斗：敌人受到 {} 点伤害，玩家护甲: {}", damage1, player1_block);
    assert_eq!(damage1, 3, "燃烧之血应该造成3点伤害");

    // 清理战斗
    app.world_mut().clear_entities();

    // 添加第二个遗物（石盾 - 战斗开始时获得护甲）
    info!("=== 获得第二个遗物 ===");
    let mut relic_collection = app.world_mut().get_resource_mut::<RelicCollection>().unwrap();
    let stone_shield = Relic {
        id: RelicId::BagOfPreparation, // 使用现有ID
        name: "石盾".to_string(),
        description: "战斗开始时获得5点护甲".to_string(),
        rarity: RelicRarity::Common,
        effects: vec![RelicEffect::OnCombatStart { damage: 0, block: 5, draw_cards: 0 }],
    };
    relic_collection.add_relic(stone_shield);
    info!("遗物数量: {}", relic_collection.count());

    // 重置标志
    let mut processed = app.world_mut().get_resource_mut::<CombatStartProcessed>().unwrap();
    processed.processed = false;

    // 第二场战斗 - 有两个遗物
    info!("=== 第二场战斗（有两个遗物）===");
    let enemy2_hp_before = 30;
    app.world_mut().spawn(Player::default());
    app.world_mut().spawn(Enemy::new(0, "敌人2", enemy2_hp_before));
    app.insert_state(GameState::Combat);
    app.update();
    app.update();

    let enemy2 = app.world_mut().query::<&Enemy>().single(app.world());
    let damage2 = enemy2_hp_before - enemy2.hp;
    let player2_block = app.world_mut().query::<&Player>().single(app.world()).block;
    info!("第二场战斗：敌人受到 {} 点伤害，玩家护甲: {}", damage2, player2_block);
    assert_eq!(damage2, 3, "燃烧之血应该造成3点伤害");
    assert_eq!(player2_block, 5, "石盾应该给予5点护甲");
    info!("✅ 测试通过：多个遗物效果正确叠加！");
}
