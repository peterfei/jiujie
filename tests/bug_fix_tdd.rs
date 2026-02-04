use bevy::prelude::*;
use bevy_card_battler::states::GameState;
use bevy_card_battler::components::map::{MapProgress, NodeType, MapNode};
use bevy_card_battler::components::{Player, Cultivation, PlayerDeck, relic::RelicCollection};

#[test]
fn test_event_heal_hang_repro() {
    let mut app = App::new();
    
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    
    app.init_state::<GameState>();
    
    // 初始化必要资源
    app.insert_resource(PlayerDeck::default());
    app.insert_resource(RelicCollection::default());
    
    // 初始化玩家和敌人
    app.world_mut().spawn(
        Player { 
            hp: 50, 
            max_hp: 80, 
            ..default()
        }
    );

    
    // 模拟 MapProgress
    let mut map_progress = MapProgress::default();
    map_progress.nodes.push(MapNode {
        id: 1,
        node_type: NodeType::Event,
        position: (0, 0),
        next_nodes: vec![],
        completed: false,
        unlocked: true,
    });
    map_progress.current_node_id = Some(1);
    app.insert_resource(map_progress);
    
    app.update();
    
    // 执行状态转换逻辑
    {
        let mut next_state = app.world_mut().resource_mut::<NextState<GameState>>();
        next_state.set(GameState::Map);
    }
    
    // 连续更新
    for _ in 0..5 {
        app.update();
    }
    
    let current_state = app.world().resource::<State<GameState>>().get();
    assert_eq!(*current_state, GameState::Map, "状态转换应该成功");
}