#!/bin/bash

# 音效文件批量转换为 Vorbis OGG 格式
# 使用 -ac 2 确保立体声输出（FFmpeg Vorbis 要求）

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  音效文件批量转换为 Vorbis OGG${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
echo ""

# 定义转换映射：源文件 -> 目标OGG文件
conversions=(
    # MP3 文件
    "big-thunder-clap.mp3|lightning_strike.ogg"
    "buff_apply.mp3|buff_apply.ogg"
    "card_play.mp3|card_play.ogg"
    "debuff_apply.mp3|debuff_apply.ogg"
    "draw_card.mp3|draw_card.ogg"
    "enemy_hit.mp3|enemy_hit.ogg"
    "fire_spell.mp3|fire_spell.ogg"
    "heal.mp3|heal.ogg"
    "holy-spell-cast-450460.mp3|ultimate_release.ogg"
    "ice_spell.mp3|ice_spell.ogg"
    "player_attack.mp3|player_attack.ogg"
    "player_hit.mp3|player_hit.ogg"
    "shield_up.mp3|shield_up.ogg"
    "shuffle_card.mp3|shuffle_card.ogg"
    "sword_strike.mp3|sword_strike.ogg"
    "thousand_swords.mp3|thousand_swords.ogg"
    "ui_click.mp3|ui_click.ogg"
    "ui_hover.mp3|ui_hover.ogg"
    "ultimate_release.mp3|ultimate_release.ogg"
    "ultimate_start.mp3|ultimate_start.ogg"

    # WAV 文件
    "block.wav|block.ogg"
    "card_hover.wav|card_hover.ogg"
    "critical_hit.wav|critical_hit.ogg"
    "lightning_strike.wav|lightning_strike.ogg"
    "player_hit.wav|player_hit.ogg"
    "ShuffleCard.wav|shuffle_card.ogg"

    # AIF 文件
    "dodge.aif|dodge.ogg"
)

success=0
failed=0
skipped=0

total=${#conversions[@]}
current=0

for entry in "${conversions[@]}"; do
    IFS='|' read -r src dst <<< "$entry"
    current=$((current + 1))

    echo -e "${CYAN}[$current/$total] 转换: ${src} → ${dst}${NC}"

    # 检查源文件
    if [ ! -f "$src" ]; then
        echo -e "  ${YELLOW}⊘ 源文件不存在，跳过${NC}"
        echo ""
        skipped=$((skipped + 1))
        continue
    fi

    # 执行转换
    if ffmpeg -i "$src" -ac 2 -vn -c:a vorbis -q:a 4 -strict -2 "$dst" -y 2>&1 | grep -q "muxing overhead"; then
        # 验证编码
        codec=$(ffprobe -v error -show_entries stream=codec_name -of default=noprint_wrappers=1:nokey=1 "$dst" 2>/dev/null | head -1)
        if [ "$codec" = "vorbis" ]; then
            size=$(ls -l "$dst" | awk '{print $5}')
            echo -e "  ${GREEN}✓ 转换成功 (Vorbis, ${size}B)${NC}"
            success=$((success + 1))
        else
            echo -e "  ${YELLOW}⚠ 编码验证失败: $codec${NC}"
            failed=$((failed + 1))
        fi
    else
        echo -e "  ${YELLOW}✗ 转换失败${NC}"
        failed=$((failed + 1))
    fi
    echo ""
done

echo -e "${CYAN}─────────────────────────────────────────────────${NC}"
echo -e "${CYAN}转换结果汇总:${NC}"
echo -e "  总计:   $total 个文件"
echo -e "  成功:   ${GREEN}${success}${NC} 个"
echo -e "  跳过:   ${YELLOW}${skipped}${NC} 个"
echo -e "  失败:   ${YELLOW}${failed}${NC} 个"
echo ""

if [ $failed -eq 0 ]; then
    echo -e "${GREEN}✅ 所有文件转换完成！${NC}"
    exit 0
else
    echo -e "${YELLOW}⚠ 部分文件转换失败${NC}"
    exit 1
fi
