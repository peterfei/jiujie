//! 战斗组件和系统

use bevy::prelude::*;
use serde::{Serialize, Deserialize};

// ============================================================================
// 战斗状态
// ============================================================================

/// 战斗回合阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
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
#[derive(Component, Resource, Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub hp: i32,
    pub max_hp: i32,
    pub energy: i32,
    pub max_energy: i32,
    pub block: i32,
    pub gold: i32,
    pub turn: u32,
    /// 中毒层数 (每回合开始扣血)
    pub poison: i32,
    /// 虚弱层数 (攻击力降低)
    pub weakness: i32,
    /// 易伤层数 (受创增加)
    pub vulnerable: i32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            hp: 80, max_hp: 80,
            energy: 3, max_energy: 3,
            block: 0, gold: 100, turn: 1,
            poison: 0, weakness: 0, vulnerable: 0,
        }
    }
}

impl Player {
    /// 计算实际造成的伤害 (考虑虚弱)
    pub fn calculate_outgoing_damage(&self, base_amount: i32) -> i32 {
        self.calculate_outgoing_damage_with_env(base_amount, None)
    }

    pub fn calculate_outgoing_damage_with_env(&self, base_amount: i32, environment: Option<&Environment>) -> i32 {
        let damage = if self.weakness > 0 {
            (base_amount as f32 * 0.75) as i32
        } else {
            base_amount
        };

        if let Some(env) = environment {
            (damage as f32 * env.damage_modifier) as i32
        } else {
            damage
        }
    }

    /// 计算实际受到的伤害 (考虑易伤)
    pub fn calculate_incoming_damage(&self, base_amount: i32) -> i32 {
        self.calculate_incoming_damage_with_env(base_amount, None)
    }

    pub fn calculate_incoming_damage_with_env(&self, base_amount: i32, _environment: Option<&Environment>) -> i32 {
        if self.vulnerable > 0 {
            (base_amount as f32 * 1.5) as i32
        } else {
            base_amount
        }
    }

    /// 受到伤害（护甲优先抵消）
    pub fn take_damage(&mut self, amount: i32) {
        self.take_damage_with_env(amount, None);
    }

    pub fn take_damage_with_env(&mut self, amount: i32, environment: Option<&Environment>) {
        let mut remaining_damage = self.calculate_incoming_damage_with_env(amount, environment);

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
        self.gain_block_with_env(amount, None);
    }

    pub fn gain_block_with_env(&mut self, amount: i32, environment: Option<&Environment>) {
        let modifier = environment.map(|e| e.block_modifier).unwrap_or(1.0);
        let final_amount = (amount as f32 * modifier) as i32;
        self.block += final_amount;
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

#[derive(Component)]
pub struct DamageNumber {
    pub value: i32,
    pub timer: f32,
    pub lifetime: f32,
    pub velocity: Vec2,
}

#[derive(Component)]
pub struct BlockIconMarker {
    pub owner: Entity,
}

#[derive(Component)]
pub struct BlockText;

#[derive(Component)]
pub struct StatusIndicator {
    pub owner: Entity,
}

// --- UI 标记组件 ---
#[derive(Component)]
pub struct EnemyHpText {
    pub owner: Entity,
}

#[derive(Component)]
pub struct EnemyIntentText {
    pub owner: Entity,
}

#[derive(Component)]
pub struct IntentIconMarker {
    pub owner: Entity,
}

#[derive(Component)]
pub struct EnemyStatusUi {
    pub owner: Entity,
}

#[derive(Component)]
pub struct PlayerHpText;

#[derive(Component)]
pub struct PlayerEnergyText;

#[derive(Component)]
pub struct PlayerBlockText;

#[derive(Component)]
pub struct TopBar;

#[derive(Component)]
pub struct TopBarHpText;

#[derive(Component)]
pub struct TopBarGoldText;

#[derive(Component)]
pub struct EnergyOrb;

#[derive(Component)]
pub struct EndTurnButton;

#[derive(Component)]
pub struct HandArea;

#[derive(Component)]
pub struct CombatUiRoot;

#[derive(Component)]
pub struct PlayerHpBarMarker;

#[derive(Component)]
pub struct PlayerHpBufferMarker;

#[derive(Component)]
pub struct EnemyHpBarMarker {
    pub owner: Entity,
}

#[derive(Component)]
pub struct EnemyHpBufferMarker {
    pub owner: Entity,
}

#[derive(Component)]
pub struct CardDescriptionMarker {
    pub card_id: u32,
}

#[derive(Event)]
pub struct StatusEffectEvent {
    pub target: Entity,
    pub msg: String,
    pub color: Color,
}

impl DamageNumber {
    pub fn new(value: i32) -> Self {
        Self {
            value,
            timer: 0.0,
            lifetime: 1.0,
            velocity: Vec2::new(0.0, 50.0),
        }
    }
}

#[derive(Event)]
pub struct DamageEffectEvent {
    pub position: Vec2,
    pub amount: i32,
}

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
    pub ai_pattern: AiPattern,
    /// 攻击力加成
    pub strength: i32,
    /// 当前护甲
    pub block: i32,
    /// 行动轮次（用于 BOSS 固定招式循环）
    pub turn_count: u32,
    /// 虚弱层数
    pub weakness: i32,
    /// 易伤层数
    pub vulnerable: i32,
    /// 中毒层数
    pub poison: i32,
}

/// 敌人意图
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnemyIntent {
    /// 攻击
    Attack { damage: i32 },
    /// 防御
    Defend { block: i32 },
    /// 强化（给自身攻击力增益）
    Buff { strength: i32 },
    /// 减益（给玩家施加负面效果）
    Debuff { poison: i32, weakness: i32 },
    /// 诅咒（向玩家牌组加入负面卡牌）
    Curse { card_id: u32 },
    /// 封印（封印玩家的手牌槽位）
    Seal { slot_index: usize, duration: u32 },
    /// 等待
    Wait,
}

// ============================================================================
// 环境系统
// ============================================================================

/// 战斗环境效果
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub name: String,
    pub description: String,
    /// 伤害加成系数 (例如 1.2 表示增加 20%)
    pub damage_modifier: f32,
    /// 护甲加成系数
    pub block_modifier: f32,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            name: "常态".to_string(),
            description: "灵气平稳，无特殊效果。".to_string(),
            damage_modifier: 1.0,
            block_modifier: 1.0,
        }
    }
}

impl Environment {
    pub fn thunder_storm() -> Self {
        Self {
            name: "雷暴".to_string(),
            description: "雷元素充盈，伤害提升 20%".to_string(),
            damage_modifier: 1.2,
            block_modifier: 1.0,
        }
    }
    
    pub fn thick_fog() -> Self {
        Self {
            name: "浓雾".to_string(),
            description: "视线受阻，防御效果提升 20%".to_string(),
            damage_modifier: 1.0,
            block_modifier: 1.2,
        }
    }
}

/// 敌人类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnemyType {
    /// 嗜血妖狼 - 激进攻击
    DemonicWolf,
    /// 剧毒蛛 - 施加中毒
    PoisonSpider,
    /// 怨灵 - 施加虚弱
    CursedSpirit,
    /// 筑基大妖 - 强力首领
    GreatDemon,
}

/// AI模式配置 - 定义敌人选择意图的概率
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiPattern {
    pub attack_chance: f32,
    pub defend_chance: f32,
    pub buff_chance: f32,
    pub debuff_chance: f32,
    pub curse_chance: f32,
    pub seal_chance: f32,
    pub damage_range: (i32, i32),
    pub block_range: (i32, i32),
    pub buff_range: (i32, i32),
}

impl AiPattern {
    pub fn demonic_wolf() -> Self {
        Self {
            attack_chance: 0.7, defend_chance: 0.1, buff_chance: 0.2, debuff_chance: 0.0,
            curse_chance: 0.0, seal_chance: 0.0,
            damage_range: (8, 12), block_range: (3, 5), buff_range: (1, 3),
        }
    }

    pub fn poison_spider() -> Self {
        Self {
            attack_chance: 0.3, defend_chance: 0.2, buff_chance: 0.0, debuff_chance: 0.3,
            curse_chance: 0.0, seal_chance: 0.2, // 蜘蛛会封印气穴
            damage_range: (5, 8), block_range: (4, 6), buff_range: (0, 0),
        }
    }

    pub fn cursed_spirit() -> Self {
        Self {
            attack_chance: 0.2, defend_chance: 0.2, buff_chance: 0.0, debuff_chance: 0.2,
            curse_chance: 0.4, seal_chance: 0.0, // 怨灵擅长施加诅咒
            damage_range: (10, 15), block_range: (5, 10), buff_range: (0, 0),
        }
    }

    pub fn great_demon() -> Self {
        Self {
            attack_chance: 0.5, defend_chance: 0.2, buff_chance: 0.1, debuff_chance: 0.1,
            curse_chance: 0.05, seal_chance: 0.05,
            damage_range: (12, 18), block_range: (6, 10), buff_range: (3, 5),
        }
    }

    pub fn from_enemy_type(enemy_type: EnemyType) -> Self {
        match enemy_type {
            EnemyType::DemonicWolf => Self::demonic_wolf(),
            EnemyType::PoisonSpider => Self::poison_spider(),
            EnemyType::CursedSpirit => Self::cursed_spirit(),
            EnemyType::GreatDemon => Self::great_demon(),
        }
    }
}

impl Enemy {
    /// 创建新敌人（默认嗜血妖狼类型）
    pub fn new(id: u32, name: impl Into<String>, hp: i32) -> Self {
        let enemy_type = EnemyType::DemonicWolf;
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
            turn_count: 0,
            weakness: 0,
            vulnerable: 0,
            poison: 0,
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
            turn_count: 0,
            weakness: 0,
            vulnerable: 0,
            poison: 0,
        }
    }

    /// 计算实际造成的伤害 (考虑虚弱)
    pub fn calculate_outgoing_damage(&self, base_amount: i32) -> i32 {
        self.calculate_outgoing_damage_with_env(base_amount, None)
    }

    pub fn calculate_outgoing_damage_with_env(&self, base_amount: i32, environment: Option<&Environment>) -> i32 {
        let damage = if self.weakness > 0 {
            (base_amount as f32 * 0.75) as i32
        } else {
            base_amount
        };

        if let Some(env) = environment {
            (damage as f32 * env.damage_modifier) as i32
        } else {
            damage
        }
    }

    /// 计算实际受到的伤害 (考虑易伤)
    pub fn calculate_incoming_damage(&self, base_amount: i32) -> i32 {
        self.calculate_incoming_damage_with_env(base_amount, None)
    }

    pub fn calculate_incoming_damage_with_env(&self, base_amount: i32, _environment: Option<&Environment>) -> i32 {
        if self.vulnerable > 0 {
            (base_amount as f32 * 1.5) as i32
        } else {
            base_amount
        }
    }

    pub fn take_damage(&mut self, amount: i32) {
        self.take_damage_with_env(amount, None);
    }

    pub fn take_damage_with_env(&mut self, amount: i32, environment: Option<&Environment>) {
        let mut remaining_damage = self.calculate_incoming_damage_with_env(amount, environment);
        
        if self.block > 0 {
            if self.block >= remaining_damage {
                self.block -= remaining_damage;
                remaining_damage = 0;
            } else {
                remaining_damage -= self.block;
                self.block = 0;
            }
        }
        self.hp = (self.hp - remaining_damage).max(0);
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

        // BOSS 专属固定招式循环
        if self.enemy_type == EnemyType::GreatDemon {
            let cycle_pos = self.turn_count % 3;
            let intent = match cycle_pos {
                0 => {
                    // 啸天：基准大伤害
                    EnemyIntent::Attack { damage: 20 + self.strength }
                },
                1 => {
                    // 瞬狱杀：中等伤害
                    EnemyIntent::Attack { damage: 15 + self.strength }
                },
                _ => {
                    // 聚灵：大幅提升攻击力
                    EnemyIntent::Buff { strength: 5 }
                }
            };
            self.intent = intent;
            self.turn_count += 1;
            return;
        }

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
        } else if roll < self.ai_pattern.attack_chance + self.ai_pattern.defend_chance + self.ai_pattern.buff_chance {
            // 强化
            let strength = rng.gen_range(self.ai_pattern.buff_range.0..=self.ai_pattern.buff_range.1);
            EnemyIntent::Buff { strength }
        } else if roll < self.ai_pattern.attack_chance + self.ai_pattern.defend_chance + self.ai_pattern.buff_chance + self.ai_pattern.debuff_chance {
            // 减益
            EnemyIntent::Debuff { poison: 2, weakness: 1 }
        } else if roll < self.ai_pattern.attack_chance + self.ai_pattern.defend_chance + self.ai_pattern.buff_chance + self.ai_pattern.debuff_chance + self.ai_pattern.curse_chance {
            // 诅咒 (card_id 500 可能是某种诅咒卡)
            EnemyIntent::Curse { card_id: 500 }
        } else if roll < self.ai_pattern.attack_chance + self.ai_pattern.defend_chance + self.ai_pattern.buff_chance + self.ai_pattern.debuff_chance + self.ai_pattern.curse_chance + self.ai_pattern.seal_chance {
            // 封印
            let slot = rng.gen_range(0..5);
            EnemyIntent::Seal { slot_index: slot, duration: 2 }
        } else {
            // 默认回退到攻击
            EnemyIntent::Attack { damage: self.ai_pattern.damage_range.0 + self.strength }
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
            EnemyIntent::Debuff { poison, weakness } => {
                info!("{} 正在施加减益效果...", self.name);
                EnemyIntent::Debuff { poison, weakness }
            }
            EnemyIntent::Curse { card_id } => {
                info!("{} 正在向你的剑冢注入诅咒...", self.name);
                EnemyIntent::Curse { card_id }
            }
            EnemyIntent::Seal { slot_index, duration } => {
                info!("{} 封印了你的第 {} 个气穴！", self.name, slot_index + 1);
                EnemyIntent::Seal { slot_index, duration }
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

/// 战斗状态
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CombatState {
    /// 当前回合阶段
    pub phase: TurnPhase,
    /// 本回合是否已抽牌
    pub cards_drawn_this_turn: bool,
}

/// 天象打击演出资源
#[derive(Resource, Debug, Clone)]
pub struct HeavenlyStrikeCinematic {
    pub active: bool,
    pub timer: Timer,
    /// 记录待造成的伤害
    pub pending_damage: i32,
    /// 记录环境名称
    pub environment_name: String,
    /// 是否已结算伤害
    pub damage_applied: bool,
    /// 已触发的闪光次数
    pub flash_count: u32,
    /// 下一次落雷特效的计时器
    pub effect_timer: Timer,
}

impl Default for HeavenlyStrikeCinematic {
    fn default() -> Self {
        Self {
            active: false,
            // 总时长延长到 4.0 秒，确保降落完整
            timer: Timer::from_seconds(4.0, TimerMode::Once),
            pending_damage: 0,
            environment_name: "".to_string(),
            damage_applied: false,
            flash_count: 0,
            effect_timer: Timer::from_seconds(0.12, TimerMode::Repeating),
        }
    }
}

impl HeavenlyStrikeCinematic {
    pub fn start(&mut self, damage: i32, env_name: String) {
        self.active = true;
        self.timer.reset();
        self.pending_damage = damage;
        self.environment_name = env_name;
        self.damage_applied = false;
    }
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

/// 敌人行动序列资源（用于逐个行动）
#[derive(Resource, Debug, Clone)]
pub struct EnemyActionQueue {
    /// 待行动的敌人实体列表
    pub enemies: Vec<Entity>,
    /// 当前正在行动的索引
    pub current_index: usize,
    /// 动作之间的间隔计时器
    pub timer: Timer,
    /// 是否已经处理完所有动作
    pub processing: bool,
}

impl Default for EnemyActionQueue {
    fn default() -> Self {
        Self {
            enemies: Vec::new(),
            current_index: 0,
            timer: Timer::from_seconds(1.0, TimerMode::Once),
            processing: false,
        }
    }
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

/// 玩家持久化状态资源（跨场景保存）
#[derive(Resource, Debug, Clone)]
pub struct PlayerDeck {
    /// 牌组中的所有卡牌
    pub cards: Vec<Card>,
    /// 当前生命值
    pub hp: i32,
    /// 最大生命值
    pub max_hp: i32,
    /// 灵石数量
    pub gold: i32,
}

impl PlayerDeck {
    /// 创建新牌组并初始化数值
    pub fn new() -> Self {
        Self {
            hp: 80,
            max_hp: 80,
            gold: 100, // 初始灵石
            cards: crate::components::cards::create_starting_deck(), // 还原初始功法
        }
    }

    /// 从 Player 实体更新数据
    pub fn update_from_player(&mut self, player: &Player) {
        self.hp = player.hp;
        self.max_hp = player.max_hp;
        self.gold = player.gold;
    }

    /// 将数据应用到 Player 实体
    pub fn apply_to_player(&self, player: &mut Player) {
        player.hp = self.hp;
        player.max_hp = self.max_hp;
        player.gold = self.gold;
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
        *self = Self::new();
    }
}

impl Default for PlayerDeck {
    fn default() -> Self {
        Self::new()
    }
}

// 需要导入 Card 类型
use crate::components::cards::{Card, create_starting_deck};

// ============================================================================
// UI 悬停面板标记组件
// ============================================================================

/// 卡牌悬停详情面板标记
#[derive(Component)]
pub struct CardHoverPanelMarker;

/// 遗物悬停详情面板标记
#[derive(Component)]
pub struct RelicHoverPanelMarker;
