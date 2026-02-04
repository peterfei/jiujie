//! 完整游戏流程集成测试 (补全资源版)
//! 验证遗物在状态跳转后的生效逻辑

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::state::state::NextState;
use bevy_card_battler::components::*;
use bevy_card_battler::components::relic::{RelicEffect, RelicRarity, RelicId};
use bevy_card_battler::states::GameState;
use bevy_card_battler::systems::RelicPlugin;

#[test]
fn test_full_game_flow_with_starting_relic() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .add_plugins(RelicPlugin)
        .init_state::<GameState>()
        .insert_resource(CombatState::default());

    // 1. 完成初始化
    app.update();

    // 2. 准备战场
    let enemy_hp_before = 30;
    app.world_mut().spawn(Enemy::new(0, "测试敌人", enemy_hp_before));
    app.world_mut().spawn(Player::default());
    // [关键修复] 使用正确的 new() 方法而非不存在的 default()
    app.world_mut().spawn(Hand::new(10));
    app.world_mut().spawn(DrawPile::new(vec![]));
    app.world_mut().spawn(DiscardPile::new());
    
    app.update();

    // 3. 模拟进入战斗状态
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    
    // 运行多帧确保状态转换和 Update 阶段的遗物系统执行
    for _ in 0..5 {
        app.update();
    }

    // 4. 检查效果
    let mut enemy_query = app.world_mut().query::<&Enemy>();
    let mut damage_dealt = 0;
    for enemy in enemy_query.iter(app.world()) {
        if enemy.name == "测试敌人" {
            damage_dealt = enemy_hp_before - enemy.hp;
        }
    }
    
    assert_eq!(damage_dealt, 3, "燃烧之血应该对敌人造成3点伤害");
}

#[test]
fn test_multiple_relics_stack_effects() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .add_plugins(RelicPlugin)
        .init_state::<GameState>()
        .insert_resource(CombatState::default());

    app.update();

    // 1. 添加第二个遗物
    {
        let mut relic_collection = app.world_mut().get_resource_mut::<RelicCollection>().unwrap();
        let stone_shield = Relic {
            id: RelicId::BagOfPreparation,
            name: "石盾".to_string(),
            description: "战斗开始时获得5点护甲".to_string(),
            rarity: RelicRarity::Common,
            effects: vec![RelicEffect::OnCombatStart { damage: 0, block: 5, draw_cards: 0 }],
        };
        relic_collection.add_relic_forced(stone_shield);
    }

    // 2. 准备战场
    app.world_mut().spawn(Enemy::new(0, "敌人1", 30));
    app.world_mut().spawn(Player::default());
    app.world_mut().spawn(Hand::new(10));
    app.world_mut().spawn(DrawPile::new(vec![]));
    app.world_mut().spawn(DiscardPile::new());
    app.update();

    // 3. 跳转状态
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    for _ in 0..5 {
        app.update();
    }

    // 4. 验证效果
    let player = app.world_mut().query::<&Player>().single(app.world());
    assert_eq!(player.block, 5, "石盾应该给予5点护甲");
    
    let mut enemy_query = app.world_mut().query::<&Enemy>();
    let mut enemy_damaged = false;
    for enemy in enemy_query.iter(app.world()) {
        if enemy.hp == 27 { enemy_damaged = true; }
    }
    assert!(enemy_damaged, "燃烧之血应该造成3点伤害");
}