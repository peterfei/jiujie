//! 集成测试模块
//!
//! 真实的 Bevy App 级别端到端测试

// 测试工具模块
mod test_utils;

// 集成测试子模块
mod integration_tests;

// 重新导出测试工具，供其他测试使用
pub use test_utils::*;
