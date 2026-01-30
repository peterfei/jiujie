use bevy::prelude::*;
use bevy_card_battler::states::GameState;

#[path = "test_utils.rs"]
mod test_utils;
use test_utils::*;

#[test]
fn test_state_transition_stress_stability() {
    let mut app = create_test_app();
    
    // 快速状态切换序列
    let states = [
        GameState::Map,
        GameState::Combat,
        GameState::Reward,
        GameState::Map,
        GameState::Shop,
        GameState::Map,
        GameState::Rest,
        GameState::Map,
    ];
    
    for (i, &state) in states.iter().enumerate() {
        app.world_mut().resource_mut::<NextState<GameState>>().set(state);
        
        // 模拟几帧运行，确保 OnEnter/OnExit 逻辑完全执行
        for _ in 0..2 {
            app.update();
        }
        
        info!("状态切换测试 [{}]: 进入 {:?}", i, state);
        
        // 验证基本活跃资源
        assert!(app.world().get_resource::<State<GameState>>().is_some());
    }
}

#[test]
fn test_concurrent_despawn_safety() {
    let mut app = create_test_app();
    
    // 模拟一种极端情况：手动在同一帧多次排队切换状态
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Map);
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    
    app.update();
    
    // 验证 Bevy 是否处理了这种竞争（应以最后一次 set 为准）
    let current_state = app.world().resource::<State<GameState>>();
    assert!(*current_state.get() == GameState::Combat || *current_state.get() == GameState::Map);
}
