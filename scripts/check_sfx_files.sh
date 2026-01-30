#!/bin/bash
#
# 音效文件状态检查脚本
#
# 使用说明：
# 运行此脚本检查音效文件状态
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

# 音效文件定义（使用数组）
SFX_FILES=(
    # 卡牌相关
    "card_play.ogg|出牌 - 卡牌相关"
    "draw_card.ogg|抽牌 - 卡牌相关"
    "shuffle_card.ogg|洗牌 - 卡牌相关"
    "card_hover.ogg|卡牌悬停 - 卡牌相关"
    "card_select.ogg|卡牌选中 - 卡牌相关"
    # 战斗相关
    "player_attack.ogg|玩家攻击 - 战斗相关"
    "player_hit.ogg|玩家受击 - 战斗相关"
    "enemy_hit.ogg|敌人受击 - 战斗相关"
    "block.ogg|格挡 - 战斗相关"
    "critical_hit.ogg|暴击 - 战斗相关"
    "dodge.ogg|闪避 - 战斗相关"
    # 法术/技能
    "lightning_strike.ogg|天雷落下 - 法术技能"
    "fire_spell.ogg|火焰法术 - 法术技能"
    "ice_spell.ogg|冰霜法术 - 法术技能"
    "heal.ogg|治疗 - 法术技能"
    "buff_apply.ogg|增益施加 - 法术技能"
    "debuff_apply.ogg|减益施加 - 法术技能"
    "shield_up.ogg|护盾升起 - 法术技能"
    # 大招/终极技能
    "ultimate_start.ogg|大招起手 - 大招技能"
    "ultimate_release.ogg|大招释放 - 大招技能"
    "sword_strike.ogg|剑气斩击 - 大招技能"
    "thousand_swords.ogg|万剑归宗 - 大招技能"
    # UI交互
    "ui_click.ogg|UI点击 - UI交互"
    "ui_hover.ogg|UI悬停 - UI交互"
    "ui_confirm.ogg|UI确认 - UI交互"
    "ui_cancel.ogg|UI取消 - UI交互"
    "ui_error.ogg|UI错误 - UI交互"
    # 系统/事件
    "breakthrough_start.ogg|突破开始 - 系统事件"
    "breakthrough_success.ogg|突破成功 - 系统事件"
    "level_up.ogg|升级 - 系统事件"
    "gold_gain.ogg|获得金币 - 系统事件"
    "relic_obtain.ogg|获得遗物 - 系统事件"
    "victory.ogg|战斗胜利 - 系统事件"
    "defeat.ogg|战斗失败 - 系统事件"
    # 敌人相关
    "enemy_spawn.ogg|敌人生成 - 敌人相关"
    "enemy_death.ogg|敌人死亡 - 敌人相关"
    "boss_appear.ogg|Boss登场 - 敌人相关"
    "boss_death.ogg|Boss死亡 - 敌人相关"
)

SFX_DIR="assets/audio/sfx"

echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
echo -e "${CYAN}${BOLD}  音效文件状态检查${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
echo ""

# 检查目录
if [ ! -d "$SFX_DIR" ]; then
    echo -e "${YELLOW}⚠ 目录不存在: $SFX_DIR${NC}"
    echo -e "${YELLOW}创建目录...${NC}"
    mkdir -p "$SFX_DIR"
fi

echo -e "${BLUE}📂 音效目录: $SFX_DIR${NC}"
echo ""

# 统计变量
TOTAL=0
EXIST=0
MISSING=0
TOTAL_SIZE=0

# 检查每个文件
echo -e "${BOLD}文件状态:${NC}"
for entry in "${SFX_FILES[@]}"; do
    file="${entry%%|*}"
    name="${entry##*|}"

    TOTAL=$((TOTAL + 1))
    path="$SFX_DIR/$file"

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
    else
        MISSING=$((MISSING + 1))
        echo -e "${RED}  ✗${NC} ${file} ${RED}(缺失)${NC}"
    fi
done
echo ""

# 汇总统计
echo -e "${BOLD}─────────────────────────────────────────────────${NC}"
echo -e "${BOLD}统计:${NC}"
echo -e "  总计: ${BLUE}${TOTAL}${NC} 个文件"
echo -e "  已存在: ${GREEN}${EXIST}${NC} 个文件"
echo -e "  缺失: ${RED}${MISSING}${NC} 个文件"

if [ $EXIST -gt 0 ]; then
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
CODE_FILE="src/components/audio.rs"
if [ -f "$CODE_FILE" ]; then
    if grep -q "__PLACEHOLDER__" "$CODE_FILE"; then
        echo -e "${YELLOW}⚠ 代码状态: 占位符未移除${NC}"
        echo -e "  运行 ${CYAN}./scripts/replace_sfx_placeholders.sh${NC} 移除占位符"
    else
        echo -e "${GREEN}✓ 代码状态: 占位符已移除${NC}"
    fi
else
    echo -e "${RED}❌ 代码文件不存在: $CODE_FILE${NC}"
fi
echo ""

# 检查音效指南文件
GUIDE_FILE="assets/audio/sfx/SOUND_EFFECTS_GUIDE.md"
if [ -f "$GUIDE_FILE" ]; then
    echo -e "${GREEN}✓ 音效资源指南存在${NC}"
    echo -e "  路径: ${CYAN}$GUIDE_FILE${NC}"
else
    echo -e "${RED}✗ 音效资源指南缺失${NC}"
    echo -e "  预期路径: ${CYAN}$GUIDE_FILE${NC}"
fi
echo ""

# 下一步建议
echo -e "${BOLD}─────────────────────────────────────────────────${NC}"
if [ $MISSING -eq 0 ] && [ $EXIST -eq $TOTAL ] && ! grep -q "__PLACEHOLDER__" "$CODE_FILE" 2>/dev/null; then
    echo -e "${GREEN}${BOLD}✅ 音效系统已就绪！${NC}"
    echo ""
    echo -e "下一步:"
    echo -e "  1. 运行测试: ${CYAN}cargo test --test sound_effects_tdd${NC}"
    echo -e "  2. 运行游戏: ${CYAN}cargo run${NC}"
elif [ $MISSING -gt 0 ] || [ $EXIST -eq 0 ]; then
    echo -e "${YELLOW}${BOLD}📋 待办事项:${NC}"
    echo ""
    echo -e "1. 查看音效资源获取指南:"
    echo -e "   ${CYAN}cat assets/audio/sfx/SOUND_EFFECTS_GUIDE.md${NC}"
    echo ""
    echo -e "2. 从推荐网站下载音效文件"
    echo ""
    echo -e "3. 编辑处理为OGG Vorbis格式"
    echo ""
    echo -e "4. 放置到 ${CYAN}$SFX_DIR/${NC} 目录"
    echo ""
    echo -e "5. 移除代码占位符:"
    echo -e "   ${CYAN}./scripts/replace_sfx_placeholders.sh${NC}"
    echo ""
    echo -e "6. 验证:"
    echo -e "   ${CYAN}cargo test --test sound_effects_tdd${NC}"
else
    echo -e "${YELLOW}${BOLD}📋 待办事项:${NC}"
    echo ""
    echo -e "音效文件已就绪，移除代码占位符:"
    echo -e "  ${CYAN}./scripts/replace_sfx_placeholders.sh${NC}"
fi
echo ""

# 退出码
if [ $MISSING -gt 0 ]; then
    exit 1
else
    exit 0
fi
