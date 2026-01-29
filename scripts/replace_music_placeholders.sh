#!/bin/bash
#
# 背景音乐占位符替换脚本
#
# 使用说明：
# 1. 将Suno生成的音频文件放置在 assets/music/ 目录
# 2. 运行此脚本移除代码中的占位符
# 3. 运行 cargo test --test background_music_tdd 验证
#

set -e

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}═══════════════════════════════════════════════════${NC}"
echo -e "${YELLOW}  背景音乐占位符替换脚本${NC}"
echo -e "${YELLOW}═══════════════════════════════════════════════════${NC}"
echo ""

# 目标文件
TARGET_FILE="src/components/background_music.rs"

# 检查文件是否存在
if [ ! -f "$TARGET_FILE" ]; then
    echo -e "${RED}❌ 错误: 找不到文件 $TARGET_FILE${NC}"
    exit 1
fi

# 检查音乐目录
MUSIC_DIR="assets/music"
if [ ! -d "$MUSIC_DIR" ]; then
    echo -e "${YELLOW}⚠ 创建目录: $MUSIC_DIR${NC}"
    mkdir -p "$MUSIC_DIR"
fi

# 检查音乐文件（排除占位符）
MISSING_FILES=()
EXPECTED_FILES=(
    "main_menu_theme.ogg"
    "map_exploration_theme.ogg"
    "normal_battle_theme.ogg"
    "boss_battle_theme.ogg"
    "tribulation_theme.ogg"
    "shop_theme.ogg"
    "rest_theme.ogg"
    "victory_theme.ogg"
)

echo -e "${YELLOW}📋 检查音乐文件...${NC}"
for file in "${EXPECTED_FILES[@]}"; do
    if [ -f "$MUSIC_DIR/$file" ]; then
        echo -e "${GREEN}  ✓ $file${NC}"
    else
        echo -e "${RED}  ✗ $file (缺失)${NC}"
        MISSING_FILES+=("$file")
    fi
done
echo ""

# 如果有缺失文件，警告但不停止
if [ ${#MISSING_FILES[@]} -gt 0 ]; then
    echo -e "${YELLOW}⚠ 警告: ${#MISSING_FILES[@]} 个音乐文件缺失${NC}"
    echo -e "${YELLOW}  建议先生成音乐再运行此脚本${NC}"
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
    sed -i '' 's|music/__PLACEHOLDER__|music|g' "$TARGET_FILE"
else
    # Linux
    sed -i 's|music/__PLACEHOLDER__|music|g' "$TARGET_FILE"
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

# 显示替换后的路径示例
echo -e "${YELLOW}📝 替换后的文件路径示例:${NC}"
grep -A 10 'pub fn file_path' "$TARGET_FILE" | grep 'music/' | head -3 | sed 's/^/  /'
echo ""

# 运行测试提示
echo -e "${YELLOW}═══════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ 占位符替换完成！${NC}"
echo -e "${YELLOW}═══════════════════════════════════════════════════${NC}"
echo ""
echo -e "${YELLOW}下一步:${NC}"
echo -e "  1. 运行测试验证: ${GREEN}cargo test --test background_music_tdd${NC}"
echo -e "  2. 运行游戏测试音乐: ${GREEN}cargo run${NC}"
echo ""
echo -e "${YELLOW}如需恢复备份:${NC}"
echo -e "  mv $BACKUP_FILE $TARGET_FILE"
echo ""
