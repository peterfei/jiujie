//! 集成测试子模块
//!
//! # 测试继承规范 (commit 2a818d5)
//!
//! 本项目的所有集成测试应该继承 `test_utils.rs` 中定义的测试框架。
//!
//! ## 使用方式
//!
//! ```rust
//! use crate::test_utils::*;
//!
//! #[test]
//! fn my_integration_test() {
//!     let mut app = create_test_app();
//!     let enemy_entity = setup_combat_scene(&mut app);
//!     // ... 测试逻辑
//! }
//! ```
//!
//! ## 为什么继承 test_utils？
//!
//! 1. **统一性**: 所有测试使用相同的 App 创建和场景设置
//! 2. **可维护性**: 测试工具升级时，所有测试自动受益
//! 3. **可复用性**: 经过验证的辅助函数可直接使用
//! 4. **稳定性**: 避免重复实现导致的 bug
//!
//! ## 模块化测试结构
//!
//! 测试按功能分类到不同模块：
//! - `*_validation.rs` - 系统验证测试
//! - `*_integration.rs` - 完整流程测试
//! - `*_bug.rs` - Bug 还原测试
//! - `*_scenario.rs` - 场景测试
//!
//! ## 扩展指南
//!
//! 如果需要新的测试辅助函数：
//! - 首先考虑能否扩展现有函数
//! - 如果确实需要新函数，添加到 `../test_utils.rs`
//! - 更新本文档说明新函数的用途
//!
//! ## 参考实现
//!
//! - `victory_flow_integration.rs` - 胜利流程完整示例
//! - `combat_start_bug.rs` - Bug 还原示例
//! - `enemy_ai_scenario.rs` - 复杂场景示例
//! - `shop_validation.rs` - 商店系统验证（新）
//! - `combat_validation.rs` - 战斗系统验证（新）
//! - `full_flow_validation.rs` - 端到端流程验证（新）

// ============================================================================
// 原有测试模块 (commit 2a818d5)
// ============================================================================

// 胜利流程集成测试
pub mod victory_flow_integration;

// 调试测试
pub mod test_debug;

// 敌人AI场景测试
pub mod enemy_ai_scenario;

// 战斗开始bug还原测试
pub mod combat_start_bug;

// Victory Delay状态泄漏bug测试
pub mod victory_delay_bug;

// 敌人Wait意图bug测试
pub mod enemy_wait_bug;

// 敌人回合时序bug测试
pub mod enemy_turn_order;

// ============================================================================
// 新增模块化测试 (2025-01-25)
// ============================================================================

// 商店系统验证测试
pub mod shop_validation;

// 战斗系统验证测试
pub mod combat_validation;

// 端到端流程验证测试
pub mod full_flow_validation;

// 测试模板（供新测试参考）
pub mod template;
