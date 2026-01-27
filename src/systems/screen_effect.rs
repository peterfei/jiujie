//! 屏幕特效系统
//!
//! 处理屏幕震动、闪光等全局视觉特效

use bevy::prelude::*;
use rand::Rng;
use crate::components::screen_effect::{
    CameraShake, ScreenFlash, ScreenEffectEvent, ScreenEffectMarker
};
use crate::states::GameState;

/// 屏幕特效插件
pub struct ScreenEffectPlugin;

impl Plugin for ScreenEffectPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ScreenEffectEvent>();
        app.add_systems(
            Update,
            (
                handle_screen_effects,
                update_camera_shake,
                update_screen_flash,
            ).run_if(in_state(GameState::Combat)
                .or(in_state(GameState::Reward))
                .or(in_state(GameState::Tribulation))
            )
        );
    }
}

/// 处理屏幕特效事件
fn handle_screen_effects(
    mut commands: Commands,
    mut events: EventReader<ScreenEffectEvent>,
    camera_query: Query<(Entity, &Transform, Option<&CameraShake>), With<Camera>>, 
) {
    // 1. 预处理：找出本帧收到的最强震动请求
    let mut max_trauma = 0.0f32;
    let mut min_decay = 100.0f32;
    let mut has_shake = false;

    for event in events.read() {
        match event {
            ScreenEffectEvent::Shake { trauma, decay } => {
                max_trauma = max_trauma.max(*trauma);
                min_decay = min_decay.min(*decay);
                has_shake = true;
            }
            ScreenEffectEvent::Flash { color, duration } => {
                spawn_flash_overlay(&mut commands, *color, *duration);
                info!("触发屏幕闪光: 颜色={:?}, 持续={:?}", color, duration);
            }
        }
    }

    // 2. 应用最强震动到所有相机
    if has_shake {
        for (entity, transform, current_shake) in camera_query.iter() {
            let mut final_trauma = max_trauma;
            let mut final_decay = min_decay;
            
            if let Some(existing) = current_shake {
                // 如果当前正在震动，则进行增量合并
                final_trauma = existing.trauma.max(max_trauma);
                final_decay = existing.decay.min(min_decay);
            }
            
            // 记录基础坐标并插入新震动
            let base_pos = current_shake.and_then(|s| s.base_translation).unwrap_or(transform.translation);
            commands.entity(entity).insert(CameraShake {
                trauma: final_trauma,
                decay: final_decay,
                offset: Vec2::ZERO,
                base_translation: Some(base_pos),
            });
        }
        info!("【视觉】触发全场景强震: 强度={:.2}", max_trauma);
    }
}

/// 创建闪光覆盖层
fn spawn_flash_overlay(commands: &mut Commands, color: Color, duration: f32) {
    let flash_color = color;
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
            BackgroundColor(flash_color),
            ZIndex(2000), // 使用极高的 ZIndex 确保覆盖所有 UI 和粒子
            ScreenFlash::new(flash_color, duration),
            ScreenEffectMarker,
            crate::components::CombatUiRoot, // 标记以便在清理战斗时自动移除
        ));
}

/// 更新相机震动
fn update_camera_shake(
    mut commands: Commands,
    mut query: Query<(Entity, &mut CameraShake, &mut Transform)>,
    time: Res<Time>,
) {
    for (entity, mut shake, mut transform) in query.iter_mut() {
        shake.trauma -= shake.decay * time.delta_secs();
        shake.trauma = shake.trauma.max(0.0);

        if shake.trauma <= 0.0 {
            // 震动结束，精准恢复初始位置 (无论是 2D 还是 3D)
            if let Some(base) = shake.base_translation {
                transform.translation = base;
            }
            commands.entity(entity).remove::<CameraShake>();
            continue;
        }

        let mut rng = rand::thread_rng();
        let angle = rng.gen::<f32>() * std::f32::consts::PI * 2.0;
        
        // 大作级 3D 震动：调大位移系数到 1.2
        let magnitude = shake.trauma * shake.trauma * 1.2; 

        shake.offset.x = angle.cos() * magnitude;
        shake.offset.y = angle.sin() * magnitude;

        // 在基础位置上应用偏移
        if let Some(base) = shake.base_translation {
            transform.translation = base + Vec3::new(shake.offset.x, shake.offset.y, 0.0);
        }
    }
}

/// 更新屏幕闪光
fn update_screen_flash(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ScreenFlash, &mut BackgroundColor)>,
    time: Res<Time>,
) {
    for (entity, mut flash, mut bg_color) in query.iter_mut() {
        flash.elapsed += time.delta_secs();

        // 更新透明度
        let alpha = flash.current_alpha();

        // 更新背景颜色
        if let Color::Srgba(mut srgba) = bg_color.0 {
            srgba.alpha = alpha.clamp(0.0, 1.0);
            bg_color.0 = Color::Srgba(srgba);
        }

        // 检查是否完成
        if flash.is_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// 辅助函数：触发屏幕震动
pub fn shake_camera(commands: &mut Commands, trauma: f32) {
    commands.trigger(ScreenEffectEvent::shake(trauma));
}

/// 辅助函数：触发轻震动
pub fn light_shake(commands: &mut Commands) {
    commands.trigger(ScreenEffectEvent::light_shake());
}

/// 辅助函数：触发强震动
pub fn heavy_shake(commands: &mut Commands) {
    commands.trigger(ScreenEffectEvent::heavy_shake());
}

/// 辅助函数：触发红色闪光
pub fn red_flash(commands: &mut Commands, duration: f32) {
    commands.trigger(ScreenEffectEvent::red_flash(duration));
}

/// 辅助函数：触发白色闪光
pub fn white_flash(commands: &mut Commands, duration: f32) {
    commands.trigger(ScreenEffectEvent::white_flash(duration));
}

/// 辅助函数：触发自定义闪光
pub fn flash_screen(commands: &mut Commands, color: Color, duration: f32) {
    commands.trigger(ScreenEffectEvent::flash(color, duration));
}
