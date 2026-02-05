use bevy::prelude::*;
use bevy_card_battler::systems::sprite::spawn_procedural_landscape;
use bevy_card_battler::resources::{LandscapeGenerator, EnvironmentAssets};
use bevy_card_battler::components::sprite::{ArenaLantern, ArenaVegetation, ArenaSpiritStone};
use bevy::ecs::system::RunSystemOnce;

#[test]
fn test_arena_decorations_v2() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    app.init_asset::<Mesh>();
    app.init_asset::<StandardMaterial>();
    app.init_asset::<Scene>();
    
    app.insert_resource(LandscapeGenerator::new(888));
    app.insert_resource(EnvironmentAssets::default());

    // 运行景观生成系统
    let _ = app.world_mut().run_system_once(spawn_procedural_landscape);
    app.update();

    // 1. 验证是否生成了灯笼
    let lantern_count = app.world_mut().query_filtered::<Entity, With<ArenaLantern>>().iter(app.world()).count();
    assert!(lantern_count > 0, "战场应生成灵灯装饰");

    // 2. 验证是否生成了竞技场中心植被
    let vegetation_count = app.world_mut().query_filtered::<Entity, With<ArenaVegetation>>().iter(app.world()).count();
    assert!(vegetation_count > 0, "战场中心应生成绿植点缀");

    // 3. 验证是否生成了灵石
    let stone_count = app.world_mut().query_filtered::<Entity, With<ArenaSpiritStone>>().iter(app.world()).count();
    assert!(stone_count > 0, "战场应生成灵石装饰");
}