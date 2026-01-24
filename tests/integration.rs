//! 集成测试模块
//!
//! 真实的 Bevy App 级别端到端测试

// 测试工具模块
mod test_utils;

// 胜利流程集成测试
mod victory_flow_integration;

// 调试测试
mod test_debug;

// 敌人AI场景测试
mod enemy_ai_scenario;

// 战斗开始bug还原测试
mod combat_start_bug;

// Victory Delay状态泄漏bug测试
mod victory_delay_bug;

// 敌人Wait意图bug测试
mod enemy_wait_bug;

// 敌人回合时序bug测试
mod enemy_turn_order;

// 重新导出测试工具，供其他测试使用
pub use test_utils::*;
