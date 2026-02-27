use bevy::prelude::*;
use bevy::scene::SceneRoot;
use bevy_hanabi::prelude::*;
use crate::components::after_image::{AfterImageConfig, GhostInstance, TrailSource};
use crate::components::particle::EffectType;
use crate::systems::gpu_particle::GpuParticleAssets;
use crate::states::GameState;

pub struct AfterImagePlugin;

impl Plugin for AfterImagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            sync_trail_emitters,
            spawn_after_images,
            update_ghosts,
        ).run_if(in_state(GameState::Combat).or(in_state(GameState::MainMenu))));
    }
}

#[derive(Component, Default)]
pub struct LastPosition(pub Vec3);

pub fn sync_trail_emitters(
    mut commands: Commands,
    gpu_assets: Res<GpuParticleAssets>,
    mut query: Query<(Entity, &Parent, &mut Visibility), With<TrailSource>>,
    config_query: Query<&AfterImageConfig>,
    existing_effects: Query<&ParticleEffect>,
) {
    for (entity, parent, mut visibility) in query.iter_mut() {
        if let Ok(config) = config_query.get(parent.get()) {
            if existing_effects.get(entity).is_err() {
                if let Some(effect_handle) = gpu_assets.effects.get(&EffectType::MovementTrail) {
                    commands.entity(entity).insert(ParticleEffect::new(effect_handle.clone()));
                }
            }
            *visibility = if config.is_active { Visibility::Visible } else { Visibility::Hidden };
        }
    }
}

pub fn spawn_after_images(
    mut commands: Commands,
    mut query: Query<(&Transform, &mut LastPosition, &mut AfterImageConfig, Option<&SceneRoot>)>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();

    for (transform, mut last_pos, mut config, scene_root_opt) in query.iter_mut() {
        let current_pos = transform.translation;
        let mut should_snapshot = false;

        // 1. 强制快照逻辑 (最高优先级，不依赖 delta)
        if config.force_snapshot {
            should_snapshot = true;
            config.force_snapshot = false;
        }

        // 2. 位移触发逻辑
        if delta > 0.0 {
            let displacement = (current_pos - last_pos.0).length();
            let velocity = displacement / delta;
            
            if velocity > config.speed_threshold {
                config.is_active = true;
                config.timer.tick(time.delta());
                if config.timer.just_finished() {
                    should_snapshot = true;
                }
            } else {
                config.is_active = false;
                config.timer.reset();
            }
        }

        if should_snapshot {
            let ghost_id = commands.spawn((
                GhostInstance {
                    ttl: Timer::from_seconds(config.ghost_ttl, TimerMode::Once),
                },
                Transform {
                    translation: current_pos,
                    rotation: transform.rotation,
                    scale: transform.scale,
                },
                GlobalTransform::default(),
                InheritedVisibility::default(),
                crate::components::combat::CombatUiRoot,
            )).id();

            if let Some(root) = scene_root_opt {
                let model_clone = commands.spawn(SceneRoot(root.0.clone())).id();
                commands.entity(ghost_id).add_child(model_clone);
            }
        }

        last_pos.0 = current_pos;
    }
}

pub fn update_ghosts(
    mut commands: Commands,
    mut query: Query<(Entity, &mut GhostInstance)>,
    time: Res<Time>,
) {
    for (entity, mut ghost) in query.iter_mut() {
        ghost.ttl.tick(time.delta());
        if ghost.ttl.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
