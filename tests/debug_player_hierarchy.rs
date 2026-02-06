use bevy::prelude::*;
use bevy_card_battler::components::sprite::{CharacterAssets, CharacterType, PlayerSpriteMarker};
use bevy_card_battler::systems::sprite::spawn_character_sprite;

#[test]
fn test_print_player_hierarchy() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(bevy::scene::ScenePlugin);
    app.add_plugins(bevy::gltf::GltfPlugin::default());
    app.init_asset::<Mesh>();
    app.init_asset::<StandardMaterial>();
    app.init_asset::<AnimationClip>();
    app.init_asset::<AnimationGraph>();

    let asset_server = app.world().resource::<AssetServer>();
    let assets = CharacterAssets::load(asset_server);
    app.insert_resource(assets.clone());

    let mut system_state = bevy::ecs::system::SystemState::<(
        Commands,
        Res<CharacterAssets>,
        ResMut<Assets<Mesh>>,
        ResMut<Assets<StandardMaterial>>,
        ResMut<Assets<AnimationGraph>>,
    )>::new(app.world_mut());

    let (mut commands, assets, mut meshes, mut materials, mut graphs) = system_state.get_mut(app.world_mut());
    
    let player = spawn_character_sprite(
        &mut commands,
        &assets,
        CharacterType::Player,
        Vec3::ZERO,
        Vec2::splat(100.0),
        None,
        None,
        &mut meshes,
        &mut materials,
        &mut graphs,
        None,
    );
    system_state.apply(app.world_mut());

    // 模拟运行几帧让 Scene 展开 (实际上测试环境没法展开真正 GLB)
    // 但我们可以检查初始插入的组件
    println!("Player Entity: {:?}", player);
    if let Some(children) = app.world().get::<Children>(player) {
        println!("Children count: {}", children.len());
    }
}
