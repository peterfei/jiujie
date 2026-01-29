use bevy::prelude::*;
use bevy_card_battler::components::map::{BreakthroughButtonMarker, MapUiRoot};
use bevy_card_battler::states::GameState;

#[test]
fn test_breakthrough_button_hover_repro() {
    let mut app = App::new();
    app.init_state::<GameState>();
    app.insert_resource(State::new(GameState::Map));
    
    // 1. 手动创建按钮，模拟真实生成的属性
    let button_entity = app.world_mut().spawn((
        Button,
        Node {
            width: Val::Px(240.0),
            height: Val::Px(90.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.05, 0.2, 0.95)),
        BreakthroughButtonMarker,
        MapUiRoot,
        Visibility::Visible,
        InheritedVisibility::VISIBLE,
    )).id();
    
    // 2. 模拟鼠标移入
    // 在 Bevy 0.15 中，Interaction 会被自动更新，或者我们可以手动发送 PointerOver
    app.world_mut().entity_mut(button_entity).insert(Interaction::Hovered);
    
    // 3. 运行多帧，模拟动画系统运行
    // 注意：这里需要添加 animation 系统的部分组件来观察冲突
    
    app.update();
    
    // 4. 检查按钮是否还在
    let node = app.world().get::<Node>(button_entity).unwrap();
    let vis = app.world().get::<InheritedVisibility>(button_entity).unwrap();
    
    println!("Button Node size after hover: {:?} x {:?}", node.width, node.height);
    println!("Button Visibility after hover: {:?}", vis.get());
    
    // 如果尺寸变成了 0，或者非法值，测试就会失败
    if let Val::Px(w) = node.width {
        assert!(w > 0.0, "按钮尺寸不应为 0");
    }
}
