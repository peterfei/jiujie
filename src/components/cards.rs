//! 卡牌组件和系统

use bevy::prelude::*;

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

/// 卡牌稀有度
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
fn create_starting_deck() -> Vec<Card> {
    vec![
        // 5张攻击卡
        Card::new(
            0,
            "打击",
            "造成6点伤害",
            CardType::Attack,
            1,
            CardEffect::DealDamage { amount: 6 },
            CardRarity::Common,
        ),
        Card::new(
            1,
            "打击",
            "造成6点伤害",
            CardType::Attack,
            1,
            CardEffect::DealDamage { amount: 6 },
            CardRarity::Common,
        ),
        Card::new(
            2,
            "打击",
            "造成6点伤害",
            CardType::Attack,
            1,
            CardEffect::DealDamage { amount: 6 },
            CardRarity::Common,
        ),
        Card::new(
            3,
            "打击",
            "造成6点伤害",
            CardType::Attack,
            1,
            CardEffect::DealDamage { amount: 6 },
            CardRarity::Common,
        ),
        Card::new(
            4,
            "打击",
            "造成6点伤害",
            CardType::Attack,
            1,
            CardEffect::DealDamage { amount: 6 },
            CardRarity::Common,
        ),
        // 4张防御卡
        Card::new(
            5,
            "防御",
            "获得5点护甲",
            CardType::Defense,
            1,
            CardEffect::GainBlock { amount: 5 },
            CardRarity::Common,
        ),
        Card::new(
            6,
            "防御",
            "获得5点护甲",
            CardType::Defense,
            1,
            CardEffect::GainBlock { amount: 5 },
            CardRarity::Common,
        ),
        Card::new(
            7,
            "防御",
            "获得5点护甲",
            CardType::Defense,
            1,
            CardEffect::GainBlock { amount: 5 },
            CardRarity::Common,
        ),
        Card::new(
            8,
            "防御",
            "获得5点护甲",
            CardType::Defense,
            1,
            CardEffect::GainBlock { amount: 5 },
            CardRarity::Common,
        ),
        // 1张攻击抽牌卡
        Card::new(
            9,
            "突刺",
            "造成3点伤害，抽1张牌",
            CardType::Attack,
            1,
            CardEffect::AttackAndDraw {
                damage: 3,
                cards: 1,
            },
            CardRarity::Uncommon,
        ),
        // 2张治疗卡
        Card::new(
            10,
            "治疗",
            "恢复5点生命",
            CardType::Skill,
            1,
            CardEffect::Heal { amount: 5 },
            CardRarity::Uncommon,
        ),
        Card::new(
            11,
            "治疗",
            "恢复5点生命",
            CardType::Skill,
            1,
            CardEffect::Heal { amount: 5 },
            CardRarity::Uncommon,
        ),
    ]
}
