# 背景音乐系统测试基线

> **更新日期**: 2026-01-29
> **测试文件**: `tests/background_music_tdd.rs`

---

## 测试执行基线

### 运行测试
```bash
# 运行所有背景音乐测试
cargo test --test background_music_tdd

# 运行特定测试
cargo test --test background_music_tdd test_bgm_type_chinese_names
```

---

## 预期结果

### 单元测试

| 测试名称 | 预期结果 | 状态 |
|---------|---------|------|
| `test_bgm_type_chinese_names` | ✓ 通过 | 所有BGM中文名称正确 |
| `test_bgm_type_file_names` | ✓ 通过 | 所有BGM文件名正确 |
| `test_bgm_type_file_paths` | ✓ 通过 | 所有路径包含占位符（等待音频生成） |
| `test_default_volumes` | ✓ 通过 | 默认音量符合预期 |
| `test_current_bgm_is_playing` | ✓ 通过 | CurrentBgm状态判断正确 |
| `test_bgm_settings_default` | ✓ 通过 | BgmSettings默认值正确 |

### 集成测试

| 测试名称 | 预期结果 | 状态 |
|---------|---------|------|
| `test_play_bgm_event_creation` | ✓ 通过 | PlayBgmEvent创建正确 |
| `test_stop_bgm_event_creation` | ✓ 通过 | StopBgmEvent创建正确 |
| `test_crossfade_bgm_event_creation` | ✓ 通过 | CrossfadeBgmEvent创建正确 |
| `test_auto_select_bgm_by_scenario` | ✓ 通过 | 场景自动选择BGM正确 |

### 完整性检查

| 测试名称 | 预期结果 | 状态 |
|---------|---------|------|
| `test_all_bgm_types_defined` | ✓ 通过 | 8种BGM类型定义完整 |
| `test_bgm_file_naming_consistency` | ✓ 通过 | 文件命名一致（以_theme结尾） |
| `test_placeholder_detection` | ⚠ 警告 | 检测到占位符，音频文件尚未替换 |
| `test_volume_ranges` | ✓ 通过 | 所有音量在有效范围内 |

---

## BGM 类型完整列表

| 类型 | 中文名称 | 文件名 | 默认音量 | 状态 |
|------|---------|--------|---------|------|
| `MainMenu` | 修仙问道 | main_menu_theme.ogg | 0.70 | ⏳ 占位符 |
| `MapExploration` | 寻仙觅缘 | map_exploration_theme.ogg | 0.60 | ⏳ 占位符 |
| `NormalBattle` | 降妖除魔 | normal_battle_theme.ogg | 0.80 | ⏳ 占位符 |
| `BossBattle` | 生死对决 | boss_battle_theme.ogg | 0.90 | ⏳ 占位符 |
| `Tribulation` | 雷劫降临 | tribulation_theme.ogg | 0.85 | ⏳ 占位符 |
| `Shop` | 坊市繁华 | shop_theme.ogg | 0.65 | ⏳ 占位符 |
| `Rest` | 修炼打坐 | rest_theme.ogg | 0.50 | ⏳ 占位符 |
| `Victory` | 众妖伏诛 | victory_theme.ogg | 0.75 | ⏳ 占位符 |

---

## 音频文件替换流程

### 第1步：生成音乐
使用 `assets/music/SUNO_PROMPTS.md` 中的Prompt在Suno生成音乐

### 第2步：下载和编辑
1. 下载生成的音频（建议OGG格式）
2. 使用Audacity进行循环处理：
   - 删除不自然的开头/结尾
   - 添加0.5秒淡入/淡出
   - 确保首尾衔接自然

### 第3步：替换文件
将文件放置到 `assets/music/` 目录：
```text
assets/music/
├── main_menu_theme.ogg
├── map_exploration_theme.ogg
├── normal_battle_theme.ogg
├── boss_battle_theme.ogg
├── tribulation_theme.ogg
├── shop_theme.ogg
├── rest_theme.ogg
└── victory_theme.ogg
```

### 第4步：移除占位符
更新 `src/components/background_music.rs` 中的文件路径：

**替换前：**
```rust
pub fn file_path(&self) -> &'static str {
    match self {
        BgmType::MainMenu => "music/__PLACEHOLDER__main_menu_theme.ogg",
        // ...
    }
}
```

**替换后：**
```rust
pub fn file_path(&self) -> &'static str {
    match self {
        BgmType::MainMenu => "music/main_menu_theme.ogg",
        // ...
    }
}
```

### 第5步：验证
```bash
# 重新运行测试
cargo test --test background_music_tdd test_placeholder_detection

# 检查输出应显示：
# 【测试】✓ 占位符已移除，音频文件已替换
```

---

## 代码集成检查清单

### 插件注册
- [ ] `BackgroundMusicPlugin` 已添加到 `GamePlugin`
- [ ] 事件系统已初始化
- [ ] 资源已初始化（`CurrentBgm`, `BgmSettings`）

### 场景集成
- [ ] 主菜单状态触发 `PlayBgmEvent::new(BgmType::MainMenu)`
- [ ] 地图状态触发 `PlayBgmEvent::new(BgmType::MapExploration)`
- [ ] 商店状态触发 `PlayBgmEvent::new(BgmType::Shop)`
- [ ] 休息状态触发 `PlayBgmEvent::new(BgmType::Rest)`
- [ ] 战斗状态根据类型触发对应BGM
- [ ] 渡劫状态触发 `PlayBgmEvent::new(BgmType::Tribulation)`

### UI控制
- [ ] 设置界面添加音量滑块
- [ ] 设置界面添加音乐开关
- [ ] 音量变更时更新 `BgmSettings` 资源

---

## 已知问题

| 问题 | 影响 | 解决方案 |
|------|------|---------|
| 占位符未移除 | 音频无法加载 | 生成音频后移除占位符 |
| 音频文件缺失 | 播放失败时无提示 | 添加加载失败日志 |
| 无平滑切换 | 场景切换时音频突兀 | 实现交叉淡出 |

---

## 更新日志

| 日期 | 更新内容 |
|------|---------|
| 2026-01-29 | 初始基线建立 |
