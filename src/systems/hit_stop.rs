use bevy::prelude::*;
use crate::components::hit_stop::{HitStopEvent, HitStopState};

pub struct HitStopPlugin;

impl Plugin for HitStopPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<HitStopEvent>()
           .init_resource::<HitStopState>()
           .add_systems(Update, (
               handle_hit_stop_events,
               update_hit_stop_timer,
           ).chain()); 
    }
}

use crate::components::screen_effect::ScreenEffectEvent;
use crate::components::after_image::AfterImageConfig;

pub fn handle_hit_stop_events(
    mut events: EventReader<HitStopEvent>,
    mut state: ResMut<HitStopState>,
    mut time: ResMut<Time<Virtual>>,
    mut screen_events: EventWriter<ScreenEffectEvent>,
    mut after_image_query: Query<&mut AfterImageConfig>,
) {
    for event in events.read() {
        state.timer.set_duration(std::time::Duration::from_secs_f32(event.duration));
        state.timer.reset();
        time.set_relative_speed(event.speed.clamp(0.0, 1.0));
        state.is_active = true;
        
        // [视觉引导] 顿帧瞬间伴随高频闪白
        screen_events.send(ScreenEffectEvent::white_flash(0.1));
        
        // [身法联动] 顿帧瞬间强制产生残影
        for mut config in after_image_query.iter_mut() {
            config.force_snapshot = true;
        }
        
        info!("【打击感】触发顿帧：时长 {}s, 速度缩放 {}", event.duration, event.speed);
    }
}

pub fn update_hit_stop_timer(
    mut state: ResMut<HitStopState>,
    mut time: ResMut<Time<Virtual>>,
    time_real: Res<Time<Real>>,
) {
    if !state.is_active { return; }

    state.timer.tick(time_real.delta());

    if state.timer.finished() {
        time.set_relative_speed(1.0);
        state.is_active = false;
    }
}
