//! 音效处理系统

use bevy::prelude::*;
use crate::components::audio::{PlaySfxEvent, SfxType};

pub struct SfxPlugin;

impl Plugin for SfxPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlaySfxEvent>();
        app.add_systems(Update, handle_sfx_events);
    }
}

/// 监听音效事件并播放
fn handle_sfx_events(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut events: EventReader<PlaySfxEvent>,
) {
    for event in events.read() {
        // TODO: 暂时使用 .mp3 格式进行效果验证，后续需统一转换为 .ogg 格式
        let sound_path = match event.sfx_type {
            SfxType::CardPlay => "audio/sfx/player_hit.mp3",
            SfxType::LightningStrike => "audio/sfx/big-thunder-clap.mp3",
            SfxType::BreakthroughSuccess => "audio/sfx/breakthrough.ogg",
            SfxType::PlayerHit => "audio/sfx/player_hit.ogg",
            SfxType::EnemyHit => "audio/sfx/enemy_hit.ogg",
            SfxType::UiClick => "audio/sfx/ui_click.ogg",
        };

        // 在 Bevy 0.15 中，播放一次性音效的标准做法
        commands.spawn((
            AudioPlayer::new(asset_server.load(sound_path)),
            PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Despawn, // 播放完自动销毁实体
                volume: bevy::audio::Volume::new(event.volume),
                ..default()
            },
        ));
        
        info!("【音效】触发播放: {:?} (路径: {})", event.sfx_type, sound_path);
    }
}
