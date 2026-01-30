# 万剑归宗与水墨视觉/音频系统基线 (2026-01-30)

## 1. 核心视觉 (VFX) 基线
### 水墨云雾 (CloudMist)
*   **风格**: 三国志 11 写意风格。
*   **纹理**: 128x128 程序化生成羽化贴图，边缘扰动 sin(4.0) + cos(7.0)。
*   **参数**: 
    *   尺寸: 800.0 - 1500.0 像素。
    *   生命周期: 12.0 - 18.0s。
    *   透明度: 0.28 (黄金平衡)。
    *   动态: 自底向上升腾 (Gravity: 15.0)，带随机旋转 (0.25 扰动)。
*   **循环**: 永久循环模式 (max_particles: 99999)。

## 2. 核心音频 (SFX) 基线
### 稳定性规范
*   **格式**: 统一使用 `.ogg`。
*   **安全检查**: 引入 `std::path::Path::exists()` 门禁，缺失文件自动跳过，杜绝加载闪退。
*   **触发覆盖**:
    *   主菜单/地图按钮点击: `UiClick`
    *   战斗受击: `EnemyHit` / `PlayerHit`
    *   功法效果: `ShieldUp`, `Heal`, `ThousandSwords`
    *   结算: `Victory`, `GoldGain`, `LevelUp`

## 3. 游戏稳定性 (Stability) 基线
### 自动存档机制
*   **触发点**: 统一在 `OnEnter(GameState::Map)` 时执行。
*   **优化点**: 彻底移除了所有按钮回调中的同步磁盘 IO，解决了大规模 UI 销毁时的假死风险。

## 4. 测试环境 (Test Env) 规范
### 标准化 Logic-Complete 环境
*   **核心函数**: `test_utils::create_test_app()`
*   **插件组成**: 
    *   `MinimalPlugins` (无头运行)
    *   `HierarchyPlugin` (支持 UI 树操作)
    *   `StatesPlugin` (完整状态机支持)
    *   `AssetPlugin` (虚拟资源加载)
*   **优势**: 初始化时间 < 100ms，支持跨状态跳转验证，杜绝 "Schedule missing" 错误。

## 5. 测试状态
*   `audio_system_tdd.rs`: ✓ 通过 (验证文件存在性)
*   `cloud_mist_vfx_tdd.rs`: ✓ 通过 (验证水墨物理参数)
*   `rest_hang_repro_tdd.rs`: ✓ 通过 (验证洞府退出稳定性)
*   `map_difficulty_scaling_tdd.rs`: ✓ 通过 (验证层级缩放系数)
