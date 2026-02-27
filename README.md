# 九界：渡劫 | JiuJie: Tribulation

[![Bevy](https://img.shields.io/badge/Engine-Bevy_0.15-orange.svg)](https://bevyengine.org)
[![Rust](https://img.shields.io/badge/Language-Rust_1.80+-red.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT%20or%20Apache--2.0-blue.svg)](LICENSE)

> **"身法如幻，惊雷裂空。在这场跨越九界的肉鸽渡劫之旅中，体验极致的打击感与视觉美学。"**
>
> **"Ghostly movements, crackling thunders. Experience the ultimate hit-feel and visual aesthetics in this Xianxia Roguelike journey across the nine realms."**

---

## 📖 简介 | Introduction

《九界：渡劫》是一款基于 **Bevy Engine (Rust)** 开发的修仙肉鸽卡牌游戏。它不仅仅是一个卡牌对战器，更是一个探索 3D 渲染极限的技术实验场。我们利用极简的纸片人美学，融合了 AAA 级的动态表现力，旨在创造出独一无二的“渡劫”体验。

**JiuJie: Tribulation** is a Xianxia Roguelike card battler powered by the **Bevy Engine (Rust)**. More than just a card game, it is a technical playground exploring the limits of 3D rendering. Blending minimalist sprite aesthetics with AAA-grade dynamic feedback, it delivers a truly unique "Ascension" experience.

---

## 🛠️ 技术矩阵 | Technical Showcase

目前项目已集成多项顶级视觉与性能方案，通过 15+ 轮 TDD（测试驱动开发）闭环验证：

Currently integrated top-tier visual and performance solutions, verified by 15+ rounds of TDD iterations:

### ⚡ 视觉与特效 | Visuals & VFX
- [x] **GPU 粒子全量加速 (Full GPU Particle Migration)**: 
  - 基于 `bevy_hanabi` 实现，支持万级粒子同屏，CPU 物理计算零开销。
  - Powered by `bevy_hanabi`, supporting 10k+ particles with zero CPU physics overhead.
- [x] **电影级分形闪电 (Cinematic Procedural Lightning)**: 
  - 采用递归中点位移算法，支持物理粗细渐变 (Tapering) 与路径纠偏。
  - Recursive midpoint displacement with physical tapering and path steering.
- [x] **身法残影系统 (Ghost After-images)**: 
  - 3D 姿态瞬间捕获 (Snapshot)，支持初始 1.15x 膨胀与动态能量爆散动画。
  - Instant 3D pose capture with 1.15x initial expansion and dynamic energy dissipation.
- [x] **GPU Ribbon 流光拖尾 (GPU Ribbon Trails)**: 
  - 随角色运动速度动态激活，呈现丝滑的能量轨迹。
  - Dynamically activated by movement speed, rendering smooth energy trails.
- [x] **HDR 加法混合材质 (HDR Additive Rendering)**: 
  - 蓝白过载内核，彻底击穿重雾环境，无惧视觉灰化。
  - High-luminance cores that pierce through fog, eliminating visual "graying."

### 👊 战斗打击感 | Combat & Juice
- [x] **AAA 级顿帧系统 (Virtual Time Hit-Stop)**: 
  - 命中瞬间 0.3s 极度减速 (0.01x)，模拟真实的物理撞击阻力。
  - 0.3s ultra-slowdown (0.01x) upon hit, simulating real physical impact resistance.
- [x] **视觉反馈联动 (Visual Feedback Sync)**: 
  - 顿帧、高频闪屏、粒子过载与残影闪现同步爆发。
  - Synchronized hit-stop, high-frequency flash, particle burst, and ghostly snapshot.
- [x] **万剑归宗：智能寻敌 (WanJian: Smart Targeting)**: 
  - 导弹式多目标自动分流，支持目标死亡后的瞬间航向修正。
  - Missile-style multi-target distribution with instant retargeting upon enemy death.

### 🏗️ 架构与底层 | Architecture
- [x] **VFX 编排器模式 (Vfx Orchestrator Pattern)**: 
  - 逻辑与渲染分离，支持复杂四阶段状态机编排。
  - Separation of logic and rendering, supporting complex 4-phase state machines.
- [x] **Headless 集成测试 (Headless Integration Testing)**: 
  - 完善的物理一致性验证集，支持在 CI 环境中运行。
  - Robust physical consistency verification suite, fully CI-compatible.

---

## 🎮 预览 | Preview

*(预留 GIF 展示位置 | Placeholder for Action GIFs)*
> **[万剑归宗 - 多目标打击 | WanJian Multi-target Strike]**
> **[身法移动 - 拖尾与残影 | Movement Trails & After-images]**

---

## 🚀 快速开始 | Quick Start

### 环境依赖 | Prerequisites
*   Rust 1.80+
*   支持 WGPU 的显卡 (Dedicated GPU with WGPU support)

### 构建与运行 | Build & Run
```bash
# 克隆仓库 Clone the repository
git clone https://github.com/peterfei/JiuJie.git
cd JiuJie

# 运行游戏 Run the game
cargo run --release
```

---

## 📜 路线图 | Roadmap
- [ ] 更多门派功法视觉重制 (More Xianxia sect VFX remasters)
- [ ] 实时环境交互粒子 (Real-time environmental interactive particles)
- [ ] 基于 Shader 的全屏后期处理特效 (Shader-based full-screen post-processing)

---

## 🤝 贡献与许可 | Contributing & License
欢迎提交 Issue 或 Pull Request 来共同打造最强 Bevy 特效库。
本项目采用 MIT 或 Apache-2.0 双协议许可。

Welcome to submit Issues or PRs. Together, we build the ultimate Bevy VFX showcase.
Licensed under MIT or Apache-2.0.
