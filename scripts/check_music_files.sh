#!/bin/bash
#
# 背景音乐文件检查脚本
#
# 使用说明：
# 运行此脚本检查音乐文件状态
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

# 音乐文件定义（使用数组而非关联数组）
MUSIC_FILES=(
    "main_menu_theme.ogg|修仙问道 - 主菜单"
    "map_exploration_theme.ogg|寻仙觅缘 - 地图探索"
    "normal_battle_theme.ogg|降妖除魔 - 普通战斗"
    "boss_battle_theme.ogg|生死对决 - Boss战"
    "tribulation_theme.ogg|雷劫降临 - 渡劫场景"
    "shop_theme.ogg|坊市繁华 - 仙家坊市"
    "rest_theme.ogg|修炼打坐 - 休息场景"
    "victory_theme.ogg|众妖伏诛 - 胜利曲目"
)

MUSIC_DIR="assets/music"

echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
echo -e "${CYAN}${BOLD}  背景音乐文件状态检查${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
echo ""

# 检查目录
if [ ! -d "$MUSIC_DIR" ]; then
    echo -e "${RED}❌ 目录不存在: $MUSIC_DIR${NC}"
    echo -e "${YELLOW}请创建目录并放置音乐文件${NC}"
    exit 1
fi

echo -e "${BLUE}📂 音乐目录: $MUSIC_DIR${NC}"
echo ""

# 统计变量
TOTAL=0
EXIST=0
MISSING=0
TOTAL_SIZE=0

# 检查每个文件
echo -e "${BOLD}文件状态:${NC}"
for entry in "${MUSIC_FILES[@]}"; do
    file="${entry%%|*}"
    name="${entry##*|}"

    TOTAL=$((TOTAL + 1))
    path="$MUSIC_DIR/$file"

    if [ -f "$path" ]; then
        EXIST=$((EXIST + 1))
        # 获取文件大小
        if [[ "$OSTYPE" == "darwin"* ]]; then
            size=$(stat -f%z "$path" 2>/dev/null || echo "0")
        else
            size=$(stat -c%s "$path" 2>/dev/null || echo "0")
        fi
        TOTAL_SIZE=$((TOTAL_SIZE + size))

        # 转换为人类可读格式
        if [ $size -ge 1048576 ]; then
            size_hr="$(echo "scale=1; $size/1048576" | bc)MB"
        elif [ $size -ge 1024 ]; then
            size_hr="$(echo "scale=1; $size/1024" | bc)KB"
        else
            size_hr="${size}B"
        fi

        echo -e "${GREEN}  ✓${NC} ${file} (${YELLOW}${size_hr}${NC})"
        echo -e "     ${name}"
    else
        MISSING=$((MISSING + 1))
        echo -e "${RED}  ✗${NC} ${file} ${RED}(缺失)${NC}"
        echo -e "     ${name}"
    fi
    echo ""
done

# 汇总统计
echo -e "${BOLD}─────────────────────────────────────────────────${NC}"
echo -e "${BOLD}统计:${NC}"
echo -e "  总计: ${BLUE}${TOTAL}${NC} 个文件"
echo -e "  已存在: ${GREEN}${EXIST}${NC} 个文件"
echo -e "  缺失: ${RED}${MISSING}${NC} 个文件"

if [ $EXIST -gt 0 ]; then
    # 转换总大小
    if [ $TOTAL_SIZE -ge 1048576 ]; then
        total_size_hr="$(echo "scale=2; $TOTAL_SIZE/1048576" | bc)MB"
    elif [ $TOTAL_SIZE -ge 1024 ]; then
        total_size_hr="$(echo "scale=2; $TOTAL_SIZE/1024" | bc)KB"
    else
        total_size_hr="${TOTAL_SIZE}B"
    fi
    echo -e "  总大小: ${YELLOW}${total_size_hr}${NC}"
fi
echo ""

# 检查代码占位符状态
echo -e "${BOLD}─────────────────────────────────────────────────${NC}"
CODE_FILE="src/components/background_music.rs"
if [ -f "$CODE_FILE" ]; then
    if grep -q "__PLACEHOLDER__" "$CODE_FILE"; then
        echo -e "${YELLOW}⚠ 代码状态: 占位符未移除${NC}"
        echo -e "  运行 ${CYAN}./scripts/replace_music_placeholders.sh${NC} 移除占位符"
    else
        echo -e "${GREEN}✓ 代码状态: 占位符已移除${NC}"
    fi
else
    echo -e "${RED}❌ 代码文件不存在: $CODE_FILE${NC}"
fi
echo ""

# 检查Suno Prompts文件
PROMPTS_FILE="assets/music/SUNO_PROMPTS.md"
if [ -f "$PROMPTS_FILE" ]; then
    echo -e "${GREEN}✓ Suno Prompts 文件存在${NC}"
    echo -e "  路径: ${CYAN}$PROMPTS_FILE${NC}"
else
    echo -e "${RED}✗ Suno Prompts 文件缺失${NC}"
    echo -e "  预期路径: ${CYAN}$PROMPTS_FILE${NC}"
fi
echo ""

# 下一步建议
echo -e "${BOLD}─────────────────────────────────────────────────${NC}"
if [ $MISSING -eq 0 ] && [ $EXIST -eq $TOTAL ] && ! grep -q "__PLACEHOLDER__" "$CODE_FILE" 2>/dev/null; then
    echo -e "${GREEN}${BOLD}✅ 音乐系统已就绪！${NC}"
    echo ""
    echo -e "下一步:"
    echo -e "  1. 运行测试: ${CYAN}cargo test --test background_music_tdd${NC}"
    echo -e "  2. 运行游戏: ${CYAN}cargo run${NC}"
elif [ $MISSING -gt 0 ] || [ $EXIST -eq 0 ]; then
    echo -e "${YELLOW}${BOLD}📋 待办事项:${NC}"
    echo ""
    echo -e "1. 使用 Suno 生成缺失的音乐文件:"
    echo -e "   ${CYAN}cat assets/music/SUNO_PROMPTS.md${NC}"
    echo ""
    echo -e "2. 下载并编辑音频文件（OGG格式）"
    echo ""
    echo -e "3. 放置到 ${CYAN}$MUSIC_DIR/${NC} 目录"
    echo ""
    echo -e "4. 移除代码占位符:"
    echo -e "   ${CYAN}./scripts/replace_music_placeholders.sh${NC}"
    echo ""
    echo -e "5. 验证:"
    echo -e "   ${CYAN}cargo test --test background_music_tdd${NC}"
else
    echo -e "${YELLOW}${BOLD}📋 待办事项:${NC}"
    echo ""
    echo -e "音乐文件已就绪，移除代码占位符:"
    echo -e "  ${CYAN}./scripts/replace_music_placeholders.sh${NC}"
fi
echo ""

# 退出码
if [ $MISSING -gt 0 ]; then
    exit 1
else
    exit 0
fi
