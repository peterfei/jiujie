use bevy::prelude::*;
use bevy::scene::SceneRoot;
use bevy::ecs::system::RunSystemOnce;
use bevy_card_battler::components::sprite::{CharacterAssets, CharacterType, SpriteMarker, CharacterSprite};
use bevy_card_battler::systems::sprite::spawn_character_sprite;

#[test]
fn test_character_3d_fallback_logic() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(bevy::scene::ScenePlugin);
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.init_resource::<Assets<Image>>();
    app.init_resource::<Assets<Scene>>();

    // 初始化 2D 资源
    let character_assets = CharacterAssets {
        player_idle: Handle::<Image>::default(),
        wolf: Handle::<Image>::default(),
        // ... 其他 2D 贴图
        ..default()
    };
    app.insert_resource(character_assets);

    // --- 🔴 红区：模拟 3D 模型未加载 (句柄为 None) ---
    app.world_mut().run_system_once(|mut commands: Commands, assets: Res<CharacterAssets>, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>| {
        spawn_character_sprite(
            &mut commands,
            &assets,
            CharacterType::Player,
            Vec3::ZERO,
            Vec2::splat(100.0),
            None,
            None,
            &mut meshes,
            &mut materials,
        );
    });

    // 验证：应当生成了 2D 属性（CharacterSprite），但没有 SceneRoot
    let sprite_query = app.world_mut().query::<&CharacterSprite>().iter(&app.world()).count();
    let scene_query = app.world_mut().query::<&SceneRoot>().iter(&app.world()).count();
    
    assert!(sprite_query > 0, "回退失败：应该生成 2D 精灵组件");
    assert_eq!(scene_query, 0, "逻辑错误：不应在无模型时生成 SceneRoot");
    println!("✅ 红区测试通过：无模型时安全降级为 2D 纸片人。");

    // --- 🟢 绿区：模拟 3D 模型已就绪 ---
    let mock_scene = app.world_mut().resource_mut::<Assets<Scene>>().add(Scene::new(World::new()));
    {
        let mut assets = app.world_mut().resource_mut::<CharacterAssets>();
        assets.player_3d = Some(mock_scene.clone()); // 注入模拟 3D 句柄
    }

    app.world_mut().run_system_once(|mut commands: Commands, assets: Res<CharacterAssets>, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>| {
        spawn_character_sprite(
            &mut commands,
            &assets,
            CharacterType::Player,
            Vec3::ZERO,
            Vec2::splat(100.0),
            None,
            None,
            &mut meshes,
            &mut materials,
        );
    });

    // 验证：应当生成了 SceneRoot 
    let scene_count = app.world_mut().query::<&SceneRoot>().iter(&app.world()).count();
    assert!(scene_count > 0, "绿区失败：3D 模型就绪时应生成 SceneRoot");
    println!("✅ 绿区测试通过：成功加载 3D 角色模型。");
}