//! 集成测试模块
//!
//! 真实的 Bevy App 级别端到端测试

// 测试工具模块
mod test_utils;

// 胜利流程集成测试
mod victory_flow_integration;

// 调试测试
mod test_debug;

// 重新导出测试工具，供其他测试使用
pub use test_utils::*;
