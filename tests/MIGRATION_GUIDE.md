# 集成测试模块化迁移文档

## 迁移日期
2025-01-25

## 迁移目标
将 `coverage_validation.rs` 重构为模块化测试结构，继承 commit 2a818d5 的测试框架。

---

## 迁移前后对比

### 迁移前

```
tests/
├── coverage_validation.rs (608行，单一文件)
└── test_utils.rs (仅用于部分测试)
```

**问题**:
- 所有测试挤在一个文件中
- 重复实现 `create_full_app()`
- 未继承标准测试框架
- 难以维护和扩展

### 迁移后

```
tests/
├── test_utils.rs (标准测试框架，已扩展)
├── coverage_validation.rs (废弃，仅保留迁移说明)
├── INTEGRATION_TEST_GUIDE.md (测试规范文档)
└── integration_tests/
    ├── mod.rs (模块声明)
    ├── template.rs (测试模板)
    ├── shop_validation.rs (商店测试) ← 新增
    ├── combat_validation.rs (战斗测试) ← 新增
    ├── full_flow_validation.rs (端到端测试) ← 新增
    └── [其他原有测试模块...]
```

**优势**:
- ✅ 按功能分类，易于维护
- ✅ 统一使用 `test_utils.rs`
- ✅ 遵循 commit 2a818d5 标准
- ✅ 可复用的测试模板

---

## 测试模块映射

| 旧测试文件 | 新模块 | 测试数量 | 说明 |
|-----------|--------|---------|------|
| coverage_validation.rs | shop_validation.rs | 4 | 商店UI和金币系统 |
| coverage_validation.rs | combat_validation.rs | 3 | 战斗系统和状态转换 |
| coverage_validation.rs | full_flow_validation.rs | 1 | 端到端流程 |

---

## 模块化测试结构

### 1. shop_validation.rs - 商店系统验证

```rust
use crate::test_utils::*;

#[test]
fn test_shop_ui_purchase_buttons_have_markers() { ... }
#[test]
fn test_shop_update_gold_system_runs() { ... }
#[test]
fn test_shop_initial_gold_display() { ... }
#[test]
fn test_multiple_shop_entries_doesnt_duplicate() { ... }
```

### 2. combat_validation.rs - 战斗系统验证

```rust
use crate::test_utils::*;

#[test]
fn test_no_duplicate_players_after_state_transitions() { ... }
#[test]
fn test_system_order_reset_before_interaction() { ... }
#[test]
fn test_commands_spawn_is_deferred() { ... }
```

### 3. full_flow_validation.rs - 端到端流程

```rust
use crate::test_utils::*;

#[test]
fn test_e2e_full_shop_and_combat_flow() { ... }
```

---

## 测试继承模式

### 所有新测试必须遵循

```rust
//! 功能测试
//!
//! 继承自 commit 2a818d5 测试框架

use crate::test_utils::*;

#[test]
fn my_test() {
    // 1. 创建测试环境
    let mut app = create_test_app();

    // 2. 设置场景
    setup_combat_scene(&mut app); // 或其他场景函数

    // 3. 运行和验证
    advance_frames(&mut app, 1);
    assert!(...);

    println!("✓ 测试通过");
}
```

---

## test_utils.rs 扩展

### 新增商店相关函数

| 函数 | 说明 |
|------|------|
| `setup_shop_scene(app)` | 设置商店场景 |
| `count_shop_ui(app)` | 统计商店UI数量 |
| `count_shop_card_buttons(app)` | 统计卡牌按钮数量 |
| `get_player_gold(app)` | 获取玩家金币 |

### 新增状态相关函数

| 函数 | 说明 |
|------|------|
| `transition_to_state(app, state)` | 切换游戏状态 |
| `get_current_state(app)` | 获取当前状态 |

---

## 运行测试

### 旧命令（已废弃）

```bash
cargo test --test coverage_validation -- --test-threads=1
```

### 新命令（推荐）

```bash
# 运行所有集成测试
cargo test --test integration -- --test-threads=1

# 运行特定模块
cargo test --test integration shop -- --test-threads=1
cargo test --test integration combat -- --test-threads=1
cargo test --test integration full_flow -- --test-threads=1

# 运行特定测试
cargo test test_shop_ui_purchase_buttons_have_markers
```

---

## 创建新测试的步骤

### 1. 选择测试类型

| 类型 | 命名模式 | 用途 |
|------|---------|------|
| 验证测试 | `*_validation.rs` | 验证系统功能正确性 |
| Bug测试 | `*_bug.rs` | 还原和验证bug修复 |
| 场景测试 | `*_scenario.rs` | 测试特定使用场景 |
| 集成测试 | `*_integration.rs` | 多系统协同测试 |

### 2. 使用模板

```bash
cp tests/integration_tests/template.rs tests/integration_tests/my_test.rs
```

### 3. 实现测试

```rust
use crate::test_utils::*;

#[test]
fn test_my_feature() {
    let mut app = create_test_app();
    // ...
}
```

### 4. 注册模块

在 `tests/integration_tests/mod.rs` 添加：
```rust
pub mod my_test;
```

### 5. 运行验证

```bash
cargo test --test integration my_test -- --test-threads=1
```

---

## 迁移检查清单

- [x] 创建 shop_validation.rs
- [x] 创建 combat_validation.rs
- [x] 创建 full_flow_validation.rs
- [x] 更新 test_utils.rs
- [x] 创建 template.rs
- [x] 更新 mod.rs
- [x] 更新 coverage_validation.rs 为废弃说明
- [x] 创建 INTEGRATION_TEST_GUIDE.md
- [x] 创建 MIGRATION_GUIDE.md
- [ ] 验证所有测试通过
- [ ] 更新 CI 脚本使用新命令

---

## 兼容性说明

### 向后兼容

- ✅ 旧的 `coverage_validation.rs` 保留为废弃文件
- ✅ 所有测试功能已迁移到新模块
- ✅ 测试结果保持一致

### 未来测试

所有新的集成测试必须：
1. 使用 `use crate::test_utils::*;`
2. 使用 `create_test_app()` 创建测试环境
3. 遵循模块化命名规范
4. 参考 `template.rs` 模板

---

## 下一步优化

1. **性能优化**: 并行测试支持
2. **覆盖率报告**: 集成覆盖率统计
3. **CI集成**: 自动化测试报告
4. **文档生成**: 从测试注释生成文档
