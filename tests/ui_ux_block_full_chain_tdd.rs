use bevy::prelude::*;
use bevy_card_battler::components::combat::{Player, BlockIconMarker, BlockText};
use bevy_card_battler::states::GameState;
use bevy_card_battler::systems::ui::UiPlugin;

#[test]
fn test_block_ui_full_chain_validation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.add_plugins(UiPlugin);
    app.init_state::<GameState>();
    app.insert_state(GameState::Combat);

    // 1. 准备玩家
    let player_id = app.world_mut().spawn(Player::default()).id();

    // 2. 手动构建与主代码完全一致的 UI 树
    // root -> container -> shield (BlockIconMarker) -> text (BlockText)
    let root = app.world_mut().spawn(Node::default()).id();
    let container = app.world_mut().spawn(Node {
        flex_direction: FlexDirection::Row,
        ..default()
    }).id();
    
    let shield = app.world_mut().spawn((
        Node { display: Display::None, ..default() },
        BlockIconMarker { owner: player_id },
    )).id();
    
    let text_node = app.world_mut().spawn((
        Text::new("0"),
        BlockText,
    )).id();

    app.world_mut().entity_mut(shield).add_child(text_node);
    app.world_mut().entity_mut(container).add_child(shield);
    app.world_mut().entity_mut(root).add_child(container);

    // 3. 模拟获得护甲
    if let Some(mut p) = app.world_mut().get_mut::<Player>(player_id) {
        p.block = 42;
    }

    // 4. 更新系统
    app.update();

    // 5. 深度验证
    let node = app.world().get::<Node>(shield).unwrap();
    assert_eq!(node.display, Display::Flex, "护甲 > 0 时，Display 必须是 Flex");

    let text = app.world().get::<Text>(text_node).unwrap();
    assert_eq!(text.0, "42", "护甲数值必须同步到 Text 组件");

    // 6. 验证父子关系是否稳固
    let children = app.world().get::<Children>(shield).unwrap();
    assert!(children.contains(&text_node), "Shield 必须包含 Text 子节点");

    println!("✅ 逻辑链条验证通过：系统正确地处理了数值和显示状态。");
}
