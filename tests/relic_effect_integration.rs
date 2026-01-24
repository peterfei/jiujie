//! 遗物效果集成测试
//! 模拟实际战斗场景，验证遗物效果是否正确触发

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::text::TextPlugin;
use bevy::sprite::TextureAtlasLayout;
use bevy_card_battler::components::*;
use bevy_card_battler::components::relic::{RelicEffect, RelicRarity, RelicId};
use bevy_card_battler::plugins::CorePlugin;
use bevy_card_battler::states::GameState;
use bevy_card_battler::systems::RelicPlugin;

/// 创建带遗物的测试应用
fn create_app_with_relic() -> App {
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

    // 初始化资源
    let map_config = MapConfig { layers: 15, nodes_per_layer: 3, node_spacing: 150.0 };
    app.insert_resource(MapProgress::new(&map_config));
    app.insert_resource(PlayerDeck::new());
    app.insert_resource(CombatState::default());

    // 添加一个战斗开始时造成伤害的遗物（燃烧之血）
    let mut relic_collection = RelicCollection::default();
    let damage_relic = Relic {
        id: RelicId::BurningBlood,
        name: "燃烧之血".to_string(),
        description: "战斗开始时对所有敌人造成3点伤害".to_string(),
        rarity: RelicRarity::Common,
        effect: RelicEffect::OnCombatStart {
            damage: 3,
            block: 0,
            draw_cards: 0,
        },
    };
    relic_collection.add_relic(damage_relic);
    app.insert_resource(relic_collection);

    // 进入战斗状态
    app.insert_state(GameState::Combat);
    app
}

#[test]
fn test_relic_damage_on_combat_start() {
    // 测试：战斗开始时，遗物应对敌人造成伤害
    let mut app = create_app_with_relic();

    // 设置敌人
    let enemy_hp_before = 30;
    app.world_mut().spawn(Enemy::new(0, "测试敌人", enemy_hp_before));

    // 运行系统（包括遗物触发）
    app.update();
    app.update(); // 多运行一次确保遗物效果触发

    // 检查敌人是否受到了遗物伤害
    let enemy = app.world_mut().query::<&Enemy>().single(app.world());
    let damage_dealt = enemy_hp_before - enemy.hp;
    info!("敌人初始HP: {}, 当前HP: {}, 受到伤害: {}", enemy_hp_before, enemy.hp, damage_dealt);
    assert_eq!(damage_dealt, 3, "遗物应该对敌人造成3点伤害，实际: {}", damage_dealt);
    assert_eq!(enemy.hp, 27, "敌人剩余HP应该是27，实际: {}", enemy.hp);
}

#[test]
fn test_relic_block_on_combat_start() {
    // 测试：战斗开始时，遗物应给玩家护甲
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

    // 添加一个战斗开始时给护甲的遗物（自定义测试遗物）
    let mut relic_collection = RelicCollection::default();
    let block_relic = Relic {
        id: RelicId::BurningBlood, // 使用现有ID，只测试护甲效果
        name: "石盾".to_string(),
        description: "战斗开始时获得5点护甲".to_string(),
        rarity: RelicRarity::Common,
        effect: RelicEffect::OnCombatStart {
            damage: 0,
            block: 5,
            draw_cards: 0,
        },
    };
    relic_collection.add_relic(block_relic);
    app.insert_resource(relic_collection);

    // 设置玩家
    app.world_mut().spawn(Player::default());

    app.insert_state(GameState::Combat);
    app.update();
    app.update();

    // 检查玩家是否获得了护甲
    let player = app.world_mut().query::<&Player>().single(app.world());
    assert_eq!(player.block, 5, "玩家应该获得5点护甲，实际: {}", player.block);
}

#[test]
fn test_empty_relic_collection_no_effect() {
    // 测试：没有遗物时不应该有任何效果
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
    app.insert_resource(RelicCollection::default()); // 空遗物背包

    // 设置玩家和敌人
    app.world_mut().spawn(Player::default());
    let enemy_hp_before = 30;
    app.world_mut().spawn(Enemy::new(0, "测试敌人", enemy_hp_before));

    app.insert_state(GameState::Combat);
    app.update();
    app.update();

    // 检查：没有任何效果
    let player_block = {
        let player = app.world_mut().query::<&Player>().single(app.world());
        player.block
    };
    let enemy_hp = {
        let enemy = app.world_mut().query::<&Enemy>().single(app.world());
        enemy.hp
    };

    assert_eq!(player_block, 0, "没有遗物时玩家不应该有护甲");
    assert_eq!(enemy_hp, enemy_hp_before, "没有遗物时敌人不应该受到伤害");
}

#[test]
fn test_combat_start_processed_only_once() {
    // 测试：战斗开始遗物效果只触发一次
    let mut app = create_app_with_relic();

    let enemy_hp_before = 30;
    app.world_mut().spawn(Enemy::new(0, "测试敌人", enemy_hp_before));

    // 运行多次update
    app.update();
    app.update();
    app.update();

    // 检查伤害只应用了一次
    let enemy = app.world_mut().query::<&Enemy>().single(app.world());
    let damage_dealt = enemy_hp_before - enemy.hp;
    assert_eq!(damage_dealt, 3, "遗物伤害应该只触发一次，实际伤害: {}", damage_dealt);
}
