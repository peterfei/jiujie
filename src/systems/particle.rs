//! 粒子特效系统
//!
//! 处理粒子发射、更新和渲染

use bevy::prelude::*;
use crate::components::particle::{
    Particle, ParticleEmitter, EmitterConfig, EffectType,
    SpawnEffectEvent, ParticleMarker, EmitterMarker
};
use crate::states::GameState;

/// 粒子白色纹理资源（用于粒子渲染）
#[derive(Resource, Clone)]
pub struct ParticleTexture {
    pub handle: Handle<Image>,
}

/// 粒子特效插件
pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnEffectEvent>();
        app.add_systems(Startup, setup_particle_texture);
        app.add_systems(
            Update,
            (
                handle_effect_events,
                update_emitters,
                update_particles,
            ).run_if(in_state(GameState::Combat)
                .or(in_state(GameState::Reward))
                .or(in_state(GameState::Tribulation))
            )
        );
    }
}

fn setup_particle_texture(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let image = Image::default();
    let handle = images.add(image);
    commands.insert_resource(ParticleTexture { handle });
}

fn handle_effect_events(
    mut commands: Commands,
    texture: Res<ParticleTexture>,
    mut events: EventReader<SpawnEffectEvent>,
) {
    for event in events.read() {
        let config = event.effect_type.config();
        if event.burst {
            spawn_particle_burst(&mut commands, &texture, event.position, &config, event.count);
        } else {
            spawn_emitter(&mut commands, event.position, config);
        }
    }
}

fn spawn_particle_burst(
    commands: &mut Commands,
    texture: &ParticleTexture,
    position: Vec3,
    config: &EmitterConfig,
    count: usize,
) {
    for _ in 0..count {
        let particle = config.spawn_particle(position);
        spawn_particle_entity(commands, texture, particle);
    }
}

fn spawn_emitter(
    commands: &mut Commands,
    position: Vec3,
    config: EmitterConfig,
) {
    commands
        .spawn((
            ParticleEmitter::new(30.0, config),
            Transform::from_translation(position),
            GlobalTransform::default(),
            EmitterMarker,
        ));
}

fn spawn_particle_entity(
    commands: &mut Commands,
    texture: &ParticleTexture,
    particle: Particle,
) {
    let size = particle.start_size;
    let color = particle.start_color;

    // 世界坐标 -> UI 坐标
    let ui_x = 640.0 + particle.position.x;
    let ui_y = 360.0 - particle.position.y;

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(ui_x - size/2.0),
                top: Val::Px(ui_y - size/2.0),
                width: Val::Px(size),
                height: Val::Px(size),
                ..default()
            },
            ImageNode::new(texture.handle.clone()).with_color(color),
            ZIndex(1000), 
            particle,
            ParticleMarker,
            crate::plugins::CombatUiRoot,
        ));
}

fn update_emitters(
    mut commands: Commands,
    texture: Res<ParticleTexture>,
    mut emitters_query: Query<(Entity, &mut ParticleEmitter, &GlobalTransform)>,
    time: Res<Time>,
) {
    for (entity, mut emitter, transform) in emitters_query.iter_mut() {
        emitter.elapsed += time.delta_secs();
        if emitter.duration > 0.0 && emitter.elapsed >= emitter.duration {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        emitter.timer += time.delta_secs();
        let interval = 1.0 / emitter.rate;

        while emitter.timer >= interval {
            emitter.timer -= interval;
            if emitter.emitted_count >= emitter.max_particles {
                if !emitter.looping { commands.entity(entity).despawn_recursive(); }
                break;
            }

            let particle = emitter.config.spawn_particle(transform.translation());
            spawn_particle_entity(&mut commands, &texture, particle);
            emitter.emitted_count += 1;
        }
    }
}

fn update_particles(
    mut commands: Commands,
    mut particles_query: Query<(Entity, &mut Particle, &mut Node, &mut ImageNode)>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    for (entity, mut particle, mut node, mut image) in particles_query.iter_mut() {
        particle.elapsed += delta;

        // 1. 先提取逻辑数值，避免借用冲突
        let velocity = particle.velocity;
        let gravity = particle.gravity;
        let rotation_speed = particle.rotation_speed;

        // 2. 更新内部逻辑状态
        particle.position += velocity * delta;
        particle.velocity += gravity * delta;
        particle.rotation += rotation_speed * delta;

        // 3. 同步到 UI 坐标
        let size = particle.current_size();
        let ui_x = 640.0 + particle.position.x;
        let ui_y = 360.0 - particle.position.y;

        node.left = Val::Px(ui_x - size/2.0);
        node.top = Val::Px(ui_y - size/2.0);
        node.width = Val::Px(size);
        node.height = Val::Px(size);

        image.color = particle.current_color();

        if particle.is_dead() {
            commands.entity(entity).despawn_recursive();
        }
    }
}