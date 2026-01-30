#!/bin/bash
#
# 音效占位符替换脚本
#
# 使用说明：
# 1. 将音效文件放置在 assets/audio/sfx/ 目录
# 2. 运行此脚本移除代码中的占位符
#

set -e

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

TARGET_FILE="src/components/audio.rs"

echo -e "${YELLOW}═══════════════════════════════════════════════════${NC}"
echo -e "${YELLOW}  音效占位符替换脚本${NC}"
echo -e "${YELLOW}═══════════════════════════════════════════════════${NC}"
echo ""

# 目标文件
if [ ! -f "$TARGET_FILE" ]; then
    echo -e "${RED}❌ 错误: 找不到文件 $TARGET_FILE${NC}"
    exit 1
fi

# 检查音效目录
SFX_DIR="assets/audio/sfx"
if [ ! -d "$SFX_DIR" ]; then
    echo -e "${YELLOW}⚠ 创建目录: $SFX_DIR${NC}"
    mkdir -p "$SFX_DIR"
fi

# 音效文件列表
SFX_FILES=(
    "card_play.ogg"
    "draw_card.ogg"
    "shuffle_card.ogg"
    "card_hover.ogg"
    "card_select.ogg"
    "player_attack.ogg"
    "player_hit.ogg"
    "enemy_hit.ogg"
    "block.ogg"
    "critical_hit.ogg"
    "dodge.ogg"
    "lightning_strike.ogg"
    "fire_spell.ogg"
    "ice_spell.ogg"
    "heal.ogg"
    "buff_apply.ogg"
    "debuff_apply.ogg"
    "shield_up.ogg"
    "ultimate_start.ogg"
    "ultimate_release.ogg"
    "sword_strike.ogg"
    "thousand_swords.ogg"
    "ui_click.ogg"
    "ui_hover.ogg"
    "ui_confirm.ogg"
    "ui_cancel.ogg"
    "ui_error.ogg"
    "breakthrough_start.ogg"
    "breakthrough_success.ogg"
    "level_up.ogg"
    "gold_gain.ogg"
    "relic_obtain.ogg"
    "victory.ogg"
    "defeat.ogg"
    "enemy_spawn.ogg"
    "enemy_death.ogg"
    "boss_appear.ogg"
    "boss_death.ogg"
)

echo -e "${YELLOW}📋 检查音效文件...${NC}"
MISSING_COUNT=0
for file in "${SFX_FILES[@]}"; do
    if [ -f "$SFX_DIR/$file" ]; then
        echo -e "${GREEN}  ✓ $file${NC}"
    else
        echo -e "${RED}  ✗ $file (缺失)${NC}"
        MISSING_COUNT=$((MISSING_COUNT + 1))
    fi
done
echo ""

# 如果有缺失文件，警告但不停止
if [ $MISSING_COUNT -gt 0 ]; then
    echo -e "${YELLOW}⚠ 警告: ${MISSING_COUNT} 个音效文件缺失${NC}"
    echo -e "${YELLOW}  建议先从音效资源网站获取文件${NC}"
    echo ""
    read -p "是否继续替换占位符？(y/N) " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${YELLOW}❌ 操作已取消${NC}"
        exit 0
    fi
fi

# 备份原文件
BACKUP_FILE="$TARGET_FILE.backup"
echo -e "${YELLOW}💾 备份原文件到: $BACKUP_FILE${NC}"
cp "$TARGET_FILE" "$BACKUP_FILE"

# 替换占位符
echo -e "${YELLOW}🔄 替换占位符...${NC}"

# 使用sed替换所有包含 __PLACEHOLDER__ 的路径
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    sed -i '' 's|audio/sfx/__PLACEHOLDER__|audio/sfx|g' "$TARGET_FILE"
else
    # Linux
    sed -i 's|audio/sfx/__PLACEHOLDER__|audio/sfx|g' "$TARGET_FILE"
fi

echo -e "${GREEN}✓ 占位符已移除${NC}"
echo ""

# 验证替换
echo -e "${YELLOW}🔍 验证替换结果...${NC}"
if grep -q "__PLACEHOLDER__" "$TARGET_FILE"; then
    echo -e "${RED}❌ 错误: 仍有占位符残留${NC}"
    echo -e "${YELLOW}恢复备份...${NC}"
    mv "$BACKUP_FILE" "$TARGET_FILE"
    exit 1
else
    echo -e "${GREEN}✓ 所有占位符已成功移除${NC}"
fi
echo ""

echo -e "${YELLOW}═══════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ 占位符替换完成！${NC}"
echo -e "${YELLOW}═══════════════════════════════════════════════════${NC}"
echo ""
echo -e "${YELLOW}下一步:${NC}"
echo -e "  1. 运行测试验证: ${GREEN}cargo test --test sound_effects_tdd${NC}"
echo -e "  2. 运行游戏测试音效: ${GREEN}cargo run${NC}"
echo ""
echo -e "${YELLOW}如需恢复备份:${NC}"
echo -e "  mv $BACKUP_FILE $TARGET_FILE"
echo ""
