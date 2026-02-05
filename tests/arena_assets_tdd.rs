use bevy::prelude::*;
use bevy::app::App;
use bevy_card_battler::resources::ArenaAssets;
use bevy_card_battler::systems::sprite::spawn_modular_arena;

#[test]
fn test_arena_modular_loading_and_spawning() {
    let mut app = App::new();
    
    // 最小化基础环境
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        bevy::scene::ScenePlugin,
    ));

    // 1. 初始化资源（模拟加载后的状态）
    let assets = ArenaAssets {
        base_platform: Handle::default(),
        pillar: Handle::default(),
        main_prop: Handle::default(),
        lantern: Handle::default(),
        sword_debris: Handle::default(),
    };
    app.insert_resource(assets);

    // 2. 手动运行生成系统
    let mut schedule = Schedule::new(Update);
    schedule.add_systems(spawn_modular_arena);
    schedule.run(app.world_mut());

    // 3. 验证实体生成数量
    // Base(1) + Pillars(4) + Prop(1) + Debris(1) = 7
    let scene_count = app.world_mut().query::<&SceneRoot>().iter(app.world()).count();
    
    println!("检测到 SceneRoot 实体数量: {}", scene_count);
    assert_eq!(scene_count, 7, "模块化对战场景实体应为 7 个");
}
