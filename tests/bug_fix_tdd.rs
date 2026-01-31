use bevy::prelude::*;
use bevy_card_battler::states::GameState;
use bevy_card_battler::components::map::{MapProgress, NodeType, MapNode};
use bevy_card_battler::components::{Player, Cultivation, PlayerDeck, relic::RelicCollection};
use bevy_card_battler::plugins::GamePlugin;

#[test]
fn test_event_heal_hang_repro() {
    let mut app = App::new();
    
    // 使用最小化插件，但包含核心逻辑
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin); // 必须添加状态机支持
    app.add_plugins(GamePlugin);
    
    // 设置初始状态为 Event
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Event);
    
    // 初始化必要资源
    app.insert_resource(PlayerDeck::default());
    app.insert_resource(RelicCollection::default());
    
    // 运行一帧进入 Event 状态
    app.update();
    
    // 确保有 Player 实体
    if app.world_mut().query::<&Player>().iter(app.world()).next().is_none() {
        app.world_mut().spawn((
            Player { 
                hp: 10, 
                max_hp: 80, 
                gold: 100, 
                energy: 3, 
                max_energy: 3, 
                block: 0,
                turn: 1,
                vulnerable: 0,
                poison: 0,
                weakness: 0,
            },
            Cultivation::new(),
        ));
    }
    
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
    
    info!("模拟点击机缘回复生命按钮...");
    
    // 修改状态并触发转换
    {
        let mut player_query = app.world_mut().query::<&mut Player>();
        let mut player = player_query.get_single_mut(app.world_mut()).unwrap();
        player.hp = (player.hp + 20).min(player.max_hp);
        
        let mut map_progress = app.world_mut().resource_mut::<MapProgress>();
        map_progress.complete_current_node();
        
        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Map);
    }
    
    // 连续更新多帧，模拟进入 Map 状态
    for i in 0..10 {
        info!("更新第 {} 帧...", i);
        app.update();
        
        let current_state = app.world().resource::<State<GameState>>().get();
        if i > 5 {
             assert_eq!(*current_state, GameState::Map, "第 {} 帧状态未成功切换到 Map!", i);
        }
    }
    
    info!("测试完成，逻辑跑通");
}
