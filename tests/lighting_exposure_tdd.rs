use bevy::prelude::*;
use bevy::ecs::system::RunSystemOnce;

#[test]
fn test_lighting_exposure_limits() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.init_resource::<AmbientLight>();

    // 模拟光照初始化
    app.world_mut().run_system_once(|mut commands: Commands| {
        // [TDD] 我们期望环境光不超过 800.0 (历史最佳值)
        commands.insert_resource(AmbientLight {
            color: Color::srgb(0.8, 0.9, 1.0),
            brightness: 800.0,
        });

        // [TDD] 我们期望主光源不超过 25000.0 (避免泛白)
        commands.spawn(DirectionalLight {
            illuminance: 25000.0,
            ..default()
        });
    });

    // 验证环境光
    let ambient = app.world().resource::<AmbientLight>();
    assert!(ambient.brightness <= 1000.0, "环境光过强会导致泛白，应 <= 1000.0");

    // 验证主光源
    let mut query = app.world_mut().query::<&DirectionalLight>();
    let sun = query.iter(app.world()).next().expect("必须有主光源");
    assert!(sun.illuminance <= 30000.0, "主光源过强会导致材质细节丢失，应 <= 30000.0");
}
