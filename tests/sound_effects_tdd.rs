//! 音效系统集成测试 (TDD)
//!
//! # 测试范围
//! - 音效类型定义
//! - 音效文件路径映射
//! - 音效事件创建
//! - 音效设置
//! - 占位符检测

use bevy_card_battler::components::audio::{
    SfxType, PlaySfxEvent, SfxSettings,
};

// ============================================================================
// 单元测试
// ============================================================================

#[test]
fn test_all_sfx_types_chinese_names() {
    // 卡牌相关
    assert_eq!(SfxType::CardPlay.chinese_name(), "出牌");
    assert_eq!(SfxType::DrawCard.chinese_name(), "抽牌");
    assert_eq!(SfxType::ShuffleCard.chinese_name(), "洗牌");
    assert_eq!(SfxType::CardHover.chinese_name(), "卡牌悬停");
    assert_eq!(SfxType::CardSelect.chinese_name(), "卡牌选中");

    // 战斗相关
    assert_eq!(SfxType::PlayerAttack.chinese_name(), "玩家攻击");
    assert_eq!(SfxType::PlayerHit.chinese_name(), "玩家受击");
    assert_eq!(SfxType::EnemyHit.chinese_name(), "敌人受击");
    assert_eq!(SfxType::Block.chinese_name(), "格挡");
    assert_eq!(SfxType::CriticalHit.chinese_name(), "暴击");
    assert_eq!(SfxType::Dodge.chinese_name(), "闪避");

    // 法术/技能
    assert_eq!(SfxType::LightningStrike.chinese_name(), "天雷落下");
    assert_eq!(SfxType::FireSpell.chinese_name(), "火焰法术");
    assert_eq!(SfxType::IceSpell.chinese_name(), "冰霜法术");
    assert_eq!(SfxType::Heal.chinese_name(), "治疗");
    assert_eq!(SfxType::BuffApply.chinese_name(), "增益施加");
    assert_eq!(SfxType::DebuffApply.chinese_name(), "减益施加");
    assert_eq!(SfxType::ShieldUp.chinese_name(), "护盾升起");

    // 大招/终极技能
    assert_eq!(SfxType::UltimateStart.chinese_name(), "大招起手");
    assert_eq!(SfxType::UltimateRelease.chinese_name(), "大招释放");
    assert_eq!(SfxType::SwordStrike.chinese_name(), "剑气斩击");
    assert_eq!(SfxType::ThousandSwords.chinese_name(), "万剑归宗");

    // UI交互
    assert_eq!(SfxType::UiClick.chinese_name(), "UI点击");
    assert_eq!(SfxType::UiHover.chinese_name(), "UI悬停");
    assert_eq!(SfxType::UiConfirm.chinese_name(), "UI确认");
    assert_eq!(SfxType::UiCancel.chinese_name(), "UI取消");
    assert_eq!(SfxType::UiError.chinese_name(), "UI错误");

    // 系统/事件
    assert_eq!(SfxType::BreakthroughStart.chinese_name(), "突破开始");
    assert_eq!(SfxType::BreakthroughSuccess.chinese_name(), "突破成功");
    assert_eq!(SfxType::LevelUp.chinese_name(), "升级");
    assert_eq!(SfxType::GoldGain.chinese_name(), "获得金币");
    assert_eq!(SfxType::RelicObtain.chinese_name(), "获得遗物");
    assert_eq!(SfxType::Victory.chinese_name(), "战斗胜利");
    assert_eq!(SfxType::Defeat.chinese_name(), "战斗失败");

    // 敌人相关
    assert_eq!(SfxType::EnemySpawn.chinese_name(), "敌人生成");
    assert_eq!(SfxType::EnemyDeath.chinese_name(), "敌人死亡");
    assert_eq!(SfxType::BossAppear.chinese_name(), "Boss登场");
    assert_eq!(SfxType::BossDeath.chinese_name(), "Boss死亡");
}

#[test]
fn test_all_sfx_types_file_names() {
    // 卡牌相关
    assert_eq!(SfxType::CardPlay.file_name(), "card_play");
    assert_eq!(SfxType::DrawCard.file_name(), "draw_card");
    assert_eq!(SfxType::ShuffleCard.file_name(), "shuffle_card");
    assert_eq!(SfxType::CardHover.file_name(), "card_hover");
    assert_eq!(SfxType::CardSelect.file_name(), "card_select");

    // 战斗相关
    assert_eq!(SfxType::PlayerAttack.file_name(), "player_attack");
    assert_eq!(SfxType::PlayerHit.file_name(), "player_hit");
    assert_eq!(SfxType::EnemyHit.file_name(), "enemy_hit");
    assert_eq!(SfxType::Block.file_name(), "block");
    assert_eq!(SfxType::CriticalHit.file_name(), "critical_hit");
    assert_eq!(SfxType::Dodge.file_name(), "dodge");

    // 法术/技能
    assert_eq!(SfxType::LightningStrike.file_name(), "lightning_strike");
    assert_eq!(SfxType::FireSpell.file_name(), "fire_spell");
    assert_eq!(SfxType::IceSpell.file_name(), "ice_spell");
    assert_eq!(SfxType::Heal.file_name(), "heal");
    assert_eq!(SfxType::BuffApply.file_name(), "buff_apply");
    assert_eq!(SfxType::DebuffApply.file_name(), "debuff_apply");
    assert_eq!(SfxType::ShieldUp.file_name(), "shield_up");

    // 大招/终极技能
    assert_eq!(SfxType::UltimateStart.file_name(), "ultimate_start");
    assert_eq!(SfxType::UltimateRelease.file_name(), "ultimate_release");
    assert_eq!(SfxType::SwordStrike.file_name(), "sword_strike");
    assert_eq!(SfxType::ThousandSwords.file_name(), "thousand_swords");

    // UI交互
    assert_eq!(SfxType::UiClick.file_name(), "ui_click");
    assert_eq!(SfxType::UiHover.file_name(), "ui_hover");
    assert_eq!(SfxType::UiConfirm.file_name(), "ui_confirm");
    assert_eq!(SfxType::UiCancel.file_name(), "ui_cancel");
    assert_eq!(SfxType::UiError.file_name(), "ui_error");

    // 系统/事件
    assert_eq!(SfxType::BreakthroughStart.file_name(), "breakthrough_start");
    assert_eq!(SfxType::BreakthroughSuccess.file_name(), "breakthrough_success");
    assert_eq!(SfxType::LevelUp.file_name(), "level_up");
    assert_eq!(SfxType::GoldGain.file_name(), "gold_gain");
    assert_eq!(SfxType::RelicObtain.file_name(), "relic_obtain");
    assert_eq!(SfxType::Victory.file_name(), "victory");
    assert_eq!(SfxType::Defeat.file_name(), "defeat");

    // 敌人相关
    assert_eq!(SfxType::EnemySpawn.file_name(), "enemy_spawn");
    assert_eq!(SfxType::EnemyDeath.file_name(), "enemy_death");
    assert_eq!(SfxType::BossAppear.file_name(), "boss_appear");
    assert_eq!(SfxType::BossDeath.file_name(), "boss_death");
}

#[test]
fn test_all_sfx_types_categories() {
    // 验证所有音效都有正确的分类
    assert_eq!(SfxType::CardPlay.category(), "卡牌相关");
    assert_eq!(SfxType::PlayerAttack.category(), "战斗相关");
    assert_eq!(SfxType::LightningStrike.category(), "法术技能");
    assert_eq!(SfxType::ThousandSwords.category(), "大招技能");
    assert_eq!(SfxType::UiClick.category(), "UI交互");
    assert_eq!(SfxType::Victory.category(), "系统事件");
    assert_eq!(SfxType::BossAppear.category(), "敌人相关");
}

#[test]
fn test_all_sfx_types_file_paths() {
    // 验证所有路径格式正确: audio/sfx/{file_name}.ogg
    assert_eq!(SfxType::CardPlay.file_path(), "audio/sfx/card_play.ogg");
    assert_eq!(SfxType::PlayerAttack.file_path(), "audio/sfx/player_attack.ogg");
    assert_eq!(SfxType::LightningStrike.file_path(), "audio/sfx/lightning_strike.ogg");
    assert_eq!(SfxType::ThousandSwords.file_path(), "audio/sfx/thousand_swords.ogg");
    assert_eq!(SfxType::UiClick.file_path(), "audio/sfx/ui_click.ogg");
    assert_eq!(SfxType::Victory.file_path(), "audio/sfx/victory.ogg");
    assert_eq!(SfxType::BossAppear.file_path(), "audio/sfx/boss_appear.ogg");

    // 验证所有路径都包含正确的目录前缀和扩展名
    let all_types = vec![
        SfxType::CardPlay,
        SfxType::DrawCard,
        SfxType::ShuffleCard,
        SfxType::PlayerAttack,
        SfxType::UiClick,
        SfxType::Victory,
        SfxType::BossAppear,
    ];

    for sfx_type in all_types {
        let path = sfx_type.file_path();
        // 验证路径以 audio/sfx/ 开头
        assert!(path.starts_with("audio/sfx/"), "路径 {} 应以 audio/sfx/ 开头", path);
        // 验证路径以 .ogg 结尾
        assert!(path.ends_with(".ogg"), "路径 {} 应以 .ogg 结尾", path);
        // 验证路径不包含占位符
        assert!(!path.contains("__PLACEHOLDER__"), "路径 {} 不应包含占位符", path);
    }
}

#[test]
fn test_default_volumes() {
    // UI音效较轻
    assert!(SfxType::UiHover.default_volume() < 0.5);
    assert!(SfxType::UiClick.default_volume() < 0.5);
    assert!(SfxType::CardHover.default_volume() < 0.5);

    // 大招音效较响
    assert!(SfxType::UltimateRelease.default_volume() > 0.8);
    assert!(SfxType::ThousandSwords.default_volume() > 0.8);
    assert!(SfxType::BossAppear.default_volume() > 0.8);
    assert!(SfxType::BossDeath.default_volume() > 0.8);

    // 暴击和重要事件
    assert!(SfxType::CriticalHit.default_volume() > 0.7);
    assert!(SfxType::Victory.default_volume() > 0.7);
    assert!(SfxType::BreakthroughSuccess.default_volume() > 0.7);

    // 所有音量在合理范围内
    assert!(SfxType::CardPlay.default_volume() > 0.0);
    assert!(SfxType::CardPlay.default_volume() <= 1.0);
}

#[test]
fn test_recommended_durations() {
    // UI音效很短
    let (min, max) = SfxType::UiHover.recommended_duration();
    assert!(min < 0.2);
    assert!(max < 0.3);

    let (min, max) = SfxType::UiClick.recommended_duration();
    assert!(min >= 0.1);
    assert!(max <= 0.3);

    // 大招音效较长
    let (min, max) = SfxType::ThousandSwords.recommended_duration();
    assert!(min >= 3.0);
    assert!(max <= 5.0);

    let (min, max) = SfxType::BossDeath.recommended_duration();
    assert!(min >= 3.0);
    assert!(max <= 5.0);
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
    assert_eq!(
        PlaySfxEvent::new(SfxType::CardPlay).get_volume(),
        SfxType::CardPlay.default_volume()
    );
}

#[test]
fn test_sfx_settings_default() {
    let settings = SfxSettings::default();
    assert!(settings.enabled);
    assert!(settings.ui_enabled);
    assert!(settings.combat_enabled);
    assert_eq!(settings.master_volume, 0.7);
}

#[test]
fn test_sfx_types_count() {
    // 验证音效类型总数
    let all_types = vec![
        // 卡牌相关 (5)
        SfxType::CardPlay,
        SfxType::DrawCard,
        SfxType::ShuffleCard,
        SfxType::CardHover,
        SfxType::CardSelect,
        // 战斗相关 (6)
        SfxType::PlayerAttack,
        SfxType::PlayerHit,
        SfxType::EnemyHit,
        SfxType::Block,
        SfxType::CriticalHit,
        SfxType::Dodge,
        // 法术/技能 (7)
        SfxType::LightningStrike,
        SfxType::FireSpell,
        SfxType::IceSpell,
        SfxType::Heal,
        SfxType::BuffApply,
        SfxType::DebuffApply,
        SfxType::ShieldUp,
        // 大招/终极技能 (4)
        SfxType::UltimateStart,
        SfxType::UltimateRelease,
        SfxType::SwordStrike,
        SfxType::ThousandSwords,
        // UI交互 (5)
        SfxType::UiClick,
        SfxType::UiHover,
        SfxType::UiConfirm,
        SfxType::UiCancel,
        SfxType::UiError,
        // 系统/事件 (7)
        SfxType::BreakthroughStart,
        SfxType::BreakthroughSuccess,
        SfxType::LevelUp,
        SfxType::GoldGain,
        SfxType::RelicObtain,
        SfxType::Victory,
        SfxType::Defeat,
        // 敌人相关 (4)
        SfxType::EnemySpawn,
        SfxType::EnemyDeath,
        SfxType::BossAppear,
        SfxType::BossDeath,
    ];

    assert_eq!(all_types.len(), 38);
}

#[test]
fn test_file_naming_consistency() {
    // 验证所有文件名都是小写且使用下划线
    let all_types = vec![
        SfxType::CardPlay,
        SfxType::PlayerAttack,
        SfxType::LightningStrike,
        SfxType::ThousandSwords,
        SfxType::UiClick,
    ];

    for sfx_type in all_types {
        let file_name = sfx_type.file_name();
        // 验证是小写
        assert_eq!(file_name, file_name.to_lowercase());
        // 验证只包含字母和下划线
        assert!(file_name.chars().all(|c| c.is_ascii_lowercase() || c == '_'));
    }
}
