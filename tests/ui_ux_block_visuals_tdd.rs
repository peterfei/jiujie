use bevy::prelude::*;
use bevy_card_battler::components::combat::{Player, BlockIconMarker, BlockText};
use bevy_card_battler::states::GameState;
use bevy_card_battler::systems::ui::UiPlugin;

#[test]
fn test_block_visibility_logic() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.add_plugins(UiPlugin); // 使用真实插件
    app.init_state::<GameState>();
    app.insert_state(GameState::Combat);

    // 1. 创建一个带有护甲的玩家
    let mut player = Player::default();
    player.block = 10;
    let player_id = app.world_mut().spawn(player).id();

    // 2. 模拟 UI 结构 (通常由 setup_combat_ui 生成，这里手动模拟)
    app.world_mut().spawn((
        Node::default(),
        BlockIconMarker { owner: player_id },
        Visibility::Hidden,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("0"),
            BlockText,
        ));
    });

    // 运行一帧处理更新系统
    app.update();

    // 3. 验证显示逻辑
    let mut query = app.world_mut().query_filtered::<&Node, With<BlockIconMarker>>();
    let node = query.get_single(app.world()).expect("应该存在护甲图标实体");
    assert_eq!(node.display, Display::Flex, "当护甲 > 0 时，图标应该是显示的 (Flex)");

    // 验证文字逻辑
    let mut text_query = app.world_mut().query_filtered::<&Text, With<BlockText>>();
    let text = text_query.get_single(app.world()).unwrap();
    assert_eq!(text.0, "10");

    // 4. 测试护甲归零
    if let Some(mut p) = app.world_mut().get_mut::<Player>(player_id) {
        p.block = 0;
    }
    app.update();
    
    let mut query = app.world_mut().query_filtered::<&Node, With<BlockIconMarker>>();
    let node = query.get_single(app.world()).unwrap();
    assert_eq!(node.display, Display::None, "当护甲为 0 时，图标应该是隐藏的 (None)");

    println!("✅ 护甲视觉化 UI/UX 优化 TDD 测试通过");
}
