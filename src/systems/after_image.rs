use bevy::prelude::*;
use bevy::scene::SceneRoot;
use bevy_hanabi::prelude::*;
use crate::components::after_image::{AfterImageConfig, GhostInstance, TrailSource};
use crate::components::particle::EffectType;
use crate::systems::gpu_particle::GpuParticleAssets;
use crate::states::GameState;

pub struct AfterImagePlugin;

#[derive(Resource)]
pub struct GhostMaterialHandle(pub Handle<StandardMaterial>);

impl Plugin for AfterImagePlugin {
    fn build(&self, app: &mut App) {
        // 初始化幽灵材质
        let mut materials = app.world_mut().resource_mut::<Assets<StandardMaterial>>();
        let ghost_mat = materials.add(StandardMaterial {
            base_color: Color::srgba(0.0, 0.5, 1.0, 1.0),
            emissive: LinearRgba::new(500.0, 1000.0, 5000.0, 1.0), // 极端发光，穿透闪屏
            alpha_mode: bevy::prelude::AlphaMode::Add,
            unlit: true,
            ..default()
        });
        app.insert_resource(GhostMaterialHandle(ghost_mat));

        app.add_systems(Update, (
            sync_trail_emitters,
            spawn_after_images,
            apply_ghost_material, // 新增：材质覆盖系统
            update_ghosts,
        ).run_if(in_state(GameState::Combat).or(in_state(GameState::MainMenu))));
    }
}

/// 自动将残影实体下的所有 Mesh 替换为幽灵材质
fn apply_ghost_material(
    ghost_material: Res<GhostMaterialHandle>,
    ghost_query: Query<Entity, Added<GhostInstance>>,
    children_query: Query<&Children>,
    mut material_query: Query<&mut MeshMaterial3d<StandardMaterial>>,
) {
    for ghost_entity in ghost_query.iter() {
        // 递归处理子实体
        replace_material_recursive(ghost_entity, &ghost_material, &children_query, &mut material_query);
    }
}

fn replace_material_recursive(
    entity: Entity,
    material: &GhostMaterialHandle,
    children_query: &Query<&Children>,
    material_query: &mut Query<&mut MeshMaterial3d<StandardMaterial>>,
) {
    if let Ok(mut mat) = material_query.get_mut(entity) {
        mat.0 = material.0.clone();
    }
    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            replace_material_recursive(*child, material, children_query, material_query);
        }
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
            // [大作级优化] 初始膨胀：残影比本体大 15%，增加视觉压迫感
            let base_scale = transform.scale * 1.15;

            let ghost_id = commands.spawn((
                GhostInstance {
                    ttl: Timer::from_seconds(config.ghost_ttl, TimerMode::Once),
                },
                Transform {
                    translation: current_pos,
                    rotation: transform.rotation,
                    scale: base_scale,
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
    mut query: Query<(Entity, &mut GhostInstance, &mut Transform)>,
    time: Res<Time>,
) {
    for (entity, mut ghost, mut transform) in query.iter_mut() {
        ghost.ttl.tick(time.delta());
        
        // [大作级优化] 动态扩散：残影随时间继续向外扩张
        transform.scale *= 1.0 + (0.2 * time.delta_secs());

        if ghost.ttl.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
