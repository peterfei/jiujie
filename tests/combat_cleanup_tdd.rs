use bevy::prelude::*;
use bevy_card_battler::components::{CombatUiRoot};
use bevy_card_battler::plugins::cleanup_combat_ui;
use bevy_card_battler::systems::sprite::spawn_procedural_landscape;
use bevy_card_battler::resources::{LandscapeGenerator, EnvironmentAssets};
use bevy::ecs::system::RunSystemOnce;

#[test]
fn test_combat_3d_cleanup_flow() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    app.init_asset::<Mesh>();
    app.init_asset::<StandardMaterial>();
    app.init_asset::<Scene>();
    
    app.insert_resource(LandscapeGenerator::new(123));
    app.insert_resource(EnvironmentAssets::default());

    // 1. 模拟进入战斗：生成景观
    let _ = app.world_mut().run_system_once(spawn_procedural_landscape);
    
    // 检查是否生成了光源和岛屿
    let light_count = app.world_mut().query::<&DirectionalLight>().iter(app.world()).count();
    let root_count = app.world_mut().query_filtered::<Entity, With<CombatUiRoot>>().iter(app.world()).count();
    
    println!("生成检测：平行光={}, 标记实体={}", light_count, root_count);
    assert!(light_count > 0, "应生成至少一个平行光");
    assert!(root_count > 0, "应生成带有 CombatUiRoot 标记的景观根节点");

    // 2. 模拟退出战斗：执行清理系统
    let _ = app.world_mut().run_system_once(cleanup_combat_ui);
    
    // 必须要 update 一次让 despawn 延迟生效
    app.update();

    // 3. 验证清理结果
    let final_light_count = app.world_mut().query::<&DirectionalLight>().iter(app.world()).count();
    let final_root_count = app.world_mut().query_filtered::<Entity, With<CombatUiRoot>>().iter(app.world()).count();

    println!("清理检测：平行光={}, 标记实体={}", final_light_count, final_root_count);
    assert_eq!(final_light_count, 0, "清理后不应残留平行光，否则会导致下一场战斗泛白");
    assert_eq!(final_root_count, 0, "清理后不应残留带有 CombatUiRoot 标记的实体");
    
    println!("✅ 绿区达成：3D 景观与光源清理逻辑验证通过！");
}
