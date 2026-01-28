use bevy::prelude::*;
use bevy_card_battler::components::combat::{Player, Enemy, VictoryDelay, CombatState};
use bevy_card_battler::components::{Cultivation, VictoryEvent};
use bevy_card_battler::states::GameState;

#[test]
fn test_integration_victory_gold_reward_flow() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin); // 关键：增加状态插件
    app.init_state::<GameState>();
    app.add_event::<VictoryEvent>();
    app.insert_resource(VictoryDelay::new(1.0));
    app.insert_resource(CombatState::default());

    // 1. 模拟初始入世
    let player_ent = app.world_mut().spawn((
        Player::default(), 
        Cultivation::new(),
    )).id();
    
    let initial_gold = app.world().get::<Player>(player_ent).unwrap().gold;
    assert_eq!(initial_gold, 100);

    // 2. 模拟遭遇战
    app.world_mut().spawn(Enemy::new(1, "遭遇毒蛛", 50));
    
    // 3. 模拟战斗过程
    let mut enemy_query = app.world_mut().query::<&mut Enemy>();
    for mut enemy in enemy_query.iter_mut(app.world_mut()) {
        enemy.hp = 0;
    }

    // 4. 运行结算系统
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.update(); 

    use bevy::ecs::system::RunSystemOnce;
    let _ = app.world_mut().run_system_once(bevy_card_battler::plugins::check_combat_end_wrapper);

    // 5. 最终验证：灵石是否增加？
    let final_gold = app.world().get::<Player>(player_ent).unwrap().gold;
    
    println!("初始灵石: {}, 结算后灵石: {}", initial_gold, final_gold);
    assert!(final_gold > initial_gold, "【红灯确认】逻辑被禁用时，灵石没有增加");
}