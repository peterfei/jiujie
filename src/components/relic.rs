//! 遗物系统组件
//!
//! 遗物提供永久性被动效果，是杀戮尖塔风格游戏的核心元素

use bevy::prelude::*;
use serde::{Serialize, Deserialize};

// ============================================================================
// 遗物定义
// ============================================================================

/// 遗物组件
#[derive(Component, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relic {
    /// 遗物ID
    pub id: RelicId,
    /// 遗物名称
    pub name: String,
    /// 遗物描述
    pub description: String,
    /// 遗物稀有度
    pub rarity: RelicRarity,
    /// 遗物效果
    pub effect: RelicEffect,
}

/// 遗物ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelicId {
    BurningBlood,     // 燃烧之血 - 战斗开始时对所有敌人造成3点伤害
    BagOfPreparation, // 准备背包 - 每场战斗开始时获得1张随机卡牌
    Anchor,           // 锚 - 每回合结束时，保留最多3张牌到下回合
    StrangeSpoon,     // 奇怪勺子 - 每打出第3张牌时，抽1张牌
}

/// 遗物稀有度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelicRarity {
    Common,    // 常见 - 基础效果
    Uncommon,  // 罕见 - 中等效果
    Rare,      // 稀有 - 强力效果
    Special,   // 特殊 - 独特效果
}

/// 遗物效果类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelicEffect {
    /// 战斗开始时触发（造成伤害、获得护甲等）
    OnCombatStart { damage: i32, block: i32, draw_cards: i32 },
    /// 回合开始时触发（获得能量、抽牌等）
    OnTurnStart { energy: i32, draw_cards: i32 },
    /// 回合结束时触发（保留手牌）
    OnTurnEnd { keep_cards: i32 },
    /// 抽牌时触发（额外抽牌）
    OnDraw { extra_cards: i32 },
    /// 造成伤害时触发（额外伤害）
    OnDealDamage { extra_damage: i32 },
    /// 受到伤害时触发（减少伤害）
    OnTakeDamage { reduction: i32 },
    /// 打出牌时触发（根据条件触发效果）
    OnCardPlayed { every_nth: i32, draw_cards: i32 },
}

/// 玩家遗物背包资源
#[derive(Resource, Debug, Clone, Default)]
pub struct RelicCollection {
    /// 已拥有的遗物列表
    pub relic: Vec<Relic>,
}

impl RelicCollection {
    /// 添加遗物（去重）
    pub fn add_relic(&mut self, relic: Relic) -> bool {
        // 检查是否已拥有同名遗物
        if self.relic.iter().any(|r| r.id == relic.id) {
            return false; // 已拥有，不重复添加
        }
        self.relic.push(relic);
        true
    }

    /// 检查是否拥有某个遗物
    pub fn has(&self, id: RelicId) -> bool {
        self.relic.iter().any(|r| r.id == id)
    }

    /// 获取拥有某个遗物的引用
    pub fn get(&self, id: RelicId) -> Option<&Relic> {
        self.relic.iter().find(|r| r.id == id)
    }

    /// 遗物数量
    pub fn count(&self) -> usize {
        self.relic.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.relic.is_empty()
    }
}

// ============================================================================
// 预定义遗物
// ============================================================================

impl Relic {
    /// 创建燃烧之血遗物 -> 飞剑符
    pub fn burning_blood() -> Self {
        Self {
            id: RelicId::BurningBlood,
            name: "飞剑符".to_string(),
            description: "每场战斗开始时，对所有敌人造成 3 点剑气伤害".to_string(),
            rarity: RelicRarity::Common,
            effect: RelicEffect::OnCombatStart { damage: 3, block: 0, draw_cards: 0 },
        }
    }

    /// 创建准备背包遗物 -> 乾坤袋
    pub fn bag_of_preparation() -> Self {
        Self {
            id: RelicId::BagOfPreparation,
            name: "乾坤袋".to_string(),
            description: "每场战斗开始时，从乾坤袋中额外获得 1 张随机功法".to_string(),
            rarity: RelicRarity::Common,
            effect: RelicEffect::OnCombatStart { damage: 0, block: 0, draw_cards: 1 },
        }
    }

    /// 创建锚遗物 -> 定风珠
    pub fn anchor() -> Self {
        Self {
            id: RelicId::Anchor,
            name: "定风珠".to_string(),
            description: "每回合结束时，保留最多 3 张手牌到下回合".to_string(),
            rarity: RelicRarity::Uncommon,
            effect: RelicEffect::OnTurnEnd { keep_cards: 3 },
        }
    }

    /// 创建奇怪勺子遗物 -> 聚灵阵
    pub fn strange_spoon() -> Self {
        Self {
            id: RelicId::StrangeSpoon,
            name: "聚灵阵".to_string(),
            description: "每打出第 3 张牌时，灵气涌动，抽 1 张牌".to_string(),
            rarity: RelicRarity::Rare,
            effect: RelicEffect::OnCardPlayed { every_nth: 3, draw_cards: 1 },
        }
    }

    /// 随机生成一个遗物
    pub fn random() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        match rng.gen_range(0..4) {
            0 => Self::burning_blood(),
            1 => Self::bag_of_preparation(),
            2 => Self::anchor(),
            _ => Self::strange_spoon(),
        }
    }

    /// 根据稀有度生成遗物
    pub fn by_rarity(rarity: RelicRarity) -> Vec<Self> {
        match rarity {
            RelicRarity::Common => vec![Self::burning_blood(), Self::bag_of_preparation()],
            RelicRarity::Uncommon => vec![Self::anchor()],
            RelicRarity::Rare => vec![Self::strange_spoon()],
            RelicRarity::Special => vec![],
        }
    }
}

// ============================================================================
// 事件
// ============================================================================

/// 遗物获取事件
#[derive(Event, Debug)]
pub struct RelicObtainedEvent {
    pub relic: Relic,
}

/// 遗物触发事件（用于日志和特效）
#[derive(Event, Debug)]
pub struct RelicTriggeredEvent {
    pub relic_id: RelicId,
    pub effect: RelicEffect,
}

// ============================================================================
// UI 标记组件
// ============================================================================

/// 遗物UI区域标记
#[derive(Component)]
pub struct RelicUiMarker;

/// 单个遗物显示项标记
#[derive(Component)]
pub struct RelicItemMarker;

/// 遗物稀有度对应的颜色
impl RelicRarity {
    /// 获取该稀有度对应的背景颜色
    pub fn color(&self) -> Color {
        match self {
            RelicRarity::Common => Color::srgb(0.7, 0.7, 0.7),      // 灰色
            RelicRarity::Uncommon => Color::srgb(0.3, 0.7, 0.3),    // 绿色
            RelicRarity::Rare => Color::srgb(0.7, 0.3, 0.9),        // 紫色
            RelicRarity::Special => Color::srgb(1.0, 0.8, 0.0),     // 金色
        }
    }

    /// 获取该稀有度对应的文本颜色
    pub fn text_color(&self) -> Color {
        match self {
            RelicRarity::Common => Color::srgb(0.2, 0.2, 0.2),
            RelicRarity::Uncommon => Color::srgb(0.1, 0.3, 0.1),
            RelicRarity::Rare => Color::srgb(0.2, 0.1, 0.3),
            RelicRarity::Special => Color::srgb(0.3, 0.2, 0.0),
        }
    }
}
