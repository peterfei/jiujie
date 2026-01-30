//! 音效处理系统
//!
//! # 功能说明
//! - 支持一次性音效播放
//! - 支持音量控制
//! - 支持音效分类开关
//! - 自动清理播放完成的实体

use bevy::prelude::*;
use crate::components::audio::{PlaySfxEvent, SfxType, SfxSettings};

/// 音效插件
pub struct SfxPlugin;

impl Plugin for SfxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SfxSettings>()
            .add_event::<PlaySfxEvent>()
            .add_systems(Update, handle_sfx_events);
    }
}

// ============================================================================
// 音效实体标记组件
// ============================================================================

#[derive(Component)]
pub struct SfxMarker {
    pub sfx_type: SfxType,
}

// ============================================================================
// 事件处理系统
// ============================================================================

/// 监听音效事件并播放
fn handle_sfx_events(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    sfx_settings: Res<SfxSettings>,
    mut events: EventReader<PlaySfxEvent>,
) {
    for event in events.read() {
        // 如果音效被禁用，跳过
        if !sfx_settings.enabled {
            continue;
        }

        // 检查分类开关
        let category = event.sfx_type.category();
        match category {
            "UI交互" if !sfx_settings.ui_enabled => continue,
            "战斗相关" | "法术技能" | "大招技能" | "敌人相关"
                if !sfx_settings.combat_enabled => continue,
            _ => {}
        }

        // 获取音效文件路径
        let sound_path = event.sfx_type.file_path();

        // 计算实际音量
        let base_volume = event.get_volume();
        let actual_volume = base_volume * sfx_settings.master_volume;

        // 播放音效
        commands.spawn((
            AudioPlayer::new(asset_server.load(sound_path)),
            PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Despawn,
                volume: bevy::audio::Volume::new(actual_volume),
                ..default()
            },
            SfxMarker { sfx_type: event.sfx_type },
        ));

        info!(
            "【音效】触发播放: {} (路径: {}, 分类: {})",
            event.sfx_type.chinese_name(),
            sound_path,
            category
        );
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sfx_type_chinese_names() {
        // 测试部分音效中文名称
        assert_eq!(SfxType::CardPlay.chinese_name(), "出牌");
        assert_eq!(SfxType::PlayerAttack.chinese_name(), "玩家攻击");
        assert_eq!(SfxType::LightningStrike.chinese_name(), "天雷落下");
        assert_eq!(SfxType::ThousandSwords.chinese_name(), "万剑归宗");
        assert_eq!(SfxType::UiClick.chinese_name(), "UI点击");
        assert_eq!(SfxType::Victory.chinese_name(), "战斗胜利");
        assert_eq!(SfxType::BossAppear.chinese_name(), "Boss登场");
    }

    #[test]
    fn test_sfx_type_file_names() {
        assert_eq!(SfxType::CardPlay.file_name(), "card_play");
        assert_eq!(SfxType::PlayerAttack.file_name(), "player_attack");
        assert_eq!(SfxType::LightningStrike.file_name(), "lightning_strike");
        assert_eq!(SfxType::ThousandSwords.file_name(), "thousand_swords");
    }

    #[test]
    fn test_sfx_type_categories() {
        assert_eq!(SfxType::CardPlay.category(), "卡牌相关");
        assert_eq!(SfxType::PlayerAttack.category(), "战斗相关");
        assert_eq!(SfxType::LightningStrike.category(), "法术技能");
        assert_eq!(SfxType::ThousandSwords.category(), "大招技能");
        assert_eq!(SfxType::UiClick.category(), "UI交互");
        assert_eq!(SfxType::Victory.category(), "系统事件");
        assert_eq!(SfxType::BossAppear.category(), "敌人相关");
    }

    #[test]
    fn test_default_volumes() {
        // UI音效较轻
        assert!(SfxType::UiHover.default_volume() < 0.5);
        // 大招音效较响
        assert!(SfxType::ThousandSwords.default_volume() > 0.8);
        assert!(SfxType::BossAppear.default_volume() > 0.8);
    }

    #[test]
    fn test_recommended_durations() {
        let (min, max) = SfxType::UiHover.recommended_duration();
        assert!(min < 0.2 && max < 0.3);

        let (min, max) = SfxType::ThousandSwords.recommended_duration();
        assert!(min >= 3.0 && max <= 5.0);
    }

    #[test]
    fn test_play_sfx_event_creation() {
        // 默认事件
        let event = PlaySfxEvent::new(SfxType::CardPlay);
        assert_eq!(event.sfx_type, SfxType::CardPlay);
        assert_eq!(event.volume, None);

        // 自定义音量
        let event = PlaySfxEvent::with_volume(SfxType::CardPlay, 0.8);
        assert_eq!(event.sfx_type, SfxType::CardPlay);
        assert_eq!(event.volume, Some(0.8));

        // 获取实际音量
        assert_eq!(event.get_volume(), 0.8);
        assert_eq!(PlaySfxEvent::new(SfxType::CardPlay).get_volume(), SfxType::CardPlay.default_volume());
    }

    #[test]
    fn test_sfx_settings_default() {
        let settings = SfxSettings::default();
        assert!(settings.enabled);
        assert!(settings.ui_enabled);
        assert!(settings.combat_enabled);
        assert_eq!(settings.master_volume, 0.7);
    }
}
