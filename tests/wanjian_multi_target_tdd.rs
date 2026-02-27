use bevy::prelude::*;
use bevy_card_battler::components::particle::{EffectType, SpawnEffectEvent, Particle};
use bevy_card_battler::components::sprite::EnemySpriteMarker;
use bevy_card_battler::systems::vfx_orchestrator::{handle_vfx_events, ParticleAssets};
use bevy_card_battler::states::GameState;

#[test]
fn test_wanjian_target_distribution() {
    let mut app = App::new();
    
    app.add_plugins(MinimalPlugins);
    app.init_state::<GameState>();
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    
    app.add_event::<SpawnEffectEvent>();
    app.add_event::<bevy_card_battler::components::screen_effect::ScreenEffectEvent>();

    // Mock assets
    app.insert_resource(Assets::<Image>::default());
    app.insert_resource(Assets::<Mesh>::default());
    app.insert_resource(Assets::<StandardMaterial>::default());
    
    let dummy_image = app.world_mut().resource_mut::<Assets<Image>>().add(Image::default());
    app.insert_resource(ParticleAssets {
        textures: std::collections::HashMap::new(),
        default_texture: dummy_image,
    });

    // 1. Spawn 3 enemies
    let e1 = app.world_mut().spawn((
        Transform::from_xyz(10.0, 0.0, 0.0),
        EnemySpriteMarker { id: 1 },
    )).id();
    let e2 = app.world_mut().spawn((
        Transform::from_xyz(-10.0, 0.0, 0.0),
        EnemySpriteMarker { id: 2 },
    )).id();
    let e3 = app.world_mut().spawn((
        Transform::from_xyz(0.0, 0.0, 10.0),
        EnemySpriteMarker { id: 3 },
    )).id();

    // 2. Add handle_vfx_events system
    app.add_systems(Update, handle_vfx_events);

    // 3. Send WanJian event with count 12
    app.world_mut().send_event(SpawnEffectEvent::new(EffectType::WanJian, Vec3::ZERO).burst(12));

    // 4. Update
    app.update();

    // 5. Check particle targets
    let mut query = app.world_mut().query::<&Particle>();
    let mut target_entities = std::collections::HashSet::new();
    let mut count = 0;
    
    for particle in query.iter(app.world()) {
        count += 1;
        if let Some(target_entity) = particle.target_entity {
            target_entities.insert(target_entity);
        }
    }

    assert_eq!(count, 12, "Should have spawned 12 particles");
    // We expect the system to automatically distribute targets among available enemies
    assert!(target_entities.contains(&e1), "Enemy 1 should be targeted");
    assert!(target_entities.contains(&e2), "Enemy 2 should be targeted");
    assert!(target_entities.contains(&e3), "Enemy 3 should be targeted");
    assert_eq!(target_entities.len(), 3, "Should have targeted all 3 enemies");
}
