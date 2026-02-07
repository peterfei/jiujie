# 测试基线 (2026-02-07)

## 概览
本次回归验证了从 Bevy 0.14 升级到 0.15 后的核心系统稳定性。通过修复 100+ 个测试文件的编译错误和运行 panic，确立了新的自动化测试标准。

## 核心指标
- **集成测试 (`cargo test --test integration`)**: 53/53 通过 (3个忽略，涉及无头模式下的粒子生成稳定性)。
- **核心 E2E 流程**: 全部通过 (涵盖地图、战斗、卡牌、商店、胜利延迟、存档、游戏结束)。
- **单元测试**: 全部通过 (涵盖音频、背景音乐、基础系统逻辑)。

## 关键修复与变更
### 1. 测试框架标准 (`tests/test_utils.rs`)
- **资产初始化**: 补全了 `AnimationGraph`, `Scene`, `AnimationClip`, `Mesh`, `StandardMaterial` 的 `init_asset`。
- **事件注册**: 统一注册了 `MouseMotion`, `MouseButtonInput`, `MouseWheel` 等交互事件，确保无头模式下 UI 系统不崩溃。
- **资源补全**: 默认插入了 `CharacterAssets`, `VictoryDelay`, `RelicCollection` 等核心资源。

### 2. 组件与结构体对齐
- **Combatant3d**: 全量修复了缺失的 `base_rotation` 和 `model_offset` 字段。
- **SpawnEffectEvent**: 
    - 移除了已弃用的 `burst` 字段（功能已被 `count` 取代）。
    - 修正了 `target_group` (现为 `Vec`) 和 `target_index` (现为 `usize`) 的初始化类型。
- **PlayerAnimationConfig**: 修正了 TDD 脚本中旧的 `hit_node` 和 `attack_node` 字段引用。

### 3. 系统逻辑确认
- **武器可见性**: 确认为 ImperialSword (万剑归宗) 期间武器应处于 `Inherited` 状态以配合特效，已修正相关 TDD 断言。
- **商店流程**: 修复了 `e2e_shop_full` 因缺少环境资产预加载导致的 panic。

## 当前已知限制
- **无头模式限制**: 
    - 部分粒子和屏幕闪烁特效在无头模式下可能无法在查询中立即捕获（已通过 `#[ignore]` 标记）。
    - `AnimationPlayer` 的权混合和状态切换在极个别 TDD 中存在单帧同步延迟。

## 运行回归建议
```bash
# 推荐的全量回归命令
cargo test -- --test-threads=1
```
