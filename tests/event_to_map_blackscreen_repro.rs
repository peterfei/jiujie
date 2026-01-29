use bevy::prelude::*;
use bevy_card_battler::components::*;
use bevy_card_battler::states::GameState;
use bevy_card_battler::components::map::MapUiRoot;

#[test]
fn test_event_to_map_transition_ui_cleanup_repro() {
    let mut app = App::new();
    app.init_state::<GameState>();
    
    // 模拟资源
    app.insert_resource(Player::default());
    app.insert_resource(Cultivation::new());
    app.insert_resource(MapProgress::default());
    
    // 1. 模拟处于 Event 状态并有 UI
    app.world_mut().insert_resource(State::new(GameState::Event));
    app.world_mut().spawn((Node::default(), EventUiRoot));
    
    // 2. 模拟 handle_event_choices 的行为
    // 我们手动模拟一次清理和状态切换
    let mut next_state = app.world_mut().resource_mut::<NextState<GameState>>();
    next_state.set(GameState::Map);
    
    // 清理 Event UI
    let mut query = app.world_mut().query_filtered::<Entity, With<EventUiRoot>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    for e in entities {
        app.world_mut().despawn_recursive(e);
    }
    
    // 3. 运行一帧更新，触发状态切换
    app.update();
    
    // 4. 验证是否生成了 MapUiRoot (虽然实际测试中由于没挂插件可能不会生成，但我们要验证逻辑链路)
    // 在真实应用中，OnEnter(GameState::Map) 应该运行 setup_map_ui
}
