//! 集成测试覆盖验证（已迁移到模块化结构）
//!
//! ⚠️ **本文件已废弃，请使用模块化测试**
//!
//! 本测试已迁移到以下模块：
//! - `shop_validation.rs` - 商店相关测试
//! - `combat_validation.rs` - 战斗相关测试
//! - `full_flow_validation.rs` - 端到端流程测试
//!
//! ## 迁移说明
//!
### 旧方式（已废弃）:
//! ```bash
//! cargo test --test coverage_validation -- --test-threads=1
//! ```
//!
### 新方式（推荐）:
//! ```bash
//! # 运行所有验证测试
//! cargo test --test integration shop -- --test-threads=1
//! cargo test --test integration combat -- --test-threads=1
//! cargo test --test integration full_flow -- --test-threads=1
//!
//! # 或运行所有集成测试
//! cargo test --test integration -- --test-threads=1
//! ```
//!
//! ## 测试映射
//!
//! | 旧测试 | 新模块 | 新测试函数 |
//!|--------|--------|-----------|
//!| test_shop_ui_purchase_buttons_have_markers | shop_validation | test_shop_ui_purchase_buttons_have_markers |
//!| test_shop_update_gold_system_runs | shop_validation | test_shop_update_gold_system_runs |
//!| test_shop_ui_gold_display_with_deferred_commands | shop_validation | test_shop_initial_gold_display |
//!| test_multiple_on_enter_same_state_doesnt_duplicate | shop_validation | test_multiple_shop_entries_doesnt_duplicate |
//!| test_no_duplicate_players_after_state_transitions | combat_validation | test_no_duplicate_players_after_state_transitions |
//!| test_system_order_reset_before_interaction | combat_validation | test_system_order_reset_before_interaction |
//!| test_commands_spawn_is_deferred | combat_validation | test_commands_spawn_is_deferred |
//!| test_e2e_full_shop_and_combat_flow | full_flow_validation | test_e2e_full_shop_and_combat_flow |
//!
//! @deprecated 请使用 tests/integration_tests/ 下的模块化测试
