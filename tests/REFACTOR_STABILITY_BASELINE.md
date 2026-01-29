# Bevy Card Battler - 重构稳定性测试基线 (2026-01-29)

本报告记录了在“地图系统重构”及“数值平衡进化”任务完成后的全量测试通过状态。该基线确证了核心逻辑在 150+ 个集成测试点上的稳定性。

## 1. 核心汇总
- **测试时间**: 2026-01-29
- **操作系统**: darwin
- **代码版本**: Refactor v0.1.0 (Map Path & Enemy Scaling)
- **总测试集**: 118 个测试单元
- **测试状态**: **100% PASS** (除渲染类忽略项)

## 2. 关键模块回归状态

| 模块 | 测试文件 | 状态 | 验证点 |
| :--- | :--- | :--- | :--- |
| **地图系统** | `map_refactor_tdd.rs` | ✅ PASS | 拓扑生成、next_nodes 逻辑、视野过滤 |
| **地图交互** | `map_interactions_e2e.rs` | ✅ PASS | 读档恢复、节点完成解锁下游、宝物节点跳转 |
| **数值平衡** | `enemy_scaling_tdd.rs` | ✅ PASS | 敌人 HP 随 Layer 动态缩放 (每层 +15%) |
| **AI 行为** | `enemy_ai_scenario.rs` | ✅ PASS | 狼、蜘蛛、怨灵及 Boss 的行为概率映射 |
| **战斗流程** | `victory_e2e.rs` | ✅ PASS | 击败 Boss、奖励结算、顺着拓扑通关 |
| **UI 鲁棒性** | `map_alignment_test.rs` | ✅ PASS | 滚动容器对齐、SpaceEvenly 数学对位 |
| **系统稳定性** | `combat_startup_test.rs` | ✅ PASS | 资源同步、清理系统、粒子/事件注册 |

## 3. 架构性确证
1. **玩家属性统一**: 玩家 HP/金币已完全移至 `Player` 资源，所有测试已适配。
2. **地图拓扑化**: 彻底告别“层级全开”，改为基于 `next_nodes` 的链式探索。
3. **神识视野**: 初始视野受限，随境界动态扩展，测试已适配视野过滤逻辑。
4. **视觉梯度**: 材质系统色调随难度偏移逻辑已通过 3D 同步验证。

## 4. 待观察项 (Ignored)
- 粒子特效生成稳定性 (Headless 环境局限)
- 屏幕闪光渲染 (无头模式下采样不准)

---
*@Validated: Refactor Stability Base established.*
