use bevy::prelude::*;
use bevy_card_battler::components::sprite::{CharacterSprite, AnimationState, PlayerSpriteMarker, PlayerAnimationConfig, PlayerWeapon, CharacterAssets, CharacterType};
use bevy_card_battler::systems::sprite::{sync_player_skeletal_animation, spawn_character_sprite};
use bevy_card_battler::resources::PlayerAssets;

#[test]
fn test_visibility_stability_lock() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Mesh>();
    app.init_asset::<StandardMaterial>();
    app.init_asset::<AnimationClip>();
    app.init_asset::<AnimationGraph>();
    app.init_asset::<Scene>();

    // 1. 准备资产
    let assets = CharacterAssets {
        player_anims: vec![Handle::default(), Handle::default(), Handle::default()],
        ..default()
    };
    app.insert_resource(assets.clone());
    
    let player_assets = PlayerAssets {
        body: Handle::default(),
        weapon: Handle::default(),
    };
    app.insert_resource(player_assets.clone());

    // 2. 生成修行者
    let player_entity = {
        let mut system_state = bevy::ecs::system::SystemState::<(
            Commands, Res<CharacterAssets>, ResMut<Assets<Mesh>>, 
            ResMut<Assets<StandardMaterial>>, ResMut<Assets<AnimationGraph>>,
            Res<PlayerAssets>
        )>::new(app.world_mut());
        let (mut commands, assets, mut meshes, mut materials, mut graphs, pa) = system_state.get_mut(app.world_mut());
        
        let e = spawn_character_sprite(
            &mut commands, &assets, CharacterType::Player, 
            Vec3::ZERO, Vec2::splat(100.0), None, None, 
            &mut meshes, &mut materials, &mut graphs, Some(&pa)
        );
        system_state.apply(app.world_mut());
        e
    };

    app.update(); // 同步层级

    let mut weapon_q = app.world_mut().query_filtered::<Entity, With<PlayerWeapon>>();
    let weapon_entity = weapon_q.iter(app.world()).next().expect("应该生成了 PlayerWeapon 实体");

    app.add_systems(Update, sync_player_skeletal_animation);

    // 3. 模拟进入“万剑归宗”状态
    {
        let mut sprite = app.world_mut().get_mut::<CharacterSprite>(player_entity).unwrap();
        sprite.state = AnimationState::ImperialSword;
    }

    // 4. 连续运行多帧，验证稳定性
    for frame in 0..10 {
        app.update();
        let vis = app.world().get::<Visibility>(weapon_entity).unwrap();
        assert_eq!(*vis, Visibility::Hidden, "帧 {}: 武器应该保持锁定隐藏状态", frame);
    }

    // 5. 切换回待机
    {
        let mut sprite = app.world_mut().get_mut::<CharacterSprite>(player_entity).unwrap();
        sprite.state = AnimationState::Idle;
    }
    app.update();

    let vis_after = app.world().get::<Visibility>(weapon_entity).unwrap();
    assert_ne!(*vis_after, Visibility::Hidden, "招式结束后应立即恢复显示");
}
