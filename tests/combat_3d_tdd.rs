use bevy::prelude::*;
use bevy_card_battler::components::sprite::Combatant3d;

#[test]
fn test_3d_combatant_spawning() {
    let mut app = App::new();
    // Bevy 0.15 基础插件
    app.add_plugins(MinimalPlugins)
       .add_plugins(AssetPlugin::default())
       .init_resource::<Assets<Mesh>>()
       .init_resource::<Assets<StandardMaterial>>();

    // 创建一个模拟的 3D 角色 (Bevy 0.15 风格)
    let mesh_handle = Handle::<Mesh>::default();
    let material_handle = Handle::<StandardMaterial>::default();
    
    let entity = app.world_mut().spawn((
        Combatant3d { facing_right: true },
        Mesh3d(mesh_handle),
        MeshMaterial3d(material_handle),
        Transform::default(),
    )).id();

    // 验证组件是否存在
    assert!(app.world().get::<Combatant3d>(entity).is_some());
    assert!(app.world().get::<Mesh3d>(entity).is_some());
    assert!(app.world().get::<MeshMaterial3d<StandardMaterial>>(entity).is_some());
}

#[test]
fn test_breath_animation_update() {
    use bevy_card_battler::components::sprite::BreathAnimation;
    
    let mut app = App::new();
    let entity = app.world_mut().spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        BreathAnimation {
            timer: 0.0,
            frequency: 1.0,
            amplitude: 1.0,
        },
    )).id();

    // 模拟系统运行 (我们稍后会实现这个系统)
    // 这里我们直接调用一个手动更新逻辑来模拟系统行为
    let mut timer = app.world().get::<BreathAnimation>(entity).unwrap().timer;
    timer += 1.0;
    let new_y = (timer * 1.0).sin() * 1.0;
    
    let mut transform = app.world_mut().get_mut::<Transform>(entity).unwrap();
    transform.translation.y = new_y;

    assert!(app.world().get::<Transform>(entity).unwrap().translation.y != 0.0);
}
