# 音效系统测试基线

> **更新日期**: 2026-01-31
> **测试文件**: `tests/sound_effects_tdd.rs`
> **资源指南**: `assets/audio/sfx/SOUND_EFFECTS_GUIDE.md`

---

## 测试执行基线

### 运行测试
```bash
# 运行所有音效测试
cargo test --test sound_effects_tdd

# 运行特定测试
cargo test --test sound_effects_tdd test_all_sfx_types_chinese_names
```

---

## 预期结果

### 单元测试 (10个)

| 测试名称 | 预期结果 | 状态 |
|---------|---------|------|
| `test_all_sfx_types_chinese_names` | ✓ 通过 | 所有38种音效中文名称正确 |
| `test_all_sfx_types_file_names` | ✓ 通过 | 所有音效文件名正确 |
| `test_all_sfx_types_categories` | ✓ 通过 | 所有音效分类正确 |
| `test_all_sfx_types_file_paths` | ✓ 通过 | 所有路径格式正确 (audio/sfx/{name}.ogg) |
| `test_default_volumes` | ✓ 通过 | 默认音量符合预期 |
| `test_recommended_durations` | ✓ 通过 | 推荐时长范围正确 |
| `test_play_sfx_event_creation` | ✓ 通过 | PlaySfxEvent创建正确 |
| `test_sfx_settings_default` | ✓ 通过 | SfxSettings默认值正确 |
| `test_sfx_types_count` | ✓ 通过 | 音效类型总数为38 |
| `test_file_naming_consistency` | ✓ 通过 | 文件命名一致 |

---

## 音效类型完整列表 (38种)

### 卡牌相关 (5种)
| 类型 | 中文名称 | 文件名 | 状态 |
|------|---------|--------|------|
| `CardPlay` | 出牌 | `card_play.ogg` | ✓ 已转换 |
| `DrawCard` | 抽牌 | `draw_card.ogg` | ✓ 已转换 |
| `ShuffleCard` | 洗牌 | `shuffle_card.ogg` | ✓ 已转换 |
| `CardHover` | 卡牌悬停 | `card_hover.ogg` | ✓ 已转换 |
| `CardSelect` | 卡牌选中 | `card_select.ogg` | ⏳ 待获取 |

### 战斗相关 (6种)
| 类型 | 中文名称 | 文件名 | 状态 |
|------|---------|--------|------|
| `PlayerAttack` | 玩家攻击 | `player_attack.ogg` | ✓ 已转换 |
| `PlayerHit` | 玩家受击 | `player_hit.ogg` | ✓ 已转换 |
| `EnemyHit` | 敌人受击 | `enemy_hit.ogg` | ✓ 已转换 |
| `Block` | 格挡 | `block.ogg` | ✓ 已转换 |
| `CriticalHit` | 暴击 | `critical_hit.ogg` | ✓ 已转换 |
| `Dodge` | 闪避 | `dodge.ogg` | ⏳ 待获取 |

### 法术技能 (7种)
| 类型 | 中文名称 | 文件名 | 状态 |
|------|---------|--------|------|
| `LightningStrike` | 天雷落下 | `lightning_strike.ogg` | ✓ 已转换 |
| `FireSpell` | 火焰法术 | `fire_spell.ogg` | ✓ 已转换 |
| `IceSpell` | 冰霜法术 | `ice_spell.ogg` | ✓ 已转换 |
| `Heal` | 治疗 | `heal.ogg` | ✓ 已转换 |
| `BuffApply` | 增益施加 | `buff_apply.ogg` | ✓ 已转换 |
| `DebuffApply` | 减益施加 | `debuff_apply.ogg` | ✓ 已转换 |
| `ShieldUp` | 护盾升起 | `shield_up.ogg` | ✓ 已转换 |

### 大招技能 (4种)
| 类型 | 中文名称 | 文件名 | 状态 |
|------|---------|--------|------|
| `UltimateStart` | 大招起手 | `ultimate_start.ogg` | ✓ 已转换 |
| `UltimateRelease` | 大招释放 | `ultimate_release.ogg` | ✓ 已转换 |
| `SwordStrike` | 剑气斩击 | `sword_strike.ogg` | ✓ 已转换 |
| `ThousandSwords` | 万剑归宗 | `thousand_swords.ogg` | ✓ 已转换 |

### UI交互 (5种)
| 类型 | 中文名称 | 文件名 | 状态 |
|------|---------|--------|------|
| `UiClick` | UI点击 | `ui_click.ogg` | ✓ 已转换 |
| `UiHover` | UI悬停 | `ui_hover.ogg` | ✓ 已转换 |
| `UiConfirm` | UI确认 | `ui_confirm.ogg` | ✓ 已转换 |
| `UiCancel` | UI取消 | `ui_cancel.ogg` | ✓ 已转换 |
| `UiError` | UI错误 | `ui_error.ogg` | ✓ 已转换 |

### 系统事件 (7种)
| 类型 | 中文名称 | 文件名 | 状态 |
|------|---------|--------|------|
| `BreakthroughStart` | 突破开始 | `breakthrough_start.ogg` | ✓ 已转换 |
| `BreakthroughSuccess` | 突破成功 | `breakthrough_success.ogg` | ✓ 已转换 |
| `LevelUp` | 升级 | `level_up.ogg` | ✓ 已转换 |
| `GoldGain` | 获得金币 | `gold_gain.ogg` | ✓ 已转换 |
| `RelicObtain` | 获得遗物 | `relic_obtain.ogg` | ✓ 已转换 |
| `Victory` | 战斗胜利 | `victory.ogg` | ✓ 已转换 |
| `Defeat` | 战斗失败 | `defeat.ogg` | ✓ 已转换 |

### 敌人相关 (4种)
| 类型 | 中文名称 | 文件名 | 状态 |
|------|---------|--------|------|
| `EnemySpawn` | 敌人生成 | `enemy_spawn.ogg` | ✓ 已转换 |
| `EnemyDeath` | 敌人死亡 | `enemy_death.ogg` | ✓ 已转换 |
| `BossAppear` | Boss登场 | `boss_appear.ogg` | ✓ 已转换 |
| `BossDeath` | Boss死亡 | `boss_death.ogg` | ✓ 已转换 |

> **进度**: 36/38 文件已转换 (全部使用 Vorbis 编码，立体声)
> **缺失**: card_select.ogg, dodge.ogg

---

## 音效文件获取流程

### 第1步：查看资源指南
```bash
cat assets/audio/sfx/SOUND_EFFECTS_GUIDE.md
```

### 第2步：访问推荐网站
- **Freesound**: https://freesound.org (需注册)
- **Zapsplat**: https://www.zapsplat.com (免费，需署名)
- **Mixkit**: https://mixkit.co/free-sound-effects (免费，无需署名)
- **爱给网**: https://www.aigei.com (中文资源)

### 第3步：搜索关键词
- 卡牌: `card play`, `shuffle cards`, `draw card`
- 战斗: `sword attack`, `hit impact`, `block sound`
- 法术: `lightning`, `fire spell`, `magic cast`
- 大招: `ultimate attack`, `powerful strike`

### 第4步：下载和编辑
1. 下载音效文件（优先选择 OGG/WAV 格式）
2. 使用 Audacity 等工具编辑：
   - 裁剪到推荐时长
   - 转换为 OGG Vorbis 格式
   - 标准化音量 (-3dB 到 -1dB)
   - 添加淡入淡出 (0.01-0.05秒)

### 第5步：放置文件
将处理好的文件放置到 `assets/audio/sfx/` 目录

### 第6步：移除占位符
```bash
./scripts/replace_sfx_placeholders.sh
```

### 第7步：验证
```bash
# 检查文件状态
./scripts/check_sfx_files.sh

# 运行测试
cargo test --test sound_effects_tdd

# 运行游戏
cargo run
```

---

## 音频格式要求

| 属性 | 要求 | 说明 |
|------|------|------|
| 格式 | OGG Vorbis | Bevy使用rodio库，支持Vorbis |
| 采样率 | 44.1kHz 或 48kHz | 标准音频采样率 |
| 比特率 | 192-320 kbps | 平衡质量和文件大小 |
| 声道 | 立体声或单声道 | 立体声推荐 |
| 音量 | -3dB 到 -1dB | 标准化后避免爆音 |

---

## 代码集成检查清单

### 插件注册
- [x] `SfxPlugin` 已添加到 `GamePlugin`
- [x] 事件系统已初始化
- [x] `SfxSettings` 资源已初始化

### 音效触发点（待实现）
- [ ] 卡牌系统触发 `CardPlay`, `DrawCard`, `CardHover`
- [ ] 战斗系统触发 `PlayerAttack`, `PlayerHit`, `EnemyHit`
- [ ] 法术系统触发 `LightningStrike`, `FireSpell` 等
- [ ] UI系统触发 `UiClick`, `UiHover`, `UiConfirm`

---

## 已知问题

| 问题 | 影响 | 解决方案 |
|------|------|---------|
| 音频文件缺失 | 20/38 音效文件待获取 | 从推荐网站下载 |
| 音效未触发 | 系统功能正常但无声音 | 在各系统中添加PlaySfxEvent触发 |

---

## 更新日志

| 日期 | 更新内容 |
|------|---------|
| 2026-01-30 | 批量转换所有音效到Vorbis格式（25/38），使用 -ac 2 立体声参数 |
| 2026-01-29 | 初始基线建立，38种音效类型定义完成 |
