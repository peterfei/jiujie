//! 卡牌组件和系统

use bevy::prelude::*;
use rand::prelude::SliceRandom;

// ============================================================================
// 卡牌组件
// ============================================================================

/// 卡牌组件
#[derive(Component, Debug, Clone)]
pub struct Card {
    /// 卡牌ID
    pub id: u32,
    /// 卡牌名称
    pub name: String,
    /// 卡牌描述
    pub description: String,
    /// 卡牌类型
    pub card_type: CardType,
    /// 能量消耗
    pub cost: i32,
    /// 卡牌效果
    pub effect: CardEffect,
    /// 稀有度
    pub rarity: CardRarity,
}

/// 卡牌类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardType {
    /// 攻击卡
    Attack,
    /// 防御卡
    Defense,
    /// 技能卡
    Skill,
    /// 能力卡（持续效果）
    Power,
}

/// 卡牌效果
#[derive(Debug, Clone, PartialEq)]
pub enum CardEffect {
    /// 造成伤害
    DealDamage { amount: i32 },
    /// 获得护甲
    GainBlock { amount: i32 },
    /// 治疗生命
    Heal { amount: i32 },
    /// 抽牌
    DrawCards { amount: i32 },
    /// 获得能量
    GainEnergy { amount: i32 },
    /// 造成伤害并抽牌
    AttackAndDraw { damage: i32, cards: i32 },
    /// 多重攻击
    MultiAttack { damage: i32, times: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardRarity {
    /// 普通
    Common,
    /// 稀有
    Uncommon,
    /// 稀有
    Rare,
    /// 特殊
    Special,
}

impl CardRarity {
    pub fn get_chinese_name(&self) -> &str {
        match self {
            CardRarity::Common => "凡阶",
            CardRarity::Uncommon => "玄阶",
            CardRarity::Rare => "地阶",
            CardRarity::Special => "天阶",
        }
    }
}

impl Card {
    /// 创建一张新卡牌
    pub fn new(
        id: u32,
        name: impl Into<String>,
        description: impl Into<String>,
        card_type: CardType,
        cost: i32,
        effect: CardEffect,
        rarity: CardRarity,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            description: description.into(),
            card_type,
            cost,
            effect,
            rarity,
        }
    }

    /// 获取卡牌显示的颜色
    pub fn get_color(&self) -> Color {
        match self.card_type {
            CardType::Attack => Color::srgb(0.9, 0.3, 0.3),   // 红色
            CardType::Defense => Color::srgb(0.3, 0.5, 0.9),   // 蓝色
            CardType::Skill => Color::srgb(0.4, 0.7, 0.4),     // 绿色
            CardType::Power => Color::srgb(0.7, 0.3, 0.7),     // 紫色
        }
    }
}

// ============================================================================
// 牌组系统
// ============================================================================

/// 抽牌堆
#[derive(Component, Debug, Clone)]
pub struct DrawPile {
    /// 卡牌列表
    pub cards: Vec<Card>,
    /// 剩余卡牌数
    pub count: usize,
}

impl DrawPile {
    /// 创建新的抽牌堆
    pub fn new(cards: Vec<Card>) -> Self {
        let count = cards.len();
        Self { cards, count }
    }

    /// 抽一张牌
    pub fn draw_card(&mut self) -> Option<Card> {
        if self.cards.is_empty() {
            None
        } else {
            self.count -= 1;
            Some(self.cards.remove(0))
        }
    }

    /// 洗牌（将弃牌堆的卡牌加入抽牌堆并打乱）
    pub fn shuffle_from_discard(&mut self, mut discard_cards: Vec<Card>) {
        use rand::seq::SliceRandom;
        discard_cards.shuffle(&mut rand::thread_rng());
        self.cards.extend(discard_cards);
        self.count = self.cards.len();
    }
}

/// 弃牌堆
#[derive(Component, Debug, Clone)]
pub struct DiscardPile {
    /// 卡牌列表
    pub cards: Vec<Card>,
    /// 剩余卡牌数
    pub count: usize,
}

impl DiscardPile {
    /// 创建新的弃牌堆
    pub fn new() -> Self {
        Self {
            cards: Vec::new(),
            count: 0,
        }
    }

    /// 添加卡牌到弃牌堆
    pub fn add_card(&mut self, card: Card) {
        self.cards.push(card);
        self.count += 1;
    }

    /// 清空弃牌堆
    pub fn clear(&mut self) -> Vec<Card> {
        let cards = self.cards.clone();
        self.cards.clear();
        self.count = 0;
        cards
    }
}

/// 手牌
#[derive(Component, Debug, Clone)]
pub struct Hand {
    /// 卡牌列表
    pub cards: Vec<Card>,
    /// 最大手牌数
    pub max_size: usize,
}

impl Hand {
    /// 创建新手牌
    pub fn new(max_size: usize) -> Self {
        Self {
            cards: Vec::new(),
            max_size,
        }
    }

    /// 添加卡牌到手牌
    pub fn add_card(&mut self, card: Card) -> bool {
        if self.cards.len() < self.max_size {
            self.cards.push(card);
            true
        } else {
            false
        }
    }

    /// 移除卡牌（打出）
    pub fn remove_card(&mut self, index: usize) -> Option<Card> {
        if index < self.cards.len() {
            Some(self.cards.remove(index))
        } else {
            None
        }
    }

    /// 获取手牌数量
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

/// 牌组配置
#[derive(Resource, Debug, Clone)]
pub struct DeckConfig {
    /// 初始牌组
    pub starting_deck: Vec<Card>,
    /// 手牌上限
    pub max_hand_size: usize,
    /// 每回合抽牌数
    pub cards_per_turn: usize,
}

impl Default for DeckConfig {
    fn default() -> Self {
        Self {
            starting_deck: create_starting_deck(),
            max_hand_size: 10,
            cards_per_turn: 5,
        }
    }
}

/// 初始牌组
pub fn create_starting_deck() -> Vec<Card> {
    vec![
        // 5张道法（攻击）卡 - 御剑术
        Card::new(
            0,
            "御剑术",
            "造成6点伤害",
            CardType::Attack,
            1,
            CardEffect::DealDamage { amount: 6 },
            CardRarity::Common,
        ),
        Card::new(
            1,
            "御剑术",
            "造成6点伤害",
            CardType::Attack,
            1,
            CardEffect::DealDamage { amount: 6 },
            CardRarity::Common,
        ),
        Card::new(
            2,
            "御剑术",
            "造成6点伤害",
            CardType::Attack,
            1,
            CardEffect::DealDamage { amount: 6 },
            CardRarity::Common,
        ),
        Card::new(
            3,
            "御剑术",
            "造成6点伤害",
            CardType::Attack,
            1,
            CardEffect::DealDamage { amount: 6 },
            CardRarity::Common,
        ),
        Card::new(
            4,
            "御剑术",
            "造成6点伤害",
            CardType::Attack,
            1,
            CardEffect::DealDamage { amount: 6 },
            CardRarity::Common,
        ),
        // 4张护体（防御）卡 - 金光咒
        Card::new(
            5,
            "金光咒",
            "获得5点护盾",
            CardType::Defense,
            1,
            CardEffect::GainBlock { amount: 5 },
            CardRarity::Common,
        ),
        Card::new(
            6,
            "金光咒",
            "获得5点护盾",
            CardType::Defense,
            1,
            CardEffect::GainBlock { amount: 5 },
            CardRarity::Common,
        ),
        Card::new(
            7,
            "金光咒",
            "获得5点护盾",
            CardType::Defense,
            1,
            CardEffect::GainBlock { amount: 5 },
            CardRarity::Common,
        ),
        Card::new(
            8,
            "金光咒",
            "获得5点护盾",
            CardType::Defense,
            1,
            CardEffect::GainBlock { amount: 5 },
            CardRarity::Common,
        ),
        // 1张剑气卡 - 剑气斩
        Card::new(
            9,
            "剑气斩",
            "造成3点伤害，抽1张牌",
            CardType::Attack,
            1,
            CardEffect::AttackAndDraw {
                damage: 3,
                cards: 1,
            },
            CardRarity::Uncommon,
        ),
        // 2张回复卡 - 回春术
        Card::new(
            10,
            "回春术",
            "恢复5点道行",
            CardType::Skill,
            1,
            CardEffect::Heal { amount: 5 },
            CardRarity::Uncommon,
        ),
        Card::new(
            11,
            "回春术",
            "恢复5点道行",
            CardType::Skill,
            1,
            CardEffect::Heal { amount: 5 },
            CardRarity::Uncommon,
        ),
    ]
}

// ============================================================================
// 卡牌池和奖励系统
// ============================================================================

/// 卡牌池 - 包含所有可获得的卡牌
pub struct CardPool;

impl CardPool {
    /// 获取所有可获得的卡牌
    pub fn all_cards() -> Vec<Card> {
        vec![
            // === 普通功法 ===
            Card::new(100, "雷法·掌心雷", "造成12点雷击伤害", CardType::Attack, 2, CardEffect::DealDamage { amount: 12 }, CardRarity::Common),
            Card::new(101, "不动明王", "获得8点护盾", CardType::Defense, 1, CardEffect::GainBlock { amount: 8 }, CardRarity::Common),
            Card::new(102, "疾风刺", "造成4点快速伤害", CardType::Attack, 0, CardEffect::DealDamage { amount: 4 }, CardRarity::Common),
            // === 稀有功法 ===
            Card::new(200, "御剑·流云", "造成8点伤害，抽2张牌", CardType::Attack, 2, CardEffect::AttackAndDraw { damage: 8, cards: 2 }, CardRarity::Uncommon),
            Card::new(201, "太极图", "获得12点护盾", CardType::Defense, 2, CardEffect::GainBlock { amount: 12 }, CardRarity::Uncommon),
            Card::new(202, "甘霖咒", "恢复10点道行", CardType::Skill, 2, CardEffect::Heal { amount: 10 }, CardRarity::Uncommon),
            Card::new(203, "破军剑", "造成6点伤害，抽1张牌", CardType::Attack, 1, CardEffect::AttackAndDraw { damage: 6, cards: 1 }, CardRarity::Uncommon),
            Card::new(204, "神识全开", "抽3张牌，获得2点灵力", CardType::Skill, 1, CardEffect::DrawCards { amount: 3 }, CardRarity::Uncommon),
            // === 传说功法 ===
            Card::new(300, "九天神雷", "造成20点毁灭伤害", CardType::Attack, 3, CardEffect::DealDamage { amount: 20 }, CardRarity::Rare),
            Card::new(301, "金身法相", "获得15点护盾，恢复5点道行", CardType::Defense, 2, CardEffect::GainBlock { amount: 15 }, CardRarity::Rare),
            Card::new(302, "万物回春", "恢复15点道行", CardType::Skill, 2, CardEffect::Heal { amount: 15 }, CardRarity::Rare),
        ]
    }

    /// 根据稀有度获取卡牌
    pub fn get_by_rarity(rarity: CardRarity) -> Vec<Card> {
        Self::all_cards().into_iter().filter(|c| c.rarity == rarity).collect()
    }

    /// 随机获取指定数量的卡牌（用于奖励）
    pub fn random_cards(count: usize) -> Vec<Card> {
        let all = Self::all_cards();
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        all.choose_multiple(&mut rng, count).cloned().collect()
    }

    /// 随机获取卡牌（偏向稀有度）
    pub fn random_rewards(count: usize) -> Vec<Card> {
        let mut rewards = Vec::new();
        use rand::Rng;

        for i in 0..count {
            let mut rng = rand::thread_rng();
            // 奖励概率：50%普通，40%稀有，10%罕见
            let rarity_roll = rng.gen::<f32>();
            let rarity = if rarity_roll < 0.5 {
                CardRarity::Common
            } else if rarity_roll < 0.9 {
                CardRarity::Uncommon
            } else {
                CardRarity::Rare
            };

            let cards = Self::get_by_rarity(rarity);
            if let Some(card) = cards.choose(&mut rng) {
                // 为每张卡创建唯一ID
                let mut card = card.clone();
                card.id = 1000 + i as u32;
                rewards.push(card);
            }
        }

        rewards
    }

    /// 获取筑基期本命功法
    pub fn get_innate_spell() -> Card {
        Card::new(
            999,
            "青莲剑歌",
            "造成30点穿透伤害，恢复10点灵力",
            CardType::Attack,
            3,
            CardEffect::MultiAttack { damage: 10, times: 3 },
            CardRarity::Special,
        )
    }
}

/// 奖励选项组件
#[derive(Component, Clone)]
pub struct RewardCard {
    /// 卡牌数据
    pub card: Card,
    /// 是否被选中
    pub selected: bool,
}
