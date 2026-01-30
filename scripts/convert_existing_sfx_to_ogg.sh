#!/bin/bash
#
# 现有音效文件批量转换为OGG脚本
#
# 使用说明：
# 将 assets/audio/sfx/ 目录下的现有音频文件转换为 OGG Vorbis 格式
#

set -e

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color
BOLD='\033[1m'

SFX_DIR="assets/audio/sfx"

echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
echo -e "${CYAN}${BOLD}  现有音效文件转换为OGG格式${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
echo ""

# 检查ffmpeg
if ! command -v ffmpeg &> /dev/null; then
    echo -e "${RED}❌ 错误: ffmpeg 未安装${NC}"
    echo -e "${YELLOW}请安装: brew install ffmpeg${NC}"
    exit 1
fi

echo -e "${GREEN}✓ ffmpeg 已安装${NC}"
echo -e "${BLUE}📂 音效目录: $SFX_DIR${NC}"
echo ""

# 文件映射：源文件 -> 目标OGG文件
declare -a CONVERSIONS=(
    # MP3文件
    "big-thunder-clap.mp3|lightning_strike.ogg"
    "buff_apply.mp3|buff_apply.ogg"
    "card_play.mp3|card_play.ogg"
    "critical_hit.mp3|critical_hit.ogg"
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

    # WAV文件
    "block.wav|block.ogg"
    "card_hover.wav|card_hover.ogg"
    "lightning_strike.wav|lightning_strike.ogg"
    "player_hit.wav|player_hit.ogg"
    "ShuffleCard.wav|shuffle_card.ogg"
    "critical_hit.wav|critical_hit.ogg"

    # AIF文件
    "dodge.aif|dodge.ogg"
)

# 转换统计
TOTAL=0
SUCCESS=0
SKIPPED=0
FAILED=0

for entry in "${CONVERSIONS[@]}"; do
    IFS='|' read -r src dst <<< "$entry"

    TOTAL=$((TOTAL + 1))
    src_path="$SFX_DIR/$src"
    dst_path="$SFX_DIR/$dst"

    echo -e "${BOLD}[$TOTAL] 转换: ${src} → ${dst}${NC}"

    # 检查源文件
    if [ ! -f "$src_path" ]; then
        echo -e "  ${YELLOW}⊘ 源文件不存在，跳过${NC}"
        echo ""
        SKIPPED=$((SKIPPED + 1))
        continue
    fi

    # 检查目标文件是否已存在
    if [ -f "$dst_path" ]; then
        echo -e "  ${YELLOW}⚠ 目标文件已存在${NC}"
        read -p "  是否覆盖？(y/N) " -n 1 -r
        echo ""
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo -e "  ${YELLOW}⊘ 跳过${NC}"
            echo ""
            SKIPPED=$((SKIPPED + 1))
            continue
        fi
    fi

    # 执行转换（使用Vorbis编码）
    echo -e "  ${BLUE}⏳ 转换中...${NC}"
    if ffmpeg -i "$src_path" -vn -c:a vorbis -q:a 4 -strict -2 "$dst_path" -y 2>&1 | grep -q "muxing overhead"; then
        # 获取文件大小
        if [[ "$OSTYPE" == "darwin"* ]]; then
            src_size=$(stat -f%z "$src_path" 2>/dev/null || echo "0")
            dst_size=$(stat -f%z "$dst_path" 2>/dev/null || echo "0")
        else
            src_size=$(stat -c%s "$src_path" 2>/dev/null || echo "0")
            dst_size=$(stat -c%s "$dst_path" 2>/dev/null || echo "0")
        fi

        # 计算压缩率
        if [ $src_size -gt 0 ]; then
            ratio=$((100 * dst_size / src_size))
            echo -e "  ${GREEN}✓ 转换成功${NC}"
            echo -e "     压缩率: ${YELLOW}${ratio}%${NC} (OGG ${dst_size}B / ${src_size}B)"
        else
            echo -e "  ${GREEN}✓ 转换成功${NC}"
        fi
        SUCCESS=$((SUCCESS + 1))
    else
        echo -e "  ${RED}✗ 转换失败${NC}"
        FAILED=$((FAILED + 1))
    fi
    echo ""
done

# 汇总
echo -e "${BOLD}─────────────────────────────────────────────────${NC}"
echo -e "${BOLD}转换结果汇总:${NC}"
echo -e "  总计:   ${BLUE}${TOTAL}${NC} 个文件"
echo -e "  成功:   ${GREEN}${SUCCESS}${NC} 个"
echo -e "  跳过:   ${YELLOW}${SKIPPED}${NC} 个"
echo -e "  失败:   ${RED}${FAILED}${NC} 个"
echo ""

# 验证OGG编码
echo -e "${BOLD}─────────────────────────────────────────────────${NC}"
echo -e "${BOLD}验证OGG编码...${NC}"
OGG_COUNT=0
for entry in "${CONVERSIONS[@]}"; do
    IFS='|' read -r src dst <<< "$entry"
    dst_path="$SFX_DIR/$dst"

    if [ -f "$dst_path" ]; then
        codec=$(ffprobe -v error -show_entries stream=codec_name -of default=noprint_wrappers=1:nokey=1 "$dst_path" 2>/dev/null | head -1)
        if [ "$codec" = "vorbis" ]; then
            echo -e "  ${GREEN}✓${NC} $dst"
            OGG_COUNT=$((OGG_COUNT + 1))
        else
            echo -e "  ${RED}✗${NC} $dst (编码: $codec)"
        fi
    fi
done
echo ""

# 下一步
echo -e "${BOLD}─────────────────────────────────────────────────${NC}"
if [ $FAILED -eq 0 ] && [ $SUCCESS -gt 0 ]; then
    echo -e "${GREEN}${BOLD}✅ 转换完成！${NC}"
    echo ""
    echo -e "下一步:"
    echo -e "  1. 检查文件状态: ${CYAN}./scripts/check_sfx_files.sh${NC}"
    echo -e "  2. 移除代码占位符: ${CYAN}./scripts/replace_sfx_placeholders.sh${NC}"
    echo -e "  3. 运行测试: ${CYAN}cargo test --test sound_effects_tdd${NC}"
    echo -e "  4. 运行游戏: ${CYAN}cargo run${NC}"
else
    echo -e "${YELLOW}⚠ 请检查失败的项目${NC}"
fi
echo ""

exit $FAILED
