use bevy::prelude::*;
use bevy::app::App;
use bevy_card_battler::components::sprite::{PlayerSpriteMarker, SpriteMarker};
use bevy_card_battler::resources::PlayerAssets;

#[test]
fn test_player_modular_loading_and_weapon_attachment() {
    let mut app = App::new();
    
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        bevy::scene::ScenePlugin,
    ));

    // 1. 模拟资源定义
    let assets = PlayerAssets {
        body: Handle::default(),
        weapon: Handle::default(),
    };
    app.insert_resource(assets);

    // 2. 运行生成系统 (模拟进入战斗或主菜单逻辑)
    // 假设系统名为 spawn_modular_player
    // 我们手动在测试中推进
    app.add_systems(Update, spawn_modular_player_test_system);
    app.update();

    // 3. 验证主体生成
    let mut player_query = app.world_mut().query_filtered::<Entity, With<PlayerSpriteMarker>>();
    let player_entity = player_query.iter(app.world()).next().expect("应该生成玩家实体");
    
    assert!(app.world().get::<SceneRoot>(player_entity).is_some(), "玩家主体应该有 SceneRoot");

    // 4. 验证武器挂载 (作为子物体)
    let mut children_query = app.world_mut().query::<&Children>();
    let children = children_query.get(app.world(), player_entity).expect("玩家实体应该有子物体(武器)");
    
    let mut weapon_found = false;
    for &child in children.iter() {
        if app.world().get::<SceneRoot>(child).is_some() {
            weapon_found = true;
            break;
        }
    }
    assert!(weapon_found, "应该在玩家子物体中找到武器模型");
}

// 临时定义的测试系统，用于驱动 TDD
fn spawn_modular_player_test_system(
    mut commands: Commands,
    player_assets: Res<PlayerAssets>,
) {
    commands.spawn((
        PlayerSpriteMarker,
        SpriteMarker,
        SceneRoot(player_assets.body.clone()),
    )).with_children(|parent| {
        parent.spawn(SceneRoot(player_assets.weapon.clone()));
    });
}
