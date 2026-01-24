//! 战斗组件和系统

use bevy::prelude::*;

// ============================================================================
// 战斗状态
// ============================================================================

/// 战斗回合阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TurnPhase {
    /// 玩家回合开始
    #[default]
    PlayerStart,
    /// 玩家出牌阶段
    PlayerAction,
    /// 敌人回合
    EnemyTurn,
    /// 回合结束
    TurnEnd,
}

// ============================================================================
// 玩家组件
// ============================================================================

/// 玩家战斗属性
#[derive(Component, Debug, Clone)]
pub struct Player {
    /// 当前生命值
    pub hp: i32,
    /// 最大生命值
    pub max_hp: i32,
    /// 当前能量
    pub energy: i32,
    /// 最大能量
    pub max_energy: i32,
    /// 当前护甲
    pub block: i32,
    /// 当前金币
    pub gold: i32,
    /// 当前回合（从1开始）
    pub turn: u32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            hp: 80,
            max_hp: 80,
            energy: 3,
            max_energy: 3,
            block: 0,
            gold: 0,
            turn: 1,
        }
    }
}

impl Player {
    /// 受到伤害（护甲优先抵消）
    pub fn take_damage(&mut self, amount: i32) {
        let mut remaining_damage = amount;

        // 护甲优先抵消伤害
        if self.block > 0 {
            if self.block >= remaining_damage {
                self.block -= remaining_damage;
                remaining_damage = 0;
            } else {
                remaining_damage -= self.block;
                self.block = 0;
            }
        }

        // 剩余伤害扣除HP
        self.hp = (self.hp - remaining_damage).max(0);
    }

    /// 恢复生命
    pub fn heal(&mut self, amount: i32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    /// 获得护甲
    pub fn gain_block(&mut self, amount: i32) {
        self.block += amount;
    }

    /// 清空护甲（回合结束时）
    pub fn clear_block(&mut self) {
        self.block = 0;
    }

    /// 获得能量
    pub fn gain_energy(&mut self, amount: i32) {
        self.energy += amount;
    }

    /// 消耗能量
    pub fn use_energy(&mut self, amount: i32) -> bool {
        if self.energy >= amount {
            self.energy -= amount;
            true
        } else {
            false
        }
    }

    /// 回合开始时重置
    pub fn start_turn(&mut self) {
        self.energy = self.max_energy;
        self.turn += 1;
    }
}

// ============================================================================
// 敌人组件
// ============================================================================

/// 敌人战斗属性
#[derive(Component, Debug, Clone)]
pub struct Enemy {
    /// 敌人ID
    pub id: u32,
    /// 敌人名称
    pub name: String,
    /// 敌人类型
    pub enemy_type: EnemyType,
    /// 当前生命值
    pub hp: i32,
    /// 最大生命值
    pub max_hp: i32,
    /// 当前意图（下次行动）
    pub intent: EnemyIntent,
    /// AI模式配置
    pub ai_pattern: AiPattern,
    /// 当前攻击力加成（来自Buff）
    pub strength: i32,
    /// 当前护甲
    pub block: i32,
}

/// 敌人意图
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyIntent {
    /// 攻击
    Attack { damage: i32 },
    /// 防御
    Defend { block: i32 },
    /// 强化（给自身攻击力增益）
    Buff { strength: i32 },
    /// 等待
    Wait,
}

/// 敌人类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyType {
    /// 哥布林 - 攻击型，高攻击概率
    Goblin,
    /// 骷髅 - 均衡型
    Skeleton,
    /// 史莱姆 - 防御型，高防御概率
    Slime,
    /// Boss - 强力型，多种攻击模式
    Boss,
}

/// AI模式配置 - 定义敌人选择意图的概率
#[derive(Debug, Clone)]
pub struct AiPattern {
    /// 攻击概率 (0.0-1.0)
    pub attack_chance: f32,
    /// 防御概率 (0.0-1.0)
    pub defend_chance: f32,
    /// 强化概率 (0.0-1.0)
    pub buff_chance: f32,
    /// 伤害范围
    pub damage_range: (i32, i32),
    /// 防御范围
    pub block_range: (i32, i32),
    /// 强化范围
    pub buff_range: (i32, i32),
}

impl AiPattern {
    /// 哥布林模式 - 激进攻击型
    pub fn goblin() -> Self {
        Self {
            attack_chance: 0.7,
            defend_chance: 0.1,
            buff_chance: 0.2,
            damage_range: (8, 12),
            block_range: (3, 5),
            buff_range: (1, 2),
        }
    }

    /// 骷髅模式 - 均衡型
    pub fn skeleton() -> Self {
        Self {
            attack_chance: 0.5,
            defend_chance: 0.3,
            buff_chance: 0.2,
            damage_range: (6, 10),
            block_range: (5, 8),
            buff_range: (2, 3),
        }
    }

    /// 史莱姆模式 - 防御型
    pub fn slime() -> Self {
        Self {
            attack_chance: 0.3,
            defend_chance: 0.5,
            buff_chance: 0.2,
            damage_range: (4, 7),
            block_range: (8, 12),
            buff_range: (1, 2),
        }
    }

    /// Boss模式 - 强力型
    pub fn boss() -> Self {
        Self {
            attack_chance: 0.6,
            defend_chance: 0.2,
            buff_chance: 0.2,
            damage_range: (12, 18),
            block_range: (6, 10),
            buff_range: (3, 5),
        }
    }

    /// 根据敌人类型获取AI模式
    pub fn from_enemy_type(enemy_type: EnemyType) -> Self {
        match enemy_type {
            EnemyType::Goblin => Self::goblin(),
            EnemyType::Skeleton => Self::skeleton(),
            EnemyType::Slime => Self::slime(),
            EnemyType::Boss => Self::boss(),
        }
    }
}

impl Enemy {
    /// 创建新敌人（默认哥布林类型）
    pub fn new(id: u32, name: impl Into<String>, hp: i32) -> Self {
        let enemy_type = EnemyType::Goblin;
        let ai_pattern = AiPattern::from_enemy_type(enemy_type);
        Self {
            id,
            name: name.into(),
            enemy_type,
            hp,
            max_hp: hp,
            intent: EnemyIntent::Wait,
            ai_pattern,
            strength: 0,
            block: 0,
        }
    }

    /// 创建指定类型的敌人
    pub fn with_type(id: u32, name: impl Into<String>, hp: i32, enemy_type: EnemyType) -> Self {
        let ai_pattern = AiPattern::from_enemy_type(enemy_type);
        Self {
            id,
            name: name.into(),
            enemy_type,
            hp,
            max_hp: hp,
            intent: EnemyIntent::Wait,
            ai_pattern,
            strength: 0,
            block: 0,
        }
    }

    /// 受到伤害
    pub fn take_damage(&mut self, amount: i32) {
        self.hp = (self.hp - amount).max(0);
    }

    /// 设置意图
    pub fn set_intent(&mut self, intent: EnemyIntent) {
        self.intent = intent;
    }

    /// 检查是否死亡
    pub fn is_dead(&self) -> bool {
        self.hp <= 0
    }

    /// 使用AI选择新的意图
    pub fn choose_new_intent(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let roll: f32 = rng.gen();
        let intent = if roll < self.ai_pattern.attack_chance {
            // 攻击
            let base_damage = rng.gen_range(self.ai_pattern.damage_range.0..=self.ai_pattern.damage_range.1);
            let total_damage = base_damage + self.strength;
            EnemyIntent::Attack { damage: total_damage }
        } else if roll < self.ai_pattern.attack_chance + self.ai_pattern.defend_chance {
            // 防御
            let block = rng.gen_range(self.ai_pattern.block_range.0..=self.ai_pattern.block_range.1);
            EnemyIntent::Defend { block }
        } else {
            // 强化（移除Wait，确保总是选择一种行动）
            let strength = rng.gen_range(self.ai_pattern.buff_range.0..=self.ai_pattern.buff_range.1);
            EnemyIntent::Buff { strength }
        };

        self.intent = intent;
    }

    /// 执行意图（敌人回合行动）
    pub fn execute_intent(&mut self) -> EnemyIntent {
        match self.intent {
            EnemyIntent::Attack { damage } => {
                // 攻击意图直接返回，由系统处理
                EnemyIntent::Attack { damage }
            }
            EnemyIntent::Defend { block } => {
                // 获得护甲
                self.block += block;
                info!("{} 获得了 {} 点护甲", self.name, block);
                EnemyIntent::Defend { block }
            }
            EnemyIntent::Buff { strength } => {
                // 获得攻击力加成
                self.strength += strength;
                info!("{} 获得了 {} 点攻击力", self.name, strength);
                EnemyIntent::Buff { strength }
            }
            EnemyIntent::Wait => {
                info!("{} 等待中", self.name);
                EnemyIntent::Wait
            }
        }
    }

    /// 回合开始时清理临时效果
    pub fn start_turn(&mut self) {
        // 清空护甲
        self.block = 0;
        // 选择新的意图
        self.choose_new_intent();
    }
}

// ============================================================================
// 战斗资源
// ============================================================================

/// 战斗配置资源
#[derive(Resource, Debug, Clone)]
pub struct CombatConfig {
    /// 每回合基础能量
    pub base_energy: i32,
    /// 初始生命值
    pub initial_hp: i32,
}

impl Default for CombatConfig {
    fn default() -> Self {
        Self {
            base_energy: 3,
            initial_hp: 80,
        }
    }
}

/// 当前战斗回合状态
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CombatState {
    /// 当前回合阶段
    pub phase: TurnPhase,
    /// 本回合是否已抽牌
    pub cards_drawn_this_turn: bool,
}

/// 胜利延迟计时器（用于延迟进入奖励界面，让粒子特效播放）
#[derive(Resource, Debug, Clone)]
pub struct VictoryDelay {
    /// 是否正在延迟
    pub active: bool,
    /// 已经过的时间
    pub elapsed: f32,
    /// 延迟时长（秒）
    pub duration: f32,
}

impl VictoryDelay {
    pub fn new(duration: f32) -> Self {
        Self {
            active: false,
            elapsed: 0.0,
            duration,
        }
    }
}

impl Default for CombatState {
    fn default() -> Self {
        Self {
            phase: TurnPhase::PlayerStart,
            cards_drawn_this_turn: false,
        }
    }
}

/// 玩家持久化牌组资源（跨战斗保存）
#[derive(Resource, Debug, Clone)]
pub struct PlayerDeck {
    /// 牌组中的所有卡牌
    pub cards: Vec<Card>,
}

impl PlayerDeck {
    /// 创建新牌组
    pub fn new() -> Self {
        Self {
            cards: create_starting_deck(),
        }
    }

    /// 添加卡牌到牌组
    pub fn add_card(&mut self, card: Card) {
        self.cards.push(card);
    }

    /// 获取牌组大小
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// 重置牌组为初始状态（用于重新开始游戏）
    pub fn reset(&mut self) {
        self.cards = create_starting_deck();
    }
}

impl Default for PlayerDeck {
    fn default() -> Self {
        Self::new()
    }
}

// 需要导入 Card 类型
use crate::components::cards::{Card, create_starting_deck};
