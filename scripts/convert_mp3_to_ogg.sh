#!/bin/bash
#
# MP3 转 OGG 转换脚本
#
# 使用说明：
# 1. 确保已安装 ffmpeg
# 2. 运行此脚本转换 assets/music/ 下的 MP3 文件
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

MUSIC_DIR="assets/music"

# 需要转换的文件映射（源文件 -> 目标文件）
declare -a FILES_TO_CONVERT=(
    "boss_battle_theme.mp3|boss_battle_theme.ogg|生死对决 - Boss战"
    "main_menu_theme.mp3|main_menu_theme.ogg|修仙问道 - 主菜单"
    "map_exploration_theme.mp3|map_exploration_theme.ogg|寻仙觅缘 - 地图探索"
    "normal_battle_theme.mp3|normal_battle_theme.ogg|降妖除魔 - 普通战斗"
    "rest_theme.mp3|rest_theme.ogg|修炼打坐 - 休息场景"
    "shop_theme.mp3|shop_theme.ogg|坊市繁华 - 仙家坊市"
    "tribulation_theme.mp3|tribulation_theme.ogg|雷劫降临 - 渡劫场景"
    "victory_theme.mp3|victory_theme.ogg|众妖伏诛 - 胜利曲目"
)

echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
echo -e "${CYAN}${BOLD}  MP3 转 OGG 音频转换${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
echo ""

# 检查 ffmpeg
if ! command -v ffmpeg &> /dev/null; then
    echo -e "${RED}❌ 错误: ffmpeg 未安装${NC}"
    echo -e "${YELLOW}请安装: brew install ffmpeg${NC}"
    exit 1
fi

echo -e "${GREEN}✓ ffmpeg 已安装${NC}"
echo -e "${BLUE}📂 音乐目录: $MUSIC_DIR${NC}"
echo ""

# 转换统计
TOTAL=0
SUCCESS=0
SKIPPED=0
FAILED=0

for entry in "${FILES_TO_CONVERT[@]}"; do
    IFS='|' read -r src dst desc <<< "$entry"

    TOTAL=$((TOTAL + 1))
    src_path="$MUSIC_DIR/$src"
    dst_path="$MUSIC_DIR/$dst"

    echo -e "${BOLD}[$TOTAL] $desc${NC}"
    echo -e "  源文件: ${CYAN}$src${NC}"
    echo -e "  目标:   ${CYAN}$dst${NC}"

    # 检查源文件
    if [ ! -f "$src_path" ]; then
        echo -e "  ${RED}✗ 源文件不存在，跳过${NC}"
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
        rm -f "$dst_path"
    fi

    # 执行转换 (使用 libopus 编码器)
    echo -e "  ${BLUE}⏳ 转换中...${NC}"
    if ffmpeg -i "$src_path" -vn -c:a libopus -b:a 128k "$dst_path" -y 2>&1 | grep -q "muxing overhead"; then
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
            echo -e "     压缩率: ${YELLOW}${ratio}%${NC} (OGG ${dst_size}B / MP3 ${src_size}B)"
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

# 询问是否删除源文件
if [ $SUCCESS -gt 0 ]; then
    read -p "是否删除已转换的 MP3 源文件？(y/N) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${YELLOW}🗑 删除 MP3 文件...${NC}"
        for entry in "${FILES_TO_CONVERT[@]}"; do
            IFS='|' read -r src dst desc <<< "$entry"
            src_path="$MUSIC_DIR/$src"
            dst_path="$MUSIC_DIR/$dst"

            if [ -f "$dst_path" ]; then
                rm -f "$src_path"
                echo -e "  ${GREEN}✓ 已删除: $src${NC}"
            fi
        done
        echo ""
    fi
fi

# 下一步
echo -e "${BOLD}─────────────────────────────────────────────────${NC}"
if [ $FAILED -eq 0 ] && [ $SUCCESS -gt 0 ]; then
    echo -e "${GREEN}${BOLD}✅ 转换完成！${NC}"
    echo ""
    echo -e "下一步:"
    echo -e "  1. 检查文件状态: ${CYAN}./scripts/check_music_files.sh${NC}"
    echo -e "  2. 移除代码占位符: ${CYAN}./scripts/replace_music_placeholders.sh${NC}"
    echo -e "  3. 运行测试: ${CYAN}cargo test --test background_music_tdd${NC}"
else
    echo -e "${YELLOW}⚠ 请检查失败的项目${NC}"
fi
echo ""

exit $FAILED
