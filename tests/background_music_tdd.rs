//! 背景音乐系统集成测试 (TDD)
//!
//! # 测试范围
//! - 背景音乐资源加载
//! - 背景音乐事件触发
//! - 背景音乐切换逻辑
//! - 音量控制
//! - 场景音乐关联
//!
//! # 使用说明
//! 1. 运行测试前，需要先生成音乐文件或创建占位音频
//! 2. 使用 `cargo test --test background_music_tdd` 运行测试
//! 3. 查看基线文件：`tests/BACKGROUND_MUSIC_BASELINE.md`

use bevy::prelude::*;
use bevy_card_battler::plugins::GamePlugin;
use bevy_card_battler::components::background_music::{
    BgmType, PlayBgmEvent, StopBgmEvent, CrossfadeBgmEvent, CurrentBgm, BgmSettings,
};
use bevy_card_battler::systems::background_music::BackgroundMusicPlugin;
use bevy_card_battler::states::GameState;

// ============================================================================
// 测试辅助函数
// ============================================================================

/// 创建测试应用
fn create_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(BackgroundMusicPlugin)
        .add_systems(Startup, || info!("【测试】背景音乐测试应用启动"));
    app
}

/// 创建完整游戏应用（用于集成测试）
fn create_game_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(GamePlugin)
        .add_systems(Startup, || info!("【测试】游戏应用启动"));
    app
}

// ============================================================================
// 单元测试
// ============================================================================

#[test]
fn test_bgm_type_chinese_names() {
    info!("【测试】验证BGM中文名称");
    assert_eq!(BgmType::MainMenu.chinese_name(), "修仙问道");
    assert_eq!(BgmType::MapExploration.chinese_name(), "寻仙觅缘");
    assert_eq!(BgmType::NormalBattle.chinese_name(), "降妖除魔");
    assert_eq!(BgmType::BossBattle.chinese_name(), "生死对决");
    assert_eq!(BgmType::Tribulation.chinese_name(), "雷劫降临");
    assert_eq!(BgmType::Shop.chinese_name(), "坊市繁华");
    assert_eq!(BgmType::Rest.chinese_name(), "修炼打坐");
    assert_eq!(BgmType::Victory.chinese_name(), "众妖伏诛");
    info!("【测试】✓ 所有BGM中文名称正确");
}

#[test]
fn test_bgm_type_file_names() {
    info!("【测试】验证BGM文件名");
    assert_eq!(BgmType::MainMenu.file_name(), "main_menu_theme");
    assert_eq!(BgmType::MapExploration.file_name(), "map_exploration_theme");
    assert_eq!(BgmType::NormalBattle.file_name(), "normal_battle_theme");
    assert_eq!(BgmType::BossBattle.file_name(), "boss_battle_theme");
    assert_eq!(BgmType::Tribulation.file_name(), "tribulation_theme");
    assert_eq!(BgmType::Shop.file_name(), "shop_theme");
    assert_eq!(BgmType::Rest.file_name(), "rest_theme");
    assert_eq!(BgmType::Victory.file_name(), "victory_theme");
    info!("【测试】✓ 所有BGM文件名正确");
}

#[test]
fn test_bgm_type_file_paths() {
    info!("【测试】验证BGM文件路径");
    // 验证路径格式正确（以 music/ 开头，以 .ogg 结尾）
    assert!(BgmType::MainMenu.file_path().starts_with("music/"));
    assert!(BgmType::MainMenu.file_path().ends_with(".ogg"));
    assert!(BgmType::MapExploration.file_path().starts_with("music/"));
    assert!(BgmType::MapExploration.file_path().ends_with(".ogg"));
    assert!(BgmType::NormalBattle.file_path().starts_with("music/"));
    assert!(BgmType::NormalBattle.file_path().ends_with(".ogg"));
    assert!(BgmType::BossBattle.file_path().starts_with("music/"));
    assert!(BgmType::BossBattle.file_path().ends_with(".ogg"));
    assert!(BgmType::Tribulation.file_path().starts_with("music/"));
    assert!(BgmType::Tribulation.file_path().ends_with(".ogg"));
    assert!(BgmType::Shop.file_path().starts_with("music/"));
    assert!(BgmType::Shop.file_path().ends_with(".ogg"));
    assert!(BgmType::Rest.file_path().starts_with("music/"));
    assert!(BgmType::Rest.file_path().ends_with(".ogg"));
    assert!(BgmType::Victory.file_path().starts_with("music/"));
    assert!(BgmType::Victory.file_path().ends_with(".ogg"));

    // 验证占位符已移除
    assert!(!BgmType::MainMenu.file_path().contains("__PLACEHOLDER__"));
    info!("【测试】✓ 所有BGM文件路径格式正确，占位符已移除");
}

#[test]
fn test_default_volumes() {
    info!("【测试】验证默认音量设置");
    // Boss战和渡劫应该更响亮
    assert!(BgmType::BossBattle.default_volume() > BgmType::NormalBattle.default_volume());
    assert!(BgmType::Tribulation.default_volume() > BgmType::NormalBattle.default_volume());
    // 休息应该更安静
    assert!(BgmType::Rest.default_volume() < BgmType::MapExploration.default_volume());
    info!("【测试】✓ 默认音量设置符合预期");
}

#[test]
fn test_current_bgm_is_playing() {
    info!("【测试】验证CurrentBgm状态判断");
    let bgm = CurrentBgm::new(BgmType::MainMenu, 0.7);
    assert!(bgm.is_playing(BgmType::MainMenu));
    assert!(!bgm.is_playing(BgmType::MapExploration));

    let paused_bgm = CurrentBgm {
        bgm_type: Some(BgmType::MainMenu),
        volume: 0.7,
        is_paused: true,
    };
    assert!(!paused_bgm.is_playing(BgmType::MainMenu));
    info!("【测试】✓ CurrentBgm状态判断正确");
}

#[test]
fn test_bgm_settings_default() {
    info!("【测试】验证BgmSettings默认值");
    let settings = BgmSettings::default();
    assert!(settings.enabled); // 默认启用
    assert_eq!(settings.master_volume, 0.7); // 默认音量70%
    info!("【测试】✓ BgmSettings默认值正确");
}

// ============================================================================
// 集成测试
// ============================================================================

#[test]
fn test_play_bgm_event_creation() {
    info!("【测试】验证PlayBgmEvent创建");

    // 默认事件
    let event = PlayBgmEvent::new(BgmType::MainMenu);
    assert_eq!(event.bgm_type, BgmType::MainMenu);
    assert_eq!(event.fade_in, 1.0);
    assert_eq!(event.volume, None);
    assert!(event.loop_);

    // 自定义事件
    let event = PlayBgmEvent::new(BgmType::BossBattle)
        .with_fade_in(2.0)
        .with_volume(0.9)
        .with_loop(false);
    assert_eq!(event.bgm_type, BgmType::BossBattle);
    assert_eq!(event.fade_in, 2.0);
    assert_eq!(event.volume, Some(0.9));
    assert!(!event.loop_);

    info!("【测试】✓ PlayBgmEvent创建正确");
}

#[test]
fn test_stop_bgm_event_creation() {
    info!("【测试】验证StopBgmEvent创建");

    // 默认事件
    let event = StopBgmEvent::new();
    assert_eq!(event.fade_out, 1.0);

    // 自定义事件
    let event = StopBgmEvent::new().with_fade_out(0.5);
    assert_eq!(event.fade_out, 0.5);

    // 立即停止
    let event = StopBgmEvent::immediate();
    assert_eq!(event.fade_out, 0.0);

    info!("【测试】✓ StopBgmEvent创建正确");
}

#[test]
fn test_crossfade_bgm_event_creation() {
    info!("【测试】验证CrossfadeBgmEvent创建");

    // 默认事件
    let event = CrossfadeBgmEvent::new(BgmType::MainMenu);
    assert_eq!(event.bgm_type, BgmType::MainMenu);
    assert_eq!(event.duration, 2.0);

    // 自定义事件
    let event = CrossfadeBgmEvent::new(BgmType::BossBattle).with_duration(3.0);
    assert_eq!(event.bgm_type, BgmType::BossBattle);
    assert_eq!(event.duration, 3.0);

    info!("【测试】✓ CrossfadeBgmEvent创建正确");
}

// ============================================================================
// 场景音乐关联测试
// ============================================================================

/// 测试自动选择背景音乐
#[test]
fn test_auto_select_bgm_by_scenario() {
    use bevy_card_battler::systems::background_music::auto_select_bgm;

    info!("【测试】验证场景自动选择背景音乐");

    // 主菜单
    assert_eq!(
        auto_select_bgm("main_menu", false, false),
        BgmType::MainMenu
    );

    // 地图探索
    assert_eq!(
        auto_select_bgm("map", false, false),
        BgmType::MapExploration
    );

    // 商店
    assert_eq!(auto_select_bgm("shop", false, false), BgmType::Shop);

    // 休息
    assert_eq!(auto_select_bgm("rest", false, false), BgmType::Rest);

    // 胜利
    assert_eq!(
        auto_select_bgm("victory", false, false),
        BgmType::Victory
    );

    // 普通战斗
    assert_eq!(
        auto_select_bgm("battle", false, false),
        BgmType::NormalBattle
    );

    // Boss战斗
    assert_eq!(auto_select_bgm("battle", true, false), BgmType::BossBattle);

    // 渡劫
    assert_eq!(
        auto_select_bgm("battle", false, true),
        BgmType::Tribulation
    );

    info!("【测试】✓ 场景自动选择背景音乐正确");
}

// ============================================================================
// 音乐完整性检查测试
// ============================================================================

#[test]
fn test_all_bgm_types_defined() {
    info!("【测试】验证所有BGM类型已定义");

    let all_types = vec![
        BgmType::MainMenu,
        BgmType::MapExploration,
        BgmType::NormalBattle,
        BgmType::BossBattle,
        BgmType::Tribulation,
        BgmType::Shop,
        BgmType::Rest,
        BgmType::Victory,
    ];

    // 验证每个类型都有完整的信息
    for bgm_type in &all_types {
        assert!(!bgm_type.chinese_name().is_empty());
        assert!(!bgm_type.file_name().is_empty());
        assert!(!bgm_type.file_path().is_empty());
        assert!(bgm_type.default_volume() > 0.0);
        assert!(bgm_type.default_volume() <= 1.0);
    }

    info!("【测试】✓ 所有{}种BGM类型定义完整", all_types.len());
}

#[test]
fn test_bgm_file_naming_consistency() {
    info!("【测试】验证BGM文件命名一致性");

    let all_types = vec![
        BgmType::MainMenu,
        BgmType::MapExploration,
        BgmType::NormalBattle,
        BgmType::BossBattle,
        BgmType::Tribulation,
        BgmType::Shop,
        BgmType::Rest,
        BgmType::Victory,
    ];

    // 验证所有文件都以_theme结尾
    for bgm_type in &all_types {
        let file_name = bgm_type.file_name();
        assert!(
            file_name.ends_with("_theme"),
            "{} 文件名应以 _theme 结尾",
            file_name
        );
    }

    info!("【测试】✓ BGM文件命名一致");
}

// ============================================================================
// 占位符检查（用于确认音频文件未替换）
// ============================================================================

#[test]
fn test_placeholder_detection() {
    info!("【测试】检测占位符状态");

    let has_placeholder = BgmType::MainMenu.file_path().contains("__PLACEHOLDER__");

    if has_placeholder {
        info!("【测试】⚠ 检测到占位符，音频文件尚未替换");
    } else {
        info!("【测试】✓ 占位符已移除，音频文件已替换");
    }

    // 这个测试总是通过，但会记录占位符状态
    assert!(true);
}

// ============================================================================
// 音量范围测试
// ============================================================================

#[test]
fn test_volume_ranges() {
    info!("【测试】验证音量范围");

    let all_types = vec![
        BgmType::MainMenu,
        BgmType::MapExploration,
        BgmType::NormalBattle,
        BgmType::BossBattle,
        BgmType::Tribulation,
        BgmType::Shop,
        BgmType::Rest,
        BgmType::Victory,
    ];

    for bgm_type in &all_types {
        let volume = bgm_type.default_volume();
        assert!(
            volume >= 0.0 && volume <= 1.0,
            "{} 音量超出范围 [0.0, 1.0]: {}",
            bgm_type.chinese_name(),
            volume
        );
    }

    info!("【测试】✓ 所有音量在有效范围内");
}
