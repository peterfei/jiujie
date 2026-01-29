# 地图重构与架构统一 - 测试基线报告

**日期**: 2026-01-29
**状态**: 全量通过 (PASS)
**基准版本**: Map Refactor v2 (Topology + Realm Vision)

## 1. 核心逻辑回归 (Core Regression)
以下模块已完成从旧架构（PlayerDeck 持有属性）到新架构（Player 资源持有属性）的迁移，并验证通过：

| 测试分类 | 文件路径 | 状态 | 回归说明 |
| :--- | :--- | :--- | :--- |
| **地图路径** | `tests/map_refactor_tdd.rs` | PASS | 验证路径连接、非法移动拦截 |
| **数据持久化** | `tests/flow_persistence_tdd.rs` | PASS | 验证 Player/Deck/Cultivation 跨场景同步 |
| **初始状态** | `tests/initial_deck_bug_tdd.rs` | PASS | 验证新修行开始时的属性分配 |
| **胜利结算** | `tests/victory_rewards_tdd.rs` | PASS | 验证战斗胜利后的金币/卡牌奖励同步 |
| **商店系统** | `tests/e2e_shop_full.rs` | PASS | 验证商店购买逻辑与金币扣除 |

## 2. 集成测试验证 (Integration Tests)
运行 `cargo test --test integration` 结果：
- **总计**: 56 个系统级测试
- **通过**: 53 个
- **忽略**: 3 个 (渲染相关)
- **关键修复**: 解决了 `MapPlugin` 缺失导致的 E2E 流程中断问题。

## 3. 架构回归标识 (Regression Tagging)
以下测试文件已手动标记 `@Validated: Refactor Regression`：
- `tests/integration_tests/full_flow_validation.rs`
- `tests/map_system_fixes.rs`
- `tests/hand_ui_timing_tdd.rs`
- `tests/deck_ui_tdd.rs`

---
**结论**: 项目已建立稳固的新基线，底层数据模型冲突已完全消除。
