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
    camera_query: Query<Entity, (With<Camera2d>, Without<CameraShake>)>,
) {
    for event in events.read() {
        match event {
            ScreenEffectEvent::Shake { trauma, decay } => {
                if let Ok(camera) = camera_query.get_single() {
                    commands.entity(camera).insert(CameraShake::new(*trauma).with_decay(*decay));
                    info!("触发屏幕震动: 强度={}", trauma);
                }
            }
            ScreenEffectEvent::Flash { color, duration } => {
                spawn_flash_overlay(&mut commands, *color, *duration);
                info!("触发屏幕闪光: 颜色={:?}, 持续={:?}", color, duration);
            }
        }
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
            GlobalTransform::default(),
            Transform::from_translation(Vec3::new(0.0, 0.0, 1000.0)), // 使用Z轴位置而非z_index
            ScreenFlash::new(flash_color, duration),
            ScreenEffectMarker,
        ));
}

/// 更新相机震动
fn update_camera_shake(
    mut commands: Commands,
    mut query: Query<(Entity, &mut CameraShake, &mut Transform)>,
    time: Res<Time>,
) {
    for (entity, mut shake, mut transform) in query.iter_mut() {
        // 衰减震动强度
        shake.trauma -= shake.decay * time.delta_secs();
        shake.trauma = shake.trauma.max(0.0);

        if shake.trauma <= 0.0 {
            // 震动结束，恢复相机位置
            transform.translation.x = 0.0;
            transform.translation.y = 0.0;
            commands.entity(entity).remove::<CameraShake>();
            continue;
        }

        // 使用 Perlin 噪声生成平滑的震动
        let mut rng = rand::thread_rng();
        let angle = rng.gen::<f32>() * std::f32::consts::PI * 2.0;
        let magnitude = shake.trauma * shake.trauma * 20.0; // 非线性增强

        shake.offset.x = angle.cos() * magnitude;
        shake.offset.y = angle.sin() * magnitude;

        // 应用震动偏移
        transform.translation.x = shake.offset.x;
        transform.translation.y = shake.offset.y;
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
