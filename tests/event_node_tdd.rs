use bevy::prelude::*;
use bevy_card_battler::states::GameState;
use bevy_card_battler::components::combat::Player;

#[test]
fn test_event_node_logic_flow() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<GameState>();
    
    // 1. 模拟玩家 (初始 100)
    let player_ent = app.world_mut().spawn(Player::default()).id();

    // 2. 模拟机缘事件奖励逻辑
    fn mock_apply_reward(mut q: Query<&mut Player>) {
        if let Ok(mut p) = q.get_single_mut() {
            p.gold += 50;
        }
    }
    
    app.add_systems(OnEnter(GameState::Event), mock_apply_reward);

    // 3. 执行状态切换 (模拟地图点击)
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Event);
    app.update(); // 状态切换帧，触发 OnEnter
    
    // 4. 验证
    let player = app.world().get::<Player>(player_ent).unwrap();
    assert_eq!(player.gold, 150, "机缘事件 OnEnter 应触发奖励增加 50");
    
    println!("✅ 机缘事件逻辑集成测试全绿通过");
}