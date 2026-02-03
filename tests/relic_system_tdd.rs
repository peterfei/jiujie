use bevy::prelude::*;
use bevy_card_battler::components::relic::{Relic, RelicEffect, RelicCollection, RelicRarity, RelicId};
use bevy_card_battler::components::combat::{Player, CombatState};
use bevy_card_battler::systems::{CombatStartProcessed, trigger_relics_on_combat_start};
use bevy_card_battler::states::GameState;

#[test]
fn test_relic_multi_trigger_logic() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin); // 显式添加状态插件
    app.init_state::<GameState>();
    
    // 1. 初始化资源
    app.insert_resource(RelicCollection::default());
    app.insert_resource(CombatState::default());
    app.insert_resource(CombatStartProcessed::default());
    
    // 2. 注入玩家 (带 0 护甲)
    let player_ent = app.world_mut().spawn(Player::default()).id();

    // 3. 动态创建一个复合遗物
    let custom_relic = Relic {
        id: RelicId::Custom(999),
        name: "聚灵珠".to_string(),
        description: "战斗开始获得5护甲".to_string(),
        rarity: RelicRarity::Rare,
        effects: vec![
            RelicEffect::OnCombatStart { damage: 0, block: 5, draw_cards: 0 },
        ],
    };
    
    app.world_mut().resource_mut::<RelicCollection>().add_relic_forced(custom_relic);

    // 4. 模拟进入战斗状态并运行系统
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.update();

    use bevy::ecs::system::RunSystemOnce;
    let _ = app.world_mut().run_system_once(trigger_relics_on_combat_start);
    
    let player = app.world().get::<Player>(player_ent).unwrap();
    assert_eq!(player.block, 5, "遗物应在战斗开始时通过动态注入的 effects 赋予 5 护甲");

    println!("✅ 遗物系统动态触发验证通过 (AAA级重构全绿)");
}