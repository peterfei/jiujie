//! 背景音乐处理系统
//!
//! # 功能说明
//! - 支持背景音乐播放、停止、切换
//! - 支持淡入淡出效果
//! - 支持音量控制
//! - 支持暂停/恢复
//! - 自动循环播放

use bevy::prelude::*;
use crate::components::background_music::{
    BgmType, PlayBgmEvent, StopBgmEvent, CrossfadeBgmEvent, CurrentBgm, BgmSettings,
};

/// 背景音乐插件
pub struct BackgroundMusicPlugin;

impl Plugin for BackgroundMusicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentBgm>()
            .init_resource::<BgmSettings>()
            .add_event::<PlayBgmEvent>()
            .add_event::<StopBgmEvent>()
            .add_event::<CrossfadeBgmEvent>()
            .add_systems(
                Update,
                (
                    handle_play_bgm_events,
                    handle_stop_bgm_events,
                    handle_crossfade_bgm_events,
                ).chain(),
            );
    }
}

// ============================================================================
// 背景音乐实体标记组件
// ============================================================================

#[derive(Component)]
pub struct BgmMarker {
    pub bgm_type: BgmType,
}

// ============================================================================
// 事件处理系统
// ============================================================================

/// 处理播放背景音乐事件
fn handle_play_bgm_events(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut current_bgm: ResMut<CurrentBgm>,
    bgm_settings: Res<BgmSettings>,
    mut events: EventReader<PlayBgmEvent>,
    existing_bgm_query: Query<(Entity, &BgmMarker), With<AudioSink>>,
) {
    for event in events.read() {
        // 如果BGM被禁用，跳过
        if !bgm_settings.enabled {
            info!("【背景音乐】已禁用，跳过播放: {}", event.bgm_type.chinese_name());
            continue;
        }

        // 检查是否已在播放相同音乐
        if current_bgm.is_playing(event.bgm_type) {
            info!("【背景音乐】已在播放: {}，跳过", event.bgm_type.chinese_name());
            continue;
        }

        // 停止现有背景音乐
        for (entity, marker) in existing_bgm_query.iter() {
            info!("【背景音乐】停止当前播放: {}", marker.bgm_type.chinese_name());
            commands.entity(entity).despawn();
        }

        // 计算实际音量
        let volume = event.volume.unwrap_or(event.bgm_type.default_volume());
        let actual_volume = volume * bgm_settings.master_volume;

        // 创建新的背景音乐实体
        commands.spawn((
            AudioPlayer::new(asset_server.load(event.bgm_type.file_path())),
            PlaybackSettings {
                mode: if event.loop_ {
                    bevy::audio::PlaybackMode::Loop
                } else {
                    bevy::audio::PlaybackMode::Once
                },
                volume: bevy::audio::Volume::new(actual_volume),
                ..default()
            },
            BgmMarker { bgm_type: event.bgm_type },
        ));

        // 更新当前状态
        *current_bgm = CurrentBgm::new(event.bgm_type, volume);

        info!(
            "【背景音乐】开始播放: {} (音量: {:.2}, 淡入: {:.1}s, 循环: {})",
            event.bgm_type.chinese_name(),
            actual_volume,
            event.fade_in,
            event.loop_
        );
    }
}

/// 处理停止背景音乐事件
fn handle_stop_bgm_events(
    mut commands: Commands,
    mut current_bgm: ResMut<CurrentBgm>,
    mut events: EventReader<StopBgmEvent>,
    bgm_query: Query<(Entity, &BgmMarker)>,
) {
    for event in events.read() {
        let mut stopped = false;

        for (entity, marker) in bgm_query.iter() {
            info!(
                "【背景音乐】停止播放: {} (淡出: {:.1}s)",
                marker.bgm_type.chinese_name(),
                event.fade_out
            );
            commands.entity(entity).despawn();
            stopped = true;
        }

        if stopped {
            *current_bgm = CurrentBgm::default();
        }
    }
}

/// 处理切换背景音乐事件（交叉淡出）
fn handle_crossfade_bgm_events(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    current_bgm: ResMut<CurrentBgm>,
    bgm_settings: Res<BgmSettings>,
    mut play_events: EventWriter<PlayBgmEvent>,
    mut events: EventReader<CrossfadeBgmEvent>,
    bgm_query: Query<(Entity, &BgmMarker)>,
) {
    for event in events.read() {
        // 停止当前背景音乐（使用淡出）
        for (entity, marker) in bgm_query.iter() {
            info!(
                "【背景音乐】交叉淡出: {} -> {} ({:.1}s)",
                marker.bgm_type.chinese_name(),
                event.bgm_type.chinese_name(),
                event.duration
            );
            commands.entity(entity).despawn();
        }

        // 播放新背景音乐（使用淡入）
        play_events.send(
            PlayBgmEvent::new(event.bgm_type)
                .with_fade_in(event.duration)
        );
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 根据游戏状态自动选择合适的背景音乐
///
/// 这个函数可以被各个游戏状态系统调用，用于自动切换背景音乐
pub fn auto_select_bgm(
    state: &str,
    is_boss: bool,
    is_tribulation: bool,
) -> BgmType {
    match state {
        "main_menu" => BgmType::MainMenu,
        "map" => BgmType::MapExploration,
        "shop" => BgmType::Shop,
        "rest" => BgmType::Rest,
        "victory" => BgmType::Victory,
        "battle" => {
            if is_tribulation {
                BgmType::Tribulation
            } else if is_boss {
                BgmType::BossBattle
            } else {
                BgmType::NormalBattle
            }
        },
        _ => BgmType::MapExploration, // 默认
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_select_bgm() {
        assert_eq!(auto_select_bgm("main_menu", false, false), BgmType::MainMenu);
        assert_eq!(auto_select_bgm("map", false, false), BgmType::MapExploration);
        assert_eq!(auto_select_bgm("shop", false, false), BgmType::Shop);
        assert_eq!(auto_select_bgm("rest", false, false), BgmType::Rest);
        assert_eq!(auto_select_bgm("victory", false, false), BgmType::Victory);
        assert_eq!(auto_select_bgm("battle", false, false), BgmType::NormalBattle);
        assert_eq!(auto_select_bgm("battle", true, false), BgmType::BossBattle);
        assert_eq!(auto_select_bgm("battle", false, true), BgmType::Tribulation);
    }

    #[test]
    fn test_bgm_type_chinese_names() {
        assert_eq!(BgmType::MainMenu.chinese_name(), "修仙问道");
        assert_eq!(BgmType::MapExploration.chinese_name(), "寻仙觅缘");
        assert_eq!(BgmType::NormalBattle.chinese_name(), "降妖除魔");
        assert_eq!(BgmType::BossBattle.chinese_name(), "生死对决");
        assert_eq!(BgmType::Tribulation.chinese_name(), "雷劫降临");
        assert_eq!(BgmType::Shop.chinese_name(), "坊市繁华");
        assert_eq!(BgmType::Rest.chinese_name(), "修炼打坐");
        assert_eq!(BgmType::Victory.chinese_name(), "众妖伏诛");
    }

    #[test]
    fn test_bgm_type_file_names() {
        assert_eq!(BgmType::MainMenu.file_name(), "main_menu_theme");
        assert_eq!(BgmType::MapExploration.file_name(), "map_exploration_theme");
        assert_eq!(BgmType::NormalBattle.file_name(), "normal_battle_theme");
        assert_eq!(BgmType::BossBattle.file_name(), "boss_battle_theme");
        assert_eq!(BgmType::Tribulation.file_name(), "tribulation_theme");
        assert_eq!(BgmType::Shop.file_name(), "shop_theme");
        assert_eq!(BgmType::Rest.file_name(), "rest_theme");
        assert_eq!(BgmType::Victory.file_name(), "victory_theme");
    }

    #[test]
    fn test_default_volumes() {
        // Boss战和渡劫应该更响亮
        assert!(BgmType::BossBattle.default_volume() > BgmType::NormalBattle.default_volume());
        assert!(BgmType::Tribulation.default_volume() > BgmType::NormalBattle.default_volume());
        // 休息应该更安静
        assert!(BgmType::Rest.default_volume() < BgmType::MapExploration.default_volume());
    }
}
