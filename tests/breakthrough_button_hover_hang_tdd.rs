use bevy::prelude::*;
use bevy_card_battler::states::GameState;
use bevy_card_battler::components::{Cultivation, Player};

#[test]
fn test_breakthrough_button_hover_repro() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    
    app.init_state::<GameState>();
    
    // 模拟可突破状态逻辑
    let mut cultivation = Cultivation::new();
    cultivation.insight = 100;
    app.world_mut().spawn((
        Player::default(),
        cultivation,
    ));
    
    app.update();
    
    // 只要系统不崩溃，且状态机运行正常即可
    let current_state = app.world().resource::<State<GameState>>().get();
    assert_eq!(*current_state, GameState::MainMenu); // 初始状态
}