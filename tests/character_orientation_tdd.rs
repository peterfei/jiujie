use bevy::prelude::*;
use bevy_card_battler::components::sprite::{CharacterAssets, CharacterType, Combatant3d};
use bevy_card_battler::systems::sprite::spawn_character_sprite;
use bevy::ecs::system::SystemState;

#[test]
fn test_character_orientation_logic() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.init_resource::<Assets<Image>>();
    app.init_resource::<Assets<Scene>>();

    let assets = CharacterAssets::default();
    app.insert_resource(assets);

    // 1. 验证玩家朝向 (PI/2 -> PI -> 0.0)
    {
        let mut system_state: SystemState<(Commands, Res<CharacterAssets>, ResMut<Assets<Mesh>>, ResMut<Assets<StandardMaterial>>)> = SystemState::new(app.world_mut());
        let (mut commands, assets, mut meshes, mut materials) = system_state.get_mut(app.world_mut());
        let entity = spawn_character_sprite(&mut commands, &assets, CharacterType::Player, Vec3::ZERO, Vec2::splat(100.0), None, None, &mut *meshes, &mut *materials);
        system_state.apply(app.world_mut());
        app.update();
        let combatant = app.world().get::<Combatant3d>(entity).expect("Should have Combatant3d");
        assert_eq!(combatant.base_rotation, 0.0, "玩家基础旋转应为 0.0");
    }

    // 2. 验证妖狼朝向 (保持 -PI/2)
    {
        let mut system_state: SystemState<(Commands, Res<CharacterAssets>, ResMut<Assets<Mesh>>, ResMut<Assets<StandardMaterial>>)> = SystemState::new(app.world_mut());
        let (mut commands, assets, mut meshes, mut materials) = system_state.get_mut(app.world_mut());
        let entity = spawn_character_sprite(&mut commands, &assets, CharacterType::DemonicWolf, Vec3::ZERO, Vec2::splat(100.0), Some(1), None, &mut *meshes, &mut *materials);
        system_state.apply(app.world_mut());
        app.update();
        let combatant = app.world().get::<Combatant3d>(entity).expect("Should have Combatant3d");
        assert_eq!(combatant.base_rotation, -std::f32::consts::FRAC_PI_2, "妖狼基础旋转应为 -PI/2");
    }

    // 3. 验证毒蛛朝向 (0.0 -> PI)
    {
        let mut system_state: SystemState<(Commands, Res<CharacterAssets>, ResMut<Assets<Mesh>>, ResMut<Assets<StandardMaterial>>)> = SystemState::new(app.world_mut());
        let (mut commands, assets, mut meshes, mut materials) = system_state.get_mut(app.world_mut());
        let entity = spawn_character_sprite(&mut commands, &assets, CharacterType::PoisonSpider, Vec3::ZERO, Vec2::splat(100.0), Some(2), None, &mut *meshes, &mut *materials);
        system_state.apply(app.world_mut());
        app.update();
        let combatant = app.world().get::<Combatant3d>(entity).expect("Should have Combatant3d");
        assert_eq!(combatant.base_rotation, std::f32::consts::PI, "毒蛛基础旋转应为 PI");
    }
}