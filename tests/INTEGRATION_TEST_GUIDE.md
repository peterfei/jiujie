# 集成测试规范

## 测试继承标准 (commit 2a818d5)

本项目所有集成测试必须继承 `tests/test_utils.rs` 中定义的测试框架。

---

## 模块化测试结构 (2025-01-25 更新)

### 测试分类

| 模块类型 | 命名模式 | 用途 | 示例 |
|---------|---------|------|------|
| 系统验证 | `*_validation.rs` | 验证功能正确性 | `shop_validation.rs` |
| Bug 还原 | `*_bug.rs` | 验证 bug 修复 | `combat_start_bug.rs` |
| 场景测试 | `*_scenario.rs` | 测试特定场景 | `enemy_ai_scenario.rs` |
| 流程测试 | `*_integration.rs` | 多系统协同 | `victory_flow_integration.rs` |
| 完整流程 | `full_flow_*.rs` | 端到端测试 | `full_flow_validation.rs` |

### 模块化优势

1. **按功能分类** - 易于查找和维护
2. **单一职责** - 每个模块专注一个领域
3. **并行开发** - 多人可同时开发不同模块
4. **独立运行** - 可以单独运行特定模块测试

---

## 快速开始

### 1. 创建新测试

```bash
# 1. 复制模板
cp tests/integration_tests/template.rs tests/integration_tests/your_test.rs

# 2. 编辑 mod.rs 添加模块
echo "pub mod your_test;" >> tests/integration_tests/mod.rs

# 3. 编写测试
vim tests/integration_tests/your_test.rs
```

### 2. 标准测试模板

```rust
use crate::test_utils::*;

#[test]
fn my_test() {
    let mut app = create_test_app();
    // ... 测试逻辑
}
```

---

## 可用辅助函数

### 环境创建

| 函数 | 说明 |
|------|------|
| `create_test_app()` | 创建完整的测试 Bevy App |
| `setup_combat_scene(app)` | 设置战斗场景（返回敌人实体） |
| `setup_shop_scene(app)` | 设置商店场景 |

### 时间控制

| 函数 | 说明 |
|------|------|
| `advance_frames(app, n)` | 运行 n 帧 |
| `advance_seconds(app, s)` | 运行约 s 秒（60fps） |

### 状态操作

| 函数 | 说明 |
|------|------|
| `transition_to_state(app, state)` | 切换游戏状态 |
| `get_current_state(app)` | 获取当前状态 |

### 实体操作

| 函数 | 说明 |
|------|------|
| `kill_enemy(app, entity)` | 杀死指定敌人 |

### 查询验证

| 函数 | 说明 |
|------|------|
| `count_particles(app)` | 统计粒子数量 |
| `count_emitters(app)` | 统计发射器数量 |
| `count_screen_effects(app)` | 统计屏幕特效数量 |
| `count_shop_ui(app)` | 统计商店UI数量 |
| `count_shop_card_buttons(app)` | 统计商店卡牌按钮数量 |
| `get_player_gold(app)` | 获取玩家金币 |
| `is_victory_delay_active(app)` | 检查胜利延迟是否激活 |
| `get_victory_delay_elapsed(app)` | 获取胜利延迟经过时间 |

---

## 测试场景

### 战斗场景测试

```rust
#[test]
fn test_combat_victory() {
    let mut app = create_test_app();
    let enemy = setup_combat_scene(&mut app);
    advance_frames(&mut app, 1);

    kill_enemy(&mut app, enemy);
    advance_frames(&mut app, 1);

    assert!(is_victory_delay_active(&app));
}
```

### 商店场景测试

```rust
#[test]
fn test_shop_initialization() {
    let mut app = create_test_app();
    setup_shop_scene(&mut app);
    advance_frames(&mut app, 1);

    assert!(count_shop_ui(&app) > 0);
    assert_eq!(get_player_gold(&app), 100);
}
```

### 状态转换测试

```rust
#[test]
fn test_state_transition() {
    let mut app = create_test_app();
    setup_combat_scene(&mut app);

    transition_to_state(&mut app, GameState::Shop);
    advance_frames(&mut app, 1);

    assert_eq!(get_current_state(&app), GameState::Shop);
}
```

---

## 命名规范

| 测试类型 | 文件名模式 | 示例 |
|---------|-----------|------|
| Bug 还原 | `{feature}_bug.rs` | `combat_start_bug.rs` |
| 场景测试 | `{feature}_scenario.rs` | `enemy_ai_scenario.rs` |
| 集成测试 | `{feature}_integration.rs` | `victory_flow_integration.rs` |
| 专项测试 | `{feature}_e2e.rs` | `shop_purchase_e2e.rs` |

---

## 运行测试

```bash
# 运行所有集成测试
cargo test --test integration

# 运行特定测试文件
cargo test --test integration test_name

# 运行特定测试函数
cargo test test_combat_victory

# 单线程模式（避免资源共享问题）
cargo test --test integration -- --test-threads=1

# 显示测试输出
cargo test --test integration -- --nocapture
```

---

## 扩展指南

### 添加新的辅助函数

如果需要新的测试辅助函数：

1. **优先考虑扩展现有函数**
   ```rust
   // ✅ 好：扩展现有函数
   pub fn setup_shop_scene(app: &mut App) {
       setup_shop_scene_with_gold(app, 100);
   }
   ```

2. **添加新函数到 test_utils.rs**
   ```rust
   /// 清晰的文档说明
   pub fn new_helper_function(app: &mut App) -> ReturnType {
       // 实现
   }
   ```

3. **更新本文档**

---

## 已知限制

1. **无头模式限制**
   - 粒子特效在无头模式下不稳定
   - 时间前进速度慢于实时
   - 解决方案：使用 `#[ignore]` 标记不稳定测试

2. **资源共享问题**
   - 并行测试可能导致状态冲突
   - 解决方案：使用 `--test-threads=1` 运行

3. **Assets 初始化**
   - 某些测试需要手动初始化 Asset 类型
   - 解决方案：使用 `create_test_app()` 已包含必要初始化

---

## 参考实现

- `victory_flow_integration.rs` - 完整的胜利流程测试
- `combat_start_bug.rs` - Bug 还原测试示例
- `shop_tdd.rs` - 商店系统 TDD 示例
- `template.rs` - 测试模板

---

## 版本历史

- **2026-02-07** - 建立 0.15 新基线，修复 100+ 编译错误，通过全量回归。
- **2025-01-25** - 确立 commit 2a818d5 为标准测试框架
- **2024-01-24** - 初始实现 (commit 2a818d5)
