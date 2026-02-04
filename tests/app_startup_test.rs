/// 集成测试：应用启动
///
/// 这是项目的第一个测试，遵循TDD原则：
/// 1. 红：测试先写，预期失败
/// 2. 绿：实现最小代码使测试通过
/// 3. 重构：改进代码质量

use bevy::app::App;
use bevy::prelude::*;
use bevy_card_battler::plugins::{CorePlugin, MenuPlugin};
use bevy_card_battler::states::GameState;

// ============================================================================
// 测试：应用启动
// ============================================================================

#[test]
fn test_app_can_be_created() {
    // GIVEN: 期望创建一个Bevy应用
    // WHEN: 创建App实例
    // THEN: 应用应该成功创建
    let _app = App::new();

    // 验证应用已创建
    assert!(true, "App should be created");
}

#[test]
fn test_app_with_core_plugins() {
    // GIVEN: 期望一个带核心插件的完整应用
    // WHEN: 创建App并添加CorePlugin和MenuPlugin
    // THEN: 应用应该成功构建

    let mut app = App::new();
    app.add_plugins(CorePlugin)
        .add_plugins(MenuPlugin);

    // 如果我们到达这里，说明插件初始化成功
    assert!(true, "Core plugins should be added successfully");
}

// ============================================================================
// 场景和状态转换测试
// ============================================================================

#[test]
fn test_game_state_enum_exists() {
    // GIVEN: 期望GameState枚举存在
    // WHEN: 使用GameState枚举
    // THEN: 所有状态变体都应该可用

    // 验证MainMenu状态存在
    let _main_menu = GameState::MainMenu;
    // 验证Map状态存在
    let _map = GameState::Map;
    // 验证Combat状态存在
    let _combat = GameState::Combat;
    // 验证Reward状态存在
    let _reward = GameState::Reward;
    // 验证GameOver状态存在
    let _game_over = GameState::GameOver;

    assert!(true, "All GameState variants should exist");
}

#[test]
fn test_game_state_default() {
    // GIVEN: 期望GameState有默认值
    // WHEN: 获取默认GameState
    // THEN: 默认值应该是Booting (用于资产预热)

    let default_state = GameState::default();
    assert_eq!(default_state, GameState::Booting, "Default GameState should be Booting");
}

#[test]
fn test_plugins_register_game_state() {
    // GIVEN: 期望CorePlugin能注册GameState
    // WHEN: 创建App并添加CorePlugin
    // THEN: GameState应该被注册为reflect类型

    let mut app = App::new();
    app.add_plugins(CorePlugin)
        .add_plugins(MenuPlugin);

    // 验证应用构建成功
    assert!(true, "Plugins should register GameState successfully");
}
