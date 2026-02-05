use bevy::prelude::*;
use bevy::scene::SceneRoot;
use bevy::ecs::system::RunSystemOnce; // 必须导入
use bevy_card_battler::resources::{EnvironmentAssets, LandscapeGenerator, EnvironmentConfig};
use bevy_card_battler::systems::sprite::spawn_procedural_landscape;

#[test]
fn test_landscape_fallback_logic_red_green() {
    let mut app = App::new();
    
    // 基础插件与资源初始化
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    // ScenePlugin 包含在 DefaultPlugins 中，这里需要手动添加
    app.add_plugins(bevy::scene::ScenePlugin); 
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.init_resource::<Assets<Image>>();
    app.init_resource::<Assets<Scene>>();
    
    app.insert_resource(LandscapeGenerator::new(666));
    app.insert_resource(EnvironmentConfig::default());

    // --- 第一阶段：红区测试 (无资源加载) ---
    // 验证：在没有 EnvironmentAssets 资源时，系统仍能正常运行（走回退逻辑）
    app.world_mut().run_system_once(spawn_procedural_landscape);
    
    // 此时不应有 SceneRoot 实体
    let scene_count = app.world_mut().query::<&SceneRoot>().iter(&app.world()).count();
    assert_eq!(scene_count, 0, "红区验证失败：未加载资源时不应产生 SceneRoot");
    println!("✅ 红区测试通过：资源缺失时安全回退至几何体。");

    // --- 第二阶段：绿区测试 (注入模拟资源) ---
    // 注入模拟的 EnvironmentAssets 
    app.init_resource::<EnvironmentAssets>();
    let mock_scene_handle = {
        let mut scenes = app.world_mut().resource_mut::<Assets<Scene>>();
        scenes.add(Scene::new(World::new()))
    };
    
    {
        let mut env_assets = app.world_mut().resource_mut::<EnvironmentAssets>();
        env_assets.rock = mock_scene_handle.clone();
        env_assets.cloud = mock_scene_handle.clone();
    }

    // 重新运行生成系统
    app.world_mut().run_system_once(spawn_procedural_landscape);

    // 验证：此时系统应检测到资源并生成 SceneRoot 实体
    let scene_count_after = app.world_mut().query::<&SceneRoot>().iter(&app.world()).count();
    assert!(scene_count_after > 0, "绿区验证失败：资源有效时应生成 SceneRoot 实体");
    println!("✅ 绿区测试通过：成功生成 {} 个基于 GLTF 的场景实体。", scene_count_after);
}
