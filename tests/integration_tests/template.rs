//! 集成测试模板
//!
//! # 如何使用此模板
//!
//! 1. 复制此文件到 tests/integration_tests/ 目录
//! 2. 重命名为 your_test_name.rs
//! 3. 替换以下内容：
//!    - 文件名和模块描述
//!    - 测试函数名称
//!    - 测试逻辑
//! 4. 在 mod.rs 中添加：pub mod your_test_name;
//!
//! # 测试继承规范 (commit 2a818d5)
//!
//! 所有集成测试必须：
//! - 使用 `use crate::test_utils::*;`
//! - 使用 `create_test_app()` 创建测试环境
//! - 使用提供的辅助函数（如 `setup_combat_scene`, `setup_shop_scene`）
//!
//! # 测试命名规范
//!
//! - Bug 还原：`{feature}_bug.rs`
//! - 场景测试：`{feature}_scenario.rs`
//! - 集成测试：`{feature}_integration.rs`

use crate::test_utils::*;
use bevy_card_battler::states::GameState;

// ============================================================================
// 示例测试：战斗场景
// ============================================================================

#[test]
fn example_combat_test() {
    // 1. 创建测试环境
    let mut app = create_test_app();

    // 2. 设置测试场景
    let enemy_entity = setup_combat_scene(&mut app);

    // 3. 运行初始化帧
    advance_frames(&mut app, 1);

    // 4. 执行测试操作
    kill_enemy(&mut app, enemy_entity);

    // 5. 运行后续帧
    advance_frames(&mut app, 1);

    // 6. 验证结果
    assert!(is_victory_delay_active(&app), "胜利延迟应该激活");

    println!("✓ 测试通过");
}

// ============================================================================
// 示例测试：商店场景
// ============================================================================

#[test]
fn example_shop_test() {
    // 1. 创建测试环境
    let mut app = create_test_app();

    // 2. 设置商店场景
    setup_shop_scene(&mut app);

    // 3. 运行初始化帧
    advance_frames(&mut app, 1);

    // 4. 验证商店UI已创建
    let shop_ui_count = count_shop_ui(&mut app);
    assert!(shop_ui_count > 0, "商店UI应该被创建");

    // 5. 验证玩家金币
    let gold = get_player_gold(&mut app);
    assert_eq!(gold, 100, "初始金币应该是100");

    println!("✓ 测试通过");
}

// ============================================================================
// 示例测试：状态转换
// ============================================================================

#[test]
fn example_state_transition_test() {
    let mut app = create_test_app();

    // 设置初始状态
    setup_combat_scene(&mut app);
    advance_frames(&mut app, 1);

    // 验证初始状态
    assert_eq!(get_current_state(&app), GameState::Combat);

    // 切换状态
    transition_to_state(&mut app, GameState::Map);
    advance_frames(&mut app, 1);

    // 验证新状态
    assert_eq!(get_current_state(&app), GameState::Map);

    println!("✓ 测试通过");
}
