# JiuJie: Tribulation

<p align="center">
  <img src="./assets/icons/icon_1024.png" width="256" height="256" alt="JiuJie Logo">
</p>

[![Bevy](https://img.shields.io/badge/Engine-Bevy_0.15-orange.svg)](https://bevyengine.org)
[![Rust](https://img.shields.io/badge/Language-Rust_1.80+-red.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT%20or%20Apache--2.0-blue.svg)](LICENSE)
[![Microsoft Store](https://img.shields.io/badge/Microsoft_Store-JiuJie-0078d4?logo=microsoft)](https://apps.microsoft.com/store/detail/9N2XV5GGRN98?cid=DevShareMCLPCS)

[中文版](./README.md)

> **"Ghostly movements, crackling thunders. Experience the ultimate hit-feel and visual aesthetics in this Xianxia Roguelike journey."**

---

## 🎮 Download & Play

Get the trial version directly from the Microsoft Store:

[**👉 Get JiuJie: Tribulation on Microsoft Store**](https://apps.microsoft.com/store/detail/9N2XV5GGRN98?cid=DevShareMCLPCS)

---

## 📖 Introduction

**JiuJie: Tribulation** is a high-performance Xianxia Roguelike card battler built with the **Bevy Engine (Rust)**.

More than just a game, it serves as a technical laboratory for 3D rendering in Rust. By combining minimalist sprite aesthetics with AAA-grade dynamic feedback and procedural VFX, it aims to deliver a visceral "Ascension" experience.

---

## 🎮 Visual Preview

<p align="center">
  <video src="https://github.com/user-attachments/assets/e7ad31af-04df-489d-903b-fdd1f234d12a" width="800" controls autoplay loop muted>
    Your browser does not support the video tag.
  </video>
</p>

> **[Movement Trails & After-images | WanJian Smart Targeting]**

---

## 🛠️ Core Technology Matrix

The project integrates advanced visual and performance solutions, verified by over 15 rounds of TDD (Test-Driven Development).

### ⚡ Top-tier Visual Effects (VFX V2)
- [x] **Full GPU Particle Acceleration**: Implemented via `bevy_hanabi`, supporting 10k+ concurrent particles with near-zero CPU overhead.
- [x] **Cinematic Procedural Lightning**: Recursive midpoint displacement algorithm featuring physical tapering and path steering for realistic lightning shapes.
- [x] **Ghost After-images**: Instant 3D pose capture (Snapshots) with 1.15x initial expansion and dynamic energy dissipation animations.
- [x] **GPU Ribbon Trails**: Smooth, glowing energy trails that activate dynamically based on character movement speed.
- [x] **HDR Additive Materials**: High-luminance cores that pierce through environmental fog, preventing visual "graying" and ensuring vibrant effects.

### 👊 Combat Juice & Time Systems
- [x] **AAA Hit-Stop System**: Virtual time scaling that triggers a 0.3s ultra-slowdown (0.01x) upon impact, simulating physical resistance.
- [x] **Synchronized Feedback**: Seamless integration of hit-stop, high-frequency screen flashes, particle overloads, and ghostly snapshots.
- [x] **WanJian: Smart Targeting**: Missile-style distribution algorithm that automatically spreads attacks across multiple targets and recalibrates mid-flight if a target dies.

### 🏗️ Software Architecture
- [x] **VFX Orchestrator Pattern**: Decouples logic from rendering, allowing complex 4-phase state machine management for advanced effects.
- [x] **Headless Integration Testing**: A robust suite of physical consistency tests capable of running in CI environments or headless macOS instances.

---

## 🚀 Quick Start

### Prerequisites
*   Rust 1.80+
*   GPU with WGPU support (Nvidia/AMD/Apple Silicon)

### Build & Run
```bash
# Clone the repository
git clone https://github.com/peterfei/JiuJie.git
cd JiuJie

# Run the game in release mode for best performance
cargo run --release
```

---

## 🤝 Contributing & License
Contributions via Issues or PRs are highly welcome.
Licensed under MIT or Apache-2.0.
