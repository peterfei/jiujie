use bevy::prelude::*;
use bevy_card_battler::components::*;
use bevy_card_battler::states::GameState;
use bevy_card_battler::systems::rest::*;

#[test]
fn test_rest_node_completion_on_leave() {
    let mut app = App::new();
    app.init_state::<GameState>();
    app.insert_resource(State::new(GameState::Rest));
    app.init_resource::<NextState<GameState>>();
    
    // 模拟资源
    let mut map_progress = MapProgress::default();
    map_progress.current_node_id = Some(0);
    app.insert_resource(map_progress);
    app.world_mut().spawn((Player::default(), Cultivation::new()));
    app.insert_resource(PlayerDeck::default());
    app.insert_resource(RelicCollection::default());
    
    // 模拟点击离开
    app.world_mut().spawn((Interaction::Pressed, LeaveButton));
    
    // 运行交互系统
    app.add_systems(Update, handle_leave_interaction);
    app.update();
    
    // 验证：节点应标记为完成
    let progress = app.world().resource::<MapProgress>();
    let node = progress.nodes.iter().find(|n| n.id == 0).unwrap();
    assert!(node.completed, "离开休息节点后，该节点应标记为已完成");
}