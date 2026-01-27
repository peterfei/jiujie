//! 粒子特效系统

use bevy::prelude::*;
use crate::components::particle::{
    Particle, ParticleEmitter, EmitterConfig, EffectType,
    SpawnEffectEvent, ParticleMarker, EmitterMarker
};
use crate::states::GameState;

use std::collections::HashMap;

#[derive(Resource, Clone)]
pub struct ParticleAssets {
    pub textures: HashMap<EffectType, Handle<Image>>,
    pub default_texture: Handle<Image>,
}

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

fn setup_particle_texture(mut commands: Commands, asset_server: Res<AssetServer>, mut images: ResMut<Assets<Image>>) {
    let mut textures = HashMap::new();
    
    // 加载专属贴图
    textures.insert(EffectType::WanJian, asset_server.load("textures/cards/sword.png"));
    textures.insert(EffectType::WebShot, asset_server.load("textures/web_effect.png"));
    textures.insert(EffectType::SwordEnergy, asset_server.load("textures/cards/sword.png"));
    
    // 默认贴图（1x1 白色）
    let default_texture = images.add(Image::default());
    
    commands.insert_resource(ParticleAssets { 
        textures,
        default_texture,
    });
}

fn handle_effect_events(mut commands: Commands, assets: Res<ParticleAssets>, mut events: EventReader<SpawnEffectEvent>) {
    for event in events.read() {
        let config = event.effect_type.config();
        if event.burst {
            for _ in 0..event.count {
                let mut particle = config.spawn_particle(event.position, event.effect_type);
                if let Some(target) = event.target {
                    particle.target = Some(target);
                }
                spawn_particle_entity(&mut commands, &assets, particle);
            }
        } else {
            commands.spawn((
                ParticleEmitter::new(30.0, config).with_type(event.effect_type),
                Transform::from_translation(event.position),
                GlobalTransform::default(),
                EmitterMarker,
            ));
        }
    }
}

fn spawn_particle_entity(commands: &mut Commands, assets: &ParticleAssets, particle: Particle) {
    let size = particle.start_size;
    let ui_x = 640.0 + particle.position.x;
    let ui_y = 360.0 - particle.position.y;

    let handle = assets.textures.get(&particle.effect_type).unwrap_or(&assets.default_texture).clone();
    
    let (w, h) = if particle.effect_type == EffectType::WanJian {
        (size, size * 4.0)
    } else {
        (size, size)
    };

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(ui_x - w/2.0), top: Val::Px(ui_y - h/2.0),
            width: Val::Px(w), height: Val::Px(h), ..default()
        },
        ImageNode::new(handle).with_color(particle.start_color),
        ZIndex(5), particle, ParticleMarker,
    ));
}

fn update_emitters(mut commands: Commands, assets: Res<ParticleAssets>, mut emitters: Query<(Entity, &mut ParticleEmitter, &GlobalTransform)>, time: Res<Time>) {
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
            spawn_particle_entity(&mut commands, &assets, particle);
            emitter.emitted_count += 1;
        }
    }
}

use crate::components::screen_effect::ScreenEffectEvent;

fn update_particles(
    mut commands: Commands, 
    mut query: Query<(Entity, &mut Particle, &mut Node, &mut ImageNode, &mut Visibility)>, 
    time: Res<Time>, 
    mut events: EventWriter<SpawnEffectEvent>,
    mut screen_events: EventWriter<ScreenEffectEvent>,
) {
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
                // 相位三：游龙突刺 (55% - 100%)
                // 视觉：飞剑划出极长且带残影的流光，不再是正弦波，而是带有弧度的切向突刺
                if let Some(target) = p.target {
                    let strike_t = (local_prog - 0.55) / 0.45;

                    // --- 大作级：在冲击开始瞬间触发视觉补偿 ---
                    if strike_t > 0.0 && strike_t < 0.05 && p.seed < 0.1 {
                        screen_events.send(ScreenEffectEvent::shake(0.4));
                        if p.seed < 0.03 { screen_events.send(ScreenEffectEvent::white_flash(0.1)); }
                    }

                    // 游龙轨迹：切向弧形突刺
                    let base_dir = (target - hub_pos).normalize();
                    let side_dir = Vec2::new(-base_dir.y, base_dir.x);

                    // 计算切向弧度：基于粒子种子的偏移，形成龙身般的曲线
                    // 使用三次贝塞尔曲线实现平滑的弧形轨迹
                    let arc_offset = (p.seed - 0.5) * 200.0; // 种子决定弧度方向和大小
                    let control_point = hub_pos.lerp(target, 0.5) + side_dir * arc_offset;

                    // 二次贝塞尔插值：B(t) = (1-t)²P0 + 2(1-t)tP1 + t²P2
                    let inv_t = 1.0 - strike_t;
                    let curve_pos = hub_pos * inv_t * inv_t
                        + control_point * 2.0 * inv_t * strike_t
                        + target * strike_t * strike_t;

                    p.position = curve_pos;

                    // 旋转角度朝向运动方向
                    let move_dir = (target - p.position).normalize();
                    p.rotation = (-move_dir.y).atan2(move_dir.x);

                    // --- 极长流光拖尾：增加残影密度 ---
                    if strike_t > 0.0 && strike_t < 1.0 {
                        // 增加拖尾粒子密度，从每帧1个增加到2-3个
                        events.send(SpawnEffectEvent::new(EffectType::SwordEnergy, p.position.extend(0.6)).burst(1));
                        // 第二层残影，略微延迟
                        if strike_t > 0.1 {
                            events.send(SpawnEffectEvent::new(EffectType::SwordEnergy, p.position.extend(0.5)).burst(1));
                        }
                    }

                    // --- 撞击火花：击中敌人时触发高亮度微小爆炸 ---
                    // 当接近目标时（最后10%的行程），触发撞击火花
                    if strike_t > 0.90 && strike_t < 0.95 {
                        // 每把剑击中时生成多个火花粒子
                        events.send(SpawnEffectEvent::new(EffectType::ImpactSpark, target.extend(0.0)).burst(8));
                    }
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
        let (w, h) = if p.effect_type == EffectType::WanJian {
            (size, size * 4.0)
        } else {
            (size, size)
        };
        let ui_x = 640.0 + p.position.x;
        let ui_y = 360.0 - p.position.y;
        node.left = Val::Px(ui_x - w/2.0); node.top = Val::Px(ui_y - h/2.0);
        node.width = Val::Px(w); node.height = Val::Px(h);
        // 5. 死亡检查与销毁 (放最后且确保逻辑闭环)
        if p.is_dead() {
            commands.entity(entity).despawn_recursive();
            continue; // 彻底跳过当前实体后续逻辑，防止 Panic
        }

        // 6. 应用视觉状态
        image.color = p.current_color();
        
        let final_rotation = if p.effect_type == EffectType::WanJian || p.effect_type == EffectType::SwordEnergy {
            p.rotation - std::f32::consts::PI / 2.0 // 修正纵向贴图
        } else {
            p.rotation
        };

        if let Some(mut ec) = commands.get_entity(entity) { 
            ec.insert(Transform::from_rotation(Quat::from_rotation_z(final_rotation))); 
        }
    }
}
