use bevy::prelude::*;
use bevy_card_battler::components::sprite::{CharacterAssets, CharacterType, Combatant3d, PlayerSpriteMarker, EnemySpriteMarker};
use bevy_card_battler::systems::sprite::{spawn_character_sprite, update_combatant_orientation};
use bevy::ecs::system::SystemState;

#[test]
fn test_character_orientation_algorithm_v7() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    app.init_asset::<Mesh>();
    app.init_asset::<StandardMaterial>();
    app.init_asset::<Scene>();

    let assets = CharacterAssets::default();
    app.insert_resource(assets);

    // 1. 生成玩家和敌人
    {
        let mut system_state: SystemState<(Commands, Res<CharacterAssets>, ResMut<Assets<Mesh>>, ResMut<Assets<StandardMaterial>>)> = SystemState::new(app.world_mut());
        let (mut commands, assets, mut meshes, mut materials) = system_state.get_mut(app.world_mut());
        
        // 玩家在左侧 (-350 -> -3.5)
        spawn_character_sprite(&mut commands, &assets, CharacterType::Player, Vec3::new(-350.0, 0.0, 0.0), Vec2::splat(100.0), None, None, &mut *meshes, &mut *materials, None);
        
        // 敌人在右侧 (350 -> 3.5)
        spawn_character_sprite(&mut commands, &assets, CharacterType::DemonicWolf, Vec3::new(350.0, 0.0, 0.0), Vec2::splat(100.0), Some(1), None, &mut *meshes, &mut *materials, None);
        
        system_state.apply(app.world_mut());
    }

    // 2. 运行算法系统
    let mut schedule = Schedule::default();
    schedule.add_systems(update_combatant_orientation);
    schedule.run(app.world_mut());

    // 3. 验证结果
    let player_rot = {
        let mut player_query = app.world_mut().query_filtered::<&Combatant3d, With<PlayerSpriteMarker>>();
        player_query.get_single(app.world()).expect("Should have player").base_rotation
    };
    
    let enemy_rot = {
        let mut enemy_query = app.world_mut().query_filtered::<&Combatant3d, With<EnemySpriteMarker>>();
        enemy_query.get_single(app.world()).expect("Should have enemy").base_rotation
    };

            // 玩家朝向右侧 (正X方向)，dir = (3.5 - (-3.5), 0, 0) = (7.0, 0, 0)

            // atan2(7.0, 0.0) = PI/2

            assert!((player_rot - std::f32::consts::FRAC_PI_2).abs() < 0.001, "玩家应朝向右侧 (PI/2), 实际: {}", player_rot);

            

            // 敌人朝向左侧 (负X方向)，dir = (-3.5 - 3.5, 0, 0) = (-7.0, 0, 0)

            // atan2(-7.0, 0.0) = -PI/2

            assert!((enemy_rot + std::f32::consts::FRAC_PI_2).abs() < 0.001, "敌人应朝向左侧 (-PI/2), 实际: {}", enemy_rot);

        }

        

    