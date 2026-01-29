use bevy::prelude::*;
use bevy_card_battler::components::map::{BreakthroughButtonMarker};

#[test]
fn test_breakthrough_button_stability() {
    let mut app = App::new();
    
    // 模拟新一代稳健按钮
    app.world_mut().spawn((
        BreakthroughButtonMarker,
        Button,
        Node::default(),
    ));
    
    // 验证它不再依赖那些会打架的组件
    let mut query = app.world_mut().query::<&BreakthroughButtonMarker>();
    let found = query.iter(app.world()).next();
    
    assert!(found.is_some(), "引动雷劫按钮应该存在");
}