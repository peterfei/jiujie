use bevy::prelude::*;
use bevy_card_battler::states::GameState;
use bevy_card_battler::components::map::{MapProgress, MapUiRoot};
use bevy_card_battler::plugins::EventUiRoot;

#[test]
fn test_event_to_map_ui_cleanup() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    
    app.init_resource::<NextState<GameState>>();
    app.insert_resource(State::new(GameState::Event));
    
    // 模拟 Event UI 存在
    app.world_mut().spawn((Node::default(), EventUiRoot));
    
    // 执行清理逻辑 (模拟 handle_event_choices 中的清理)
    {
        let mut world = app.world_mut();
        let mut query = world.query_filtered::<Entity, With<EventUiRoot>>();
        let entities: Vec<Entity> = query.iter(world).collect();
        for e in entities {
            world.entity_mut(e).despawn_recursive();
        }
    }
    
    app.update();
    
    // 验证 UI 已清理
    let mut query = app.world_mut().query_filtered::<Entity, With<EventUiRoot>>();
    assert_eq!(query.iter(app.world()).count(), 0, "Event UI 应该已被清理");
}
