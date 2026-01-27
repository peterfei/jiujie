# 「万剑归宗 · 诛仙剑阵」四相位终极视觉方案 - TDD 总结

## 项目概述

使用 TDD 模式重构「万剑归宗」技能，从三相位升级为四相位终极视觉方案。

**日期**: 2026-01-27
**状态**: ✅ 完成并修复 NaN Bug
**测试覆盖**: 16/16 通过

---

## 四相位设计

### 第一相位：万剑齐鸣 (The Call) - 0% ~ 20%
- **视觉**: 飞剑从虚空中"撕裂"而出，斜插向天际
- **动感**: 强烈后坐力（先沉一下再极速弹射）
- **函数**: `phase_one_the_call()`

### 第二相位：八卦剑轮 (Celestial Mandala) - 20% ~ 45%
- **视觉**: 立体多层圆锥形剑阵
- **动感**: 像鱼群"呼吸"颤动，剑身嗡鸣
- **函数**: `phase_two_celestial_mandala()`

### 第三相位：瞬狱锁定 (Ominous Pause) - 45% ~ 55%
- **视觉**: 全屏突然一静，飞剑调头指向敌人，背景变暗
- **动感**: 瞬间静止后的压迫感
- **函数**: `phase_three_ominous_pause()`

### 第四相位：极速穿心 (Mach Piercing) - 55% ~ 100%
- **视觉**: 极长残影流光，切向突刺
- **动感**: 每把剑击中时触发高亮火花
- **函数**: `phase_four_mach_piercing()`

---

## 代码修改点

### 文件修改: `src/systems/particle.rs`

1. **新增四个相位辅助函数** (行 280-410)
2. **修改主循环相位划分** (行 149-162)

### 测试文件: `tests/wanjian_four_phase_tdd.rs`

16 个测试用例覆盖：
- ✅ 时间区间验证
- ✅ 后坐力函数
- ✅ 三层圆锥结构
- ✅ 呼吸颤动效果
- ✅ 减速静止效果
- ✅ 三次贝塞尔曲线
- ✅ **NaN 防护**
- ✅ **边界值验证**

---

## 🐛 Bug 修复记录

### Bug #1: 负数 delay 导致 NaN

**错误信息**:
```
thread 'Compute Task Pool (3)' panicked at core/src/num/f32.rs:1405:9:
min > max, or either was NaN. min = 0.0, max = NaN
Encountered a panic in system `bevy_ui::layout::ui_layout_system`!
```

**根本原因**:
```rust
// 问题代码
let speed_factor = (1.0 - strike_t) * 5.0 + 1.0;  // strike_t=0 时 = 6.0
let trail_count = (speed_factor * 2.0) as usize;  // = 12

for i in 0..trail_count {
    let delay = 0.06 - (i as f32 * 0.015);  // i=11 时 = -0.105 ❌
}
```

**修复方案**:
```rust
// 修复代码
let delay = (0.06 - (i as f32 * 0.015)).max(0.0);  // 确保 delay 非负
```

---

### Bug #2: 位置 NaN 传播

**根本原因**:
- 第三相位减速阶段没有更新位置
- 第四相位继承无效位置进行贝塞尔计算
- `.normalize()` 对 NaN 向量返回 NaN

**修复方案**:

**第三相位** (`phase_three_ominous_pause`):
```rust
// 确保 position 有效
if !p.position.x.is_finite() || !p.position.y.is_finite() {
    p.position = Vec2::new(0.0, 250.0); // 默认位置
}
```

**第四相位** (`phase_four_mach_piercing`):
```rust
// 防护性检查
let inv_t = (1.0 - strike_t).max(0.0); // 确保 inv_t 非负

// 验证计算结果
if curve_pos.x.is_finite() && curve_pos.y.is_finite() {
    p.position = curve_pos;
}
```

---

### Bug #3: 实体删除后仍尝试更新 Transform

**错误信息**:
```
error[B0003]: src/systems/particle.rs:205:16: Could not insert a bundle (of type `bevy_transform::components::transform::Transform`)
for entity 5058v8#34359743426 because it doesn't exist in this World.
```

**根本原因**:
- 实体在 `is_dead()` 检查后通过 `despawn_recursive()` 删除
- 但后续的 `get_entity()` 仍返回 `Some`（命令延迟执行）
- 尝试向已删除的实体插入 Transform 导致 panic

**修复方案**:
```rust
// ❌ 错误：先删除实体，后更新 Transform
if p.is_dead() {
    commands.entity(entity).despawn_recursive();
    continue;
}
// ... 后续代码
if let Some(mut ec) = commands.get_entity(entity) {
    ec.insert(Transform::from_rotation(...));
}

// ✅ 正确：先更新 Transform，后删除实体
// 在死亡检查前更新 Transform
if commands.get_entity(entity).is_some() {
    if let Some(mut ec) = commands.get_entity(entity) {
        ec.insert(Transform::from_rotation(Quat::from_rotation_z(final_rotation)));
    }
}

// 死亡检查放最后
if p.is_dead() {
    commands.entity(entity).despawn_recursive();
    continue;
}
```

---

## 📊 TDD 测试覆盖对比

| 修复前 | 修复后 |
|--------|--------|
| 12 个测试 | 16 个测试 |
| 缺少 NaN 防护 | 新增 4 个边界测试 |
| 未覆盖极端值 | 覆盖所有边界情况 |

### 新增测试用例

1. `test_phase_four_trail_delay_never_negative` - delay 非负验证
2. `test_phase_four_extreme_speed_factor` - 极端速度因子
3. `test_phase_four_position_validation` - 位置有效性检查
4. `test_phase_three_position_preservation` - 位置保持验证

---

## 🎓 为什么 TDD 没有覆盖到 NaN Bug

| 原因 | 说明 |
|------|------|
| **1. 单元测试隔离** | TDD 测试只验证数学公式，没有实际运行 Bevy ECS 系统 |
| **2. 缺少集成测试** | 没有测试完整的事件发送 → 粒子创建 → UI 渲染流程 |
| **3. 边界条件遗漏** | 没有测试极端情况（strike_t=0 时速度最大） |
| **4. 状态依赖** | NaN 问题只在完整的游戏循环中才会暴露 |

### TDD 的局限性

| 能覆盖 | 不能覆盖 |
|--------|----------|
| ✅ 数学公式正确性 | ❌ ECS 系统交互 |
| ✅ 相位时间划分 | ❌ UI 布局计算 |
| ✅ 边界值（如果明确测试） | ❌ 运行时状态传播 |
| ✅ 函数输入输出 | ❌ 跨帧状态变化 |

---

## 🔧 防御性编程最佳实践

### 1. 数值范围限制

```rust
// ❌ 错误：没有限制范围
let trail_count = (speed_factor * 2.0) as usize;

// ✅ 正确：添加上限
let trail_count = ((speed_factor * 2.0) as usize).min(6);
```

### 2. NaN 防护

```rust
// ❌ 错误：直接使用计算结果
p.position = curve_pos;

// ✅ 正确：验证后使用
if curve_pos.x.is_finite() && curve_pos.y.is_finite() {
    p.position = curve_pos;
} else {
    p.position = Vec2::new(0.0, 250.0); // 默认值
}
```

### 3. 负数防护

```rust
// ❌ 错误：可能产生负数
let delay = 0.06 - (i as f32 * 0.015);

// ✅ 正确：限制最小值
let delay = (0.06 - (i as f32 * 0.015)).max(0.0);
```

### 4. 除零防护

```rust
// ❌ 错误：可能除以零
let value = x / y;

// ✅ 正确：使用 normalize_or
let dir = (target - pos).normalize_or(Vec2::ZERO);
// 或者
let result = if y.abs() > 1e-6 { x / y } else { 0.0 };
```

---

## 📁 关键文件清单

| 文件 | 修改内容 |
|------|----------|
| `src/systems/particle.rs` | 四相位逻辑 + NaN 防护 |
| `tests/wanjian_four_phase_tdd.rs` | 新建：16 个测试用例 |

---

## ✅ 验收标准

- [x] 所有 TDD 测试通过 (16/16)
- [x] NaN Bug 修复完成
- [x] 添加防御性检查
- [x] 相关测试无回归
- [x] 代码审查通过

---

## 🚀 下次避免类似问题的 Checklist

### 编码阶段

- [ ] 所有用户输入添加范围验证
- [ ] 浮点数运算后检查 `.is_finite()`
- [ ] 数组索引前检查边界
- [ ] 除法操作前检查除数

### 测试阶段

- [ ] 单元测试覆盖边界值（0、负数、最大值）
- [ ] 添加 NaN/Inf 测试用例
- [ ] 编写集成测试验证完整流程
- [ ] 使用日志追踪数值变化

### 代码审查

- [ ] 检查所有 `.unwrap()` 和 `.expect()`
- [ ] 验证向量运算的返回值
- [ ] 确认所有 `as` 转换的安全性
- [ ] 检查循环边界条件

---

## 📝 技术债务

- [ ] 考虑添加 E2E 测试覆盖完整战斗流程
- [ ] 添加性能监控（粒子数量限制）
- [ ] 考虑使用 `Option<T>` 代替可能无效的值

---

**生成时间**: 2026-01-27
**版本**: v1.0
**状态**: ✅ 完成并验证
