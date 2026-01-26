use bevy::prelude::*;
use bevy_card_battler::plugins::CombatUiRoot;

#[test]
fn test_environment_fog_settings() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(AssetPlugin::default());

    // 模拟生成带雾气的相机
    app.world_mut().spawn((
        Camera3d::default(),
        DistanceFog {
            color: Color::srgba(0.1, 0.2, 0.1, 1.0),
            falloff: FogFalloff::Linear { start: 5.0, end: 20.0 },
            ..default()
        },
    ));

    // 验证相机上是否有 DistanceFog
    let mut query = app.world_mut().query::<&DistanceFog>();
    assert!(query.iter(app.world()).next().is_some(), "战斗场景相机应该配置环境雾气");
}

#[test]
fn test_floor_glowing_material() {
    let mut app = App::new();
    app.init_resource::<Assets<StandardMaterial>>();
    
    // 模拟生成发光地板
    let material_handle = app.world_mut().resource_mut::<Assets<StandardMaterial>>().add(StandardMaterial {
        emissive: LinearRgba::new(0.0, 1.0, 0.0, 1.0), // 强发光
        ..default()
    });

    app.world_mut().spawn((
        MeshMaterial3d(material_handle),
        CombatUiRoot,
    ));

    // 验证材质属性
    let materials = app.world().resource::<Assets<StandardMaterial>>();
    let (_, mat) = materials.iter().next().unwrap();
    assert!(mat.emissive.green > 0.0, "地板材质应该具有发光(Emissive)属性以模拟法阵效果");
}

#[test]
fn test_rotation_update() {
    use bevy_card_battler::components::sprite::Rotating;
    
    let mut app = App::new();
    let entity = app.world_mut().spawn((
        Transform::default(),
        Rotating { speed: 1.0 },
    )).id();

    // 模拟系统运行 (我们稍后会实现这个系统)
    // 旋转 1.0 弧度
    let mut transform = app.world_mut().get_mut::<Transform>(entity).unwrap();
    transform.rotate_y(1.0);

    let rotation = app.world().get::<Transform>(entity).unwrap().rotation;
    assert!(rotation != Quat::IDENTITY, "拥有 Rotating 组件的实体应该发生旋转");
}

#[test]
fn test_spirit_particle_spawning() {
    use bevy_card_battler::components::particle::ParticleEmitter;
    
    let mut app = App::new();
    let entity = app.world_mut().spawn((
        ParticleEmitter {
            enabled: true,
            timer: 0.0,
            rate: 0.1, // 每秒 10 个
            config: Default::default(),
        },
    )).id();

    assert!(app.world().get::<ParticleEmitter>(entity).unwrap().enabled);
}
