//! 粒子特效系统

use bevy::prelude::*;
use crate::components::particle::{
    Particle, ParticleEmitter, EmitterConfig, EffectType,
    SpawnEffectEvent, ParticleMarker, EmitterMarker
};
use crate::states::GameState;

#[derive(Resource, Clone)]
pub struct ParticleTexture { pub handle: Handle<Image> }

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
    commands.insert_resource(ParticleTexture { handle: images.add(image) });
}

fn handle_effect_events(mut commands: Commands, texture: Res<ParticleTexture>, mut events: EventReader<SpawnEffectEvent>) {
    for event in events.read() {
        let config = event.effect_type.config();
        if event.burst {
            for _ in 0..event.count {
                let particle = config.spawn_particle(event.position, event.effect_type);
                spawn_particle_entity(&mut commands, &texture, particle);
            }
        } else {
            commands.spawn((
                ParticleEmitter::new(30.0, config, event.effect_type),
                Transform::from_translation(event.position),
                GlobalTransform::default(),
                EmitterMarker,
            ));
        }
    }
}

fn spawn_particle_entity(commands: &mut Commands, texture: &ParticleTexture, particle: Particle) {
    let size = particle.start_size;
    let ui_x = 640.0 + particle.position.x;
    let ui_y = 360.0 - particle.position.y;

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(ui_x - size/2.0), top: Val::Px(ui_y - size/2.0),
            width: Val::Px(size), height: Val::Px(size), ..default()
        },
        ImageNode::new(texture.handle.clone()).with_color(particle.start_color),
        ZIndex(1000), particle, ParticleMarker, crate::plugins::CombatUiRoot,
    ));
}

fn update_emitters(mut commands: Commands, texture: Res<ParticleTexture>, mut emitters: Query<(Entity, &mut ParticleEmitter, &GlobalTransform)>, time: Res<Time>) {
    for (entity, mut emitter, transform) in emitters.iter_mut() {
        emitter.elapsed += time.delta_secs();
        if emitter.duration > 0.0 && emitter.elapsed >= emitter.duration { commands.entity(entity).despawn_recursive(); continue; }
        emitter.timer += time.delta_secs();
        let interval = 1.0 / emitter.rate;
        while emitter.timer >= interval {
            emitter.timer -= interval;
            if emitter.emitted_count >= emitter.max_particles {
                if !emitter.looping { commands.entity(entity).despawn_recursive(); }
                break;
            }
            let particle = emitter.config.spawn_particle(transform.translation(), emitter.effect_type);
            spawn_particle_entity(&mut commands, &texture, particle);
            emitter.emitted_count += 1;
        }
    }
}

fn update_particles(mut commands: Commands, mut query: Query<(Entity, &mut Particle, &mut Node, &mut ImageNode, &mut Visibility)>, time: Res<Time>) {
    let delta = time.delta_secs();
    for (entity, mut p, mut node, mut image, mut visibility) in query.iter_mut() {
        p.elapsed += delta;
        let global_prog = (p.elapsed / p.lifetime).min(1.0);

        if p.effect_type == EffectType::WanJian {
            // --- 大作级：万剑归宗 · 游龙出世逻辑 ---
            // local_prog 决定了飞剑启动和结束的先后顺序，形成龙身
            let local_prog = (global_prog * 1.6 - p.seed * 0.6).clamp(0.0, 1.0);
            
            if local_prog <= 0.0 {
                *visibility = Visibility::Hidden;
                continue;
            } else {
                *visibility = Visibility::Visible;
            }

            // 定义半空集结点 (Hub)
            let hub_pos = Vec2::new(0.0, 250.0);

            if local_prog < 0.3 {
                // 相位一：御剑飞升 (0% - 30%)
                let t = local_prog / 0.3;
                // 从脚下斜向上飞向 Hub 略靠后的位置
                let start = p.start_pos;
                let target = hub_pos + Vec2::new(-100.0, 50.0);
                p.position = start.lerp(target, t);
                
                let move_dir = target - start;
                p.rotation = (-move_dir.y).atan2(move_dir.x);
            } else if local_prog < 0.55 {
                // 相位二：万剑聚阵 (30% - 55%)
                let t = (local_prog - 0.3) / 0.25;
                // 在 Hub 周围进行极速公转，半径不断收缩
                let angle = (t * 6.0 * std::f32::consts::PI) + (p.seed * 6.28);
                let radius = 80.0 * (1.0 - t * 0.5); 
                
                let new_pos = hub_pos + Vec2::new(angle.cos() * radius, angle.sin() * radius);
                let move_vec = new_pos - p.position;
                if move_vec.length() > 0.1 { p.rotation = (-move_vec.y).atan2(move_vec.x); }
                p.position = new_pos;
            } else {
                // 相位三：金龙扫尾 (55% - 100%)
                if let Some(target) = p.target {
                    let strike_t = (local_prog - 0.55) / 0.45;
                    
                    // 游龙轨迹：基础直线 + 正弦波侧向位移
                    let base_dir = (target - hub_pos).normalize();
                    let side_dir = Vec2::new(-base_dir.y, base_dir.x); // 法向量
                    
                    // 波动幅度随时间减小，模拟从游动到直刺的平滑过渡
                    let wave = (strike_t * 12.0 + p.seed * 5.0).sin() * 60.0 * (1.0 - strike_t);
                    
                    let base_pos = hub_pos.lerp(target, strike_t.powi(2)); // 加速度前进
                    let new_pos = base_pos + side_dir * wave;
                    
                    let move_vec = new_pos - p.position;
                    p.rotation = (-move_vec.y).atan2(move_vec.x);
                    p.position = new_pos;
                }
            }
        } else {
            // --- 通用粒子逻辑 ---
            let current_v = p.velocity;
            let grav = p.gravity;
            p.position += current_v * delta;
            p.velocity += grav * delta;
            
            let v_len = p.velocity.length();
            if v_len > 10.0 {
                p.rotation = (-p.velocity.y).atan2(p.velocity.x);
            } else {
                let rs = p.rotation_speed;
                p.rotation += rs * delta;
            }
        }

        let size = p.current_size();
        let ui_x = 640.0 + p.position.x;
        let ui_y = 360.0 - p.position.y;
        node.left = Val::Px(ui_x - size/2.0); node.top = Val::Px(ui_y - size/2.0);
        node.width = Val::Px(size); node.height = Val::Px(size);
        if let Some(mut ec) = commands.get_entity(entity) { ec.insert(Transform::from_rotation(Quat::from_rotation_z(p.rotation))); }
        image.color = p.current_color();
        if p.is_dead() { commands.entity(entity).despawn_recursive(); }
    }
}
