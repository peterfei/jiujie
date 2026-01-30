# Suno AI 音乐生成 Prompts

> 使用说明：复制每个Prompt到Suno生成音乐，生成后下载并替换到对应的文件路径

---

## 1. 地图探索 - 「寻仙觅缘」

**文件名**: `map_exploration_theme.ogg`

**Prompt**:
```
[Instrumental only]
Epic Chinese fantasy orchestral exploration theme, ethereal bamboo flute (dizi) melody accompanied by guzheng harp, atmospheric synth pads creating mysterious fog layers, moderate tempo 85 BPM, D minor pentatonic scale, sense of journey and ascending to mountains, traditional Chinese instruments mixed with cinematic ambient sounds, soft wind chimes and water drop textures, hopeful and adventurous mood, 2 minutes loop, clean recording
```

**风格标签**:
`chinese fantasy, orchestral, ambient, exploration, guzheng, dizi, ethereal, mysterious, adventure`

---

## 2. 普通战斗 - 「降妖除魔」

**文件名**: `normal_battle_theme.ogg`

**Prompt**:
```
[Instrumental only]
Intense Chinese wuxia combat music, fast-paced erhu (Chinese violin) with aggressive bowing techniques, driving taiko drum patterns, 130 BPM, E minor, energetic battle rhythm with traditional percussion, pipa (Chinese lute) rapid tremolo passages, heroic and determined mood, dynamic build-ups with cymbal crashes, folk orchestra intensity, 2.5 minutes loop, punchy and powerful mix
```

**风格标签**:
`chinese wuxia, battle, erhu, taiko, pipa, combat, intense, fast-paced, heroic, folk`

---

## 3. Boss战 - 「生死对决」

**文件名**: `boss_battle_theme.ogg`

**Prompt**:
```
[Instrumental only]
Epic orchestral boss battle theme, grand Chinese percussion war drums, dramatic brass section with french horns and trombones, 145 BPM, C minor, powerful orchestral hits and impacts, traditional suona (Chinese horn) for dramatic effect, shredding guzheng passages, building tension with orchestral crescendo, apocalyptic and overwhelming sense of danger, cinematic production, 3 minutes loop, high intensity throughout
```

**风格标签**:
`epic orchestral, boss battle, chinese percussion, suona, guzheng, brass, dramatic, intense, cinematic`

---

## 4. 渡劫场景 - 「雷劫降临」

**文件名**: `tribulation_theme.ogg`

**Prompt**:
```
[Instrumental only]
Dark atmospheric thunder soundscape, ominous orchestral stings, deep rumbling bass frequencies, dramatic lightning strike impact sounds, haunting male choir chanting in low register, 90 BPM, F# minor, sense of celestial punishment and trial, traditional Chinese gong hits, suspenseful tension building, apocalyptic energy, thunder and rain texture layers, 2 minutes loop, cinematic and threatening
```

**风格标签**:
`dark, atmospheric, thunder, choir, ominous, tribulation, chinese gong, suspense, apocalyptic`

---

## 5. 仙家坊市 - 「坊市繁华」

**文件名**: `shop_theme.ogg`

**Prompt**:
```
[Instrumental only]
Peaceful Chinese folk merchant music, cheerful bamboo flute (dizi) major key melody, relaxed guzheng arpeggios, warm and welcoming atmosphere, 100 BPM, G major, traditional Chinese ensemble including yangqin (hammered dulcimer), soft woodblock percussion, bustling marketplace feeling without being chaotic, safe haven and rest area, happy and lighthearted mood, 2 minutes loop, clean and warm production
```

**风格标签**:
`chinese folk, peaceful, dizi, guzheng, merchant, cheerful, warm, traditional, happy`

---

## 6. 休息场景 - 「修炼打坐」

**文件名**: `rest_theme.ogg`

**Prompt**:
```
[Instrumental only]
Ambient meditation and cultivation music, sparse guqin (Chinese zither) with natural resonance and sustain, soft wind chimes and singing bowl textures, gentle synth pad foundation, 60 BPM, very slow and spacious, healing and recovery atmosphere, zen tranquility, minimal instrumentation with lots of silence, reverb and space for contemplation, 2 minutes loop, ASMR quality recording
```

**风格标签**:
`ambient, meditation, guqin, wind chimes, healing, zen, peaceful, minimal, relaxing`

---

## 7. 胜利曲目 - 「众妖伏诛」

**文件名**: `victory_theme.ogg`

**Prompt**:
```
[Instrumental only]
Triumphant Chinese victory fanfare, celebratory cymbals and large gong hits, upbeat folk melody on dizi and suona combination, heroic orchestral backing, 115 BPM, D major, sense of accomplishment and glory, traditional Chinese percussion ensemble including drum rolls, grand and uplifting mood, festival atmosphere, 1 minute loop, bright and triumphant mix
```

**风格标签**:
`triumphant, victory, chinese folk, celebratory, fanfare, dizi, suona, gong, heroic`

---

## 8. 游戏主菜单 - 「修仙问道」

**文件名**: `main_menu_theme.ogg`

**Prompt**:
```
[Instrumental only]
Grand Chinese fantasy main theme, emotional erhu solo with expressive vibrato opening, sweeping orchestral backdrop with strings and brass, philosophical and mystical atmosphere, 85 BPM, B minor to D major modulation, sense of ancient power and destiny, traditional instruments blending with cinematic orchestral production, guzheng cascading runs, emotional and epic feeling, 3 minutes, cinematic opening quality
```

**风格标签**:
`chinese fantasy, epic, erhu, orchestral, philosophical, mystical, cinematic, grand, emotional`

---

## 生成检查清单

使用Suno生成后，请按以下清单验收：

### 音频质量要求
- [ ] 格式：OGG (Vorbis编码) 或 MP3 320kbps
- [ ] 采样率：44.1kHz 或 48kHz
- [ ] 声道：立体声
- [ ] 循环：首尾衔接自然（可使用音频软件修剪）
- [ ] 时长：符合Prompt要求

### 放置路径
```
assets/music/
├── main_menu_theme.ogg          # 主菜单
├── map_exploration_theme.ogg    # 地图探索
├── normal_battle_theme.ogg      # 普通战斗
├── boss_battle_theme.ogg        # Boss战
├── tribulation_theme.ogg        # 渡劫
├── shop_theme.ogg               # 仙家坊市
├── rest_theme.ogg               # 休息场景
└── victory_theme.ogg            # 胜利
```

### 音频编辑建议
推荐使用 Audacity（免费）进行循环处理：
1. 打开生成的音频
2. 删除不自然的开头/结尾
3. 使用「淡入/淡出」0.5秒
4. 导出为OGG格式
