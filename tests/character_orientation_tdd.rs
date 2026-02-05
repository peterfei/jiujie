use bevy::prelude::*;
use bevy_card_battler::components::sprite::{CharacterAssets, CharacterType, Combatant3d};
use bevy_card_battler::systems::sprite::spawn_character_sprite;
use bevy::ecs::system::SystemState;

#[test]
fn test_character_orientation_logic_v6() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    app.init_asset::<Mesh>();
    app.init_asset::<StandardMaterial>();
    app.init_asset::<Scene>();

    let assets = CharacterAssets::default();
    app.insert_resource(assets);

    // 1. 验证玩家朝向 (顺时针 90度 -> -PI/2 -> 朝向右侧)
    {
        let mut system_state: SystemState<(Commands, Res<CharacterAssets>, ResMut<Assets<Mesh>>, ResMut<Assets<StandardMaterial>>)> = SystemState::new(app.world_mut());
        let (mut commands, assets, mut meshes, mut materials) = system_state.get_mut(app.world_mut());
        let entity = spawn_character_sprite(&mut commands, &assets, CharacterType::Player, Vec3::ZERO, Vec2::splat(100.0), None, None, &mut *meshes, &mut *materials);
        system_state.apply(app.world_mut());
        app.update();
        let combatant = app.world().get::<Combatant3d>(entity).expect("Should have Combatant3d");
        assert_eq!(combatant.base_rotation, -std::f32::consts::FRAC_PI_2);
    }

    // 2. 验证所有敌人朝向 (逆时针 90度 -> PI/2 -> 朝向左侧)
    let enemy_types = vec![CharacterType::DemonicWolf, CharacterType::PoisonSpider, CharacterType::CursedSpirit, CharacterType::GreatDemon];
    for character_type in enemy_types {
        let mut system_state: SystemState<(Commands, Res<CharacterAssets>, ResMut<Assets<Mesh>>, ResMut<Assets<StandardMaterial>>)> = SystemState::new(app.world_mut());
        let (mut commands, assets, mut meshes, mut materials) = system_state.get_mut(app.world_mut());
        let entity = spawn_character_sprite(&mut commands, &assets, character_type, Vec3::ZERO, Vec2::splat(100.0), Some(1), None, &mut *meshes, &mut *materials);
        system_state.apply(app.world_mut());
        app.update();
        let combatant = app.world().get::<Combatant3d>(entity).expect("Should have Combatant3d");
        assert_eq!(combatant.base_rotation, std::f32::consts::FRAC_PI_2, "{:?} 应面向左侧", character_type);
    }
}