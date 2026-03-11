//! 屏幕特效系统 (高保真色彩修正版)
//!
//! 处理屏幕震动、闪光等全局视觉特效，确保 SDR 下色彩不溢出。

use bevy::prelude::*;
use rand::Rng;
use crate::components::screen_effect::{CameraShake, ScreenFlash, ScreenEffectEvent, ScreenEffectMarker, ScreenWarning};
use crate::components::combat::{CombatUiRoot, Player};
use crate::states::GameState;

pub struct ScreenEffectPlugin;

impl Plugin for ScreenEffectPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ScreenEffectEvent>();
        app.add_systems(Update, (
            handle_screen_effects,
            update_camera_shake,
            update_screen_flash,
            update_screen_warning,
        ).run_if(in_state(GameState::Combat)));
    }
}

fn update_screen_warning(
    time: Res<Time>,
    player_query: Query<&Player>,
    mut warning_query: Query<(&mut Visibility, &mut BackgroundColor), With<ScreenWarning>>,
) {
    if let Ok(player) = player_query.get_single() {
        let is_low_hp = (player.hp as f32 / player.max_hp as f32) < 0.35; 
        let is_weakened = player.weakness > 0;
        
        if let Ok((mut vis, mut color)) = warning_query.get_single_mut() {
            if is_low_hp || is_weakened {
                *vis = Visibility::Visible;
                let alpha = 0.25 + (time.elapsed_secs() * 4.0).sin() * 0.15;
                color.0 = Color::srgba(0.8, 0.0, 0.0, alpha);
            } else {
                *vis = Visibility::Hidden;
            }
        }
    }
}

/// 创建闪光覆盖层 (单实例逻辑，防止颜色叠加变白)
fn spawn_flash_overlay(commands: &mut Commands, color: Color, duration: f32, existing_flash: &Query<Entity, With<ScreenFlash>>) {
    // 1. 彻底清除旧的闪屏实体，防止颜色叠加
    for entity in existing_flash.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // 2. 强制颜色截断，适配 SDR
    let safe_color = match color {
        Color::Srgba(c) => Color::srgba(c.red.clamp(0.0, 1.0), c.green.clamp(0.0, 1.0), c.blue.clamp(0.0, 1.0), c.alpha.clamp(0.0, 1.0)),
        Color::LinearRgba(c) => Color::linear_rgba(c.red.clamp(0.0, 1.0), c.green.clamp(0.0, 1.0), c.blue.clamp(0.0, 1.0), c.alpha.clamp(0.0, 1.0)),
        _ => color,
    };

    // 3. 生成新的覆盖层
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                ..default()
            },
            BackgroundColor(safe_color),
            ZIndex(9999), 
            ScreenFlash::new(safe_color, duration),
            ScreenEffectMarker,
            CombatUiRoot,
        ));
}

/// 处理屏幕特效事件
pub fn handle_screen_effects(
    mut commands: Commands,
    mut events: EventReader<ScreenEffectEvent>,
    camera_query: Query<(Entity, &Transform, Option<&CameraShake>), With<Camera>>, 
    existing_flash: Query<Entity, With<ScreenFlash>>,
) {
    let mut max_trauma = 0.0f32;
    let mut min_decay = 100.0f32;
    let mut accumulated_impulse = Vec2::ZERO;
    let mut has_effect = false;

    for event in events.read() {
        match event {
            ScreenEffectEvent::Shake { trauma, decay } => {
                max_trauma = max_trauma.max(*trauma);
                min_decay = min_decay.min(*decay);
                has_effect = true;
            }
            ScreenEffectEvent::Impact { impulse, duration } => {
                accumulated_impulse += *impulse;
                max_trauma = max_trauma.max(0.4); 
                min_decay = min_decay.min(1.0 / *duration);
                has_effect = true;
            }
            ScreenEffectEvent::Flash { color, duration } => {
                spawn_flash_overlay(&mut commands, *color, *duration, &existing_flash);
            }
        }
    }

    if has_effect {
        for (entity, transform, current_shake) in camera_query.iter() {
            let base_pos = current_shake.and_then(|s| s.base_translation).unwrap_or(transform.translation);
            let final_trauma = current_shake.map(|s| s.trauma.max(max_trauma)).unwrap_or(max_trauma);
            let final_decay = current_shake.map(|s| s.decay.min(min_decay)).unwrap_or(min_decay);
            let final_impulse = current_shake.map(|s| s.impulse + accumulated_impulse).unwrap_or(accumulated_impulse);

            commands.entity(entity).insert(CameraShake {
                trauma: final_trauma,
                decay: final_decay,
                offset: Vec2::ZERO,
                impulse: final_impulse,
                base_translation: Some(base_pos),
            });
        }
    }
}

/// 更新相机震动
pub fn update_camera_shake(
    mut commands: Commands,
    mut query: Query<(Entity, &mut CameraShake, &mut Transform)>,
    time: Res<Time>,
) {
    for (entity, mut shake, mut transform) in query.iter_mut() {
        let delta = time.delta_secs();
        shake.trauma -= shake.decay * delta;
        shake.trauma = shake.trauma.max(0.0);
        let damping = 5.0; 
        shake.impulse = shake.impulse * (1.0 - damping * delta).max(0.0);

        if shake.trauma <= 0.0 && shake.impulse.length() < 0.01 {
            if let Some(base) = shake.base_translation { transform.translation = base; }
            commands.entity(entity).remove::<CameraShake>();
            continue;
        }

        let mut rng = rand::thread_rng();
        let angle = rng.gen::<f32>() * std::f32::consts::PI * 2.0;
        let shake_magnitude = shake.trauma * shake.trauma * 1.5; 
        shake.offset.x = angle.cos() * shake_magnitude + shake.impulse.x;
        shake.offset.y = angle.sin() * shake_magnitude + shake.impulse.y;

        if let Some(base) = shake.base_translation {
            transform.translation = base + Vec3::new(shake.offset.x, shake.offset.y, 0.0);
        }
    }
}

/// 更新屏幕闪光 (同步 Alpha)
fn update_screen_flash(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ScreenFlash, &mut BackgroundColor)>,
    time: Res<Time>,
) {
    for (entity, mut flash, mut bg_color) in query.iter_mut() {
        flash.elapsed += time.delta_secs();
        let alpha = flash.current_alpha().clamp(0.0, 1.0);

        let current_color = bg_color.0;
        bg_color.0 = match current_color {
            Color::Srgba(mut c) => { c.alpha = alpha; Color::Srgba(c) },
            Color::LinearRgba(mut c) => { c.alpha = alpha; Color::LinearRgba(c) },
            _ => current_color,
        };

        if flash.is_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn shake_camera(commands: &mut Commands, trauma: f32) { commands.trigger(ScreenEffectEvent::shake(trauma)); }
pub fn light_shake(commands: &mut Commands) { commands.trigger(ScreenEffectEvent::light_shake()); }
pub fn heavy_shake(commands: &mut Commands) { commands.trigger(ScreenEffectEvent::heavy_shake()); }
pub fn red_flash(commands: &mut Commands, duration: f32) { commands.trigger(ScreenEffectEvent::red_flash(duration)); }
pub fn white_flash(commands: &mut Commands, duration: f32) { commands.trigger(ScreenEffectEvent::white_flash(duration)); }
pub fn flash_screen(commands: &mut Commands, color: Color, duration: f32) { commands.trigger(ScreenEffectEvent::flash(color, duration)); }
