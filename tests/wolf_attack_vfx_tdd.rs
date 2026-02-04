
#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy_card_battler::components::particle::{EffectType, SpawnEffectEvent};
    use bevy_card_battler::components::sprite::{
        CharacterAnimationEvent, AnimationState, PhysicalImpact, SpriteMarker, CharacterSprite, 
        CharacterType, ActionType, BreathAnimation
    };
    use bevy_card_battler::systems::sprite::update_physical_impacts;
    
    // Mock resources
    #[derive(Resource)]
    struct VictoryDelay(f32);

    #[test]
    fn test_wolf_attack_particle_position_tdd() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<SpawnEffectEvent>();
        app.add_event::<CharacterAnimationEvent>();
        app.add_event::<bevy_card_battler::components::ScreenEffectEvent>(); // Required by system
        
        // Mock time
        app.init_resource::<Time>();

        // Add the system under test
        app.add_systems(Update, update_physical_impacts);

        // Setup test entities
        // 1. Spawn a Wolf at 3D position X=3.5 (Right side)
        // Note: The system assumes the entity has PhysicalImpact
        let wolf_start_pos_3d = Vec3::new(3.5, 0.8, 0.0);
        let wolf = app.world_mut().spawn((
            Transform::from_translation(wolf_start_pos_3d),
            PhysicalImpact { 
                home_position: wolf_start_pos_3d, 
                action_type: ActionType::WolfPounce, 
                // Set action_timer so it simulates being "mid-jump" close to landing
                // Total time 0.8s. Landing happens > 0.95 normalized time.
                // So action_timer should be small (close to 0).
                action_timer: 0.01, 
                target_offset_dist: 7.0, // Distance from 3.5 to -3.5 is 7.0
                current_offset: Vec3::new(-6.9, 0.0, 0.0), // Almost at target (-3.5)
                action_stage: 0, // Ready to land
                ..default() 
            },
            BreathAnimation::default(),
        )).id();

        let mut found_impact = false;

        // Run the app for a few frames to trigger the system
        for _ in 0..5 {
            app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(16));
            app.update();

            // Check captured events immediately after update
            let events = app.world().resource::<Events<SpawnEffectEvent>>();
            let mut reader = events.get_reader();
            
            for event in reader.read(events) {
                if event.effect_type == EffectType::ImpactSpark {
                    found_impact = true;
                    
                    assert!(
                        event.position.x.abs() > 100.0, 
                        "Particle position X ({}) is too small! It seems to be in 3D coordinates (-3.5) instead of UI coordinates (-350.0). This causes it to appear in the center of the screen.", 
                        event.position.x
                    );
                }
            }
        }

        assert!(found_impact, "Should have spawned an ImpactSpark particle");
    }
}
