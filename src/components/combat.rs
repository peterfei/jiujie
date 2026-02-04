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
    /// 剑意值 (0-5)
    pub sword_intent: i32,
    /// 中毒层数 (每回合开始扣血)
    pub poison: i32,
    /// 灼烧层数 (每回合开始扣血，并随时间递减)
    pub burn: i32,
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
            sword_intent: 0,
            poison: 0, burn: 0, weakness: 0, vulnerable: 0,
        }
    }
}

impl Player {
    /// 积累剑意
    pub fn add_sword_intent(&mut self, amount: i32) {
        self.sword_intent = (self.sword_intent + amount).min(5);
    }

    /// 重置剑意
    pub fn reset_sword_intent(&mut self) {
        self.sword_intent = 0;
    }

    /// 获取当前剑意带来的额外伤害加成
    pub fn get_intent_damage_bonus(&self) -> i32 {
        match self.sword_intent {
            0..=2 => 0,
            3..=4 => 2,
            5 => 5, // 人剑合一
            _ => 0,
        }
    }

    /// 计算实际造成的伤害 (考虑虚弱和剑意)
    pub fn calculate_outgoing_damage(&self, base_amount: i32) -> i32 {
        self.calculate_outgoing_damage_with_env(base_amount, None)
    }

    pub fn calculate_outgoing_damage_with_env(&self, base_amount: i32, environment: Option<&Environment>) -> i32 {
        // 先应用基础伤害 + 剑意加成
        let total_base = base_amount + self.get_intent_damage_bonus();
        
        let damage = if self.weakness > 0 {
            (total_base as f32 * 0.75) as i32
        } else {
            total_base
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
pub struct SwordIntentText;

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
    pub poison: i32,
    /// 敌人词缀
    pub affixes: Vec<EnemyAffix>,
    /// [新增] 是否处于“蓄势”状态（下一次攻击伤害翻倍）
    pub is_charged: bool,
}

/// 敌人词缀
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnemyAffix {
    /// 精英: 全属性提升，体型变大，金色
    Elite,
    /// 虚弱: 属性降低，体型变小，灰色
    Weak,
    /// 狂暴: 攻击力大幅提升，防御降低，红色
    Berserk,
    /// 坚韧: 护甲提升，蓝色
    Tank,
    /// 迅捷: 闪避率提升（暂未实现逻辑，仅视觉），青色
    Swift,
    /// 火焰: 攻击施加灼烧，红色
    Fire,
    /// 剧毒: 攻击施加中毒，绿色
    Poison,
    /// 寒冰: 攻击施加虚弱，蓝色
    Ice,
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

/// AI模式配置 - 支持概率选择或固定序列
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
    /// [新增] 固定招式序列 (如果不为空，则优先按序列循环)
    pub sequence: Vec<EnemyIntent>,
    /// [新增] 当前招式进度
    pub current_step: usize,
}

impl AiPattern {
    pub fn new_random(
        attack: f32, defend: f32, buff: f32, 
        damage: (i32, i32), block: (i32, i32),
    ) -> Self {
        Self {
            attack_chance: attack,
            defend_chance: defend,
            buff_chance: buff,
            debuff_chance: 0.0,
            curse_chance: 0.0,
            seal_chance: 0.0,
            damage_range: damage,
            block_range: block,
            buff_range: (1, 3),
            sequence: Vec::new(),
            current_step: 0,
        }
    }

    pub fn demonic_wolf() -> Self {
        Self {
            attack_chance: 0.7, defend_chance: 0.1, buff_chance: 0.2, debuff_chance: 0.0,
            curse_chance: 0.0, seal_chance: 0.0,
            damage_range: (8, 12), block_range: (3, 5), buff_range: (1, 3),
            sequence: Vec::new(),
            current_step: 0,
        }
    }

    pub fn poison_spider() -> Self {
        Self {
            attack_chance: 0.3, defend_chance: 0.2, buff_chance: 0.0, debuff_chance: 0.3,
            curse_chance: 0.0, seal_chance: 0.2, 
            damage_range: (5, 8), block_range: (4, 6), buff_range: (0, 0),
            sequence: Vec::new(),
            current_step: 0,
        }
    }

    pub fn cursed_spirit() -> Self {
        Self {
            attack_chance: 0.2, defend_chance: 0.2, buff_chance: 0.0, debuff_chance: 0.2,
            curse_chance: 0.4, seal_chance: 0.0,
            damage_range: (10, 15), block_range: (5, 10), buff_range: (0, 0),
            sequence: Vec::new(),
            current_step: 0,
        }
    }

    pub fn great_demon() -> Self {
        // Boss 采用固定序列
        Self {
            attack_chance: 0.5, defend_chance: 0.2, buff_chance: 0.1, debuff_chance: 0.1,
            curse_chance: 0.05, seal_chance: 0.05,
            damage_range: (12, 18), block_range: (6, 10), buff_range: (3, 5),
            sequence: vec![
                EnemyIntent::Attack { damage: 15 },      // 1. 试探
                EnemyIntent::Defend { block: 12 },       // 2. 蓄势 (获得护甲)
                EnemyIntent::Attack { damage: 28 },      // 3. 破魔斩 (重击)
                EnemyIntent::Wait,                       // 4. 喘息
            ],
            current_step: 0,
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

    /// 获取下一步意图
    pub fn next_intent(&mut self, roll: f32, strength: i32) -> EnemyIntent {
        if !self.sequence.is_empty() {
            let mut intent = self.sequence[self.current_step].clone();
            // 应用当前的攻击力加成
            if let EnemyIntent::Attack { ref mut damage } = intent {
                *damage += strength;
            }
            self.current_step = (self.current_step + 1) % self.sequence.len();
            intent
        } else {
            // 原有的概率逻辑
            if roll < self.attack_chance {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let base_damage = rng.gen_range(self.damage_range.0..=self.damage_range.1);
                EnemyIntent::Attack { damage: base_damage + strength }
            } else if roll < self.attack_chance + self.defend_chance {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let block = rng.gen_range(self.block_range.0..=self.block_range.1);
                EnemyIntent::Defend { block }
            } else if roll < self.attack_chance + self.defend_chance + self.buff_chance {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let strength_gain = rng.gen_range(self.buff_range.0..=self.buff_range.1);
                EnemyIntent::Buff { strength: strength_gain }
            } else if roll < self.attack_chance + self.defend_chance + self.buff_chance + self.debuff_chance {
                EnemyIntent::Debuff { poison: 2, weakness: 1 }
            } else if roll < self.attack_chance + self.defend_chance + self.buff_chance + self.debuff_chance + self.curse_chance {
                EnemyIntent::Curse { card_id: 500 }
            } else if roll < self.attack_chance + self.defend_chance + self.buff_chance + self.debuff_chance + self.curse_chance + self.seal_chance {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let slot = rng.gen_range(0..5);
                EnemyIntent::Seal { slot_index: slot, duration: 2 }
            } else {
                EnemyIntent::Attack { damage: self.damage_range.0 + strength }
            }
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
            affixes: Vec::new(),
            is_charged: false,
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
            affixes: Vec::new(),
            is_charged: false,
        }
    }

    /// 计算实际造成的伤害 (考虑虚弱)
    pub fn calculate_outgoing_damage(&self, base_amount: i32) -> i32 {
        self.calculate_outgoing_damage_with_env(base_amount, None)
    }

    pub fn calculate_outgoing_damage_with_env(&self, base_amount: i32, environment: Option<&Environment>) -> i32 {
        let mut damage = if self.weakness > 0 {
            (base_amount as f32 * 0.75) as i32
        } else {
            base_amount
        };

        // 应用蓄势加成 (翻倍)
        if self.is_charged {
            damage *= 2;
        }

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

    /// 消耗“蓄势”状态
    pub fn consume_charge(&mut self) {
        if self.is_charged {
            self.is_charged = false;
            info!("✨ {} 的蓄势劲力已倾泻而出", self.name);
        }
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

        // 如果是二阶段 Boss 且血量过低，切换至狂暴序列
        if self.enemy_type == EnemyType::GreatDemon && self.hp < self.max_hp / 2 {
            // 检查是否已经切换过序列 (通过序列第一个招式的伤害值来判断，或者检查长度)
            let is_already_rage = self.ai_pattern.sequence.len() == 3;
            if !is_already_rage {
                 self.ai_pattern.sequence = vec![
                    EnemyIntent::Attack { damage: 35 }, 
                    EnemyIntent::Buff { strength: 8 },                 
                    EnemyIntent::Attack { damage: 25 }, 
                 ];
                 self.ai_pattern.current_step = 0;
                 info!("🔥 {} 进入了【狂暴二阶段】！", self.name);
            }
        }

        self.intent = self.ai_pattern.next_intent(rng.gen(), self.strength);
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
                // 如果是大妖 (Boss)，防御即蓄势
                if self.enemy_type == EnemyType::GreatDemon {
                    self.is_charged = true;
                    info!("🛡️ {} 正在蓄势，其势待发！", self.name);
                }
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

    /// 应用攻击附带的词缀效果
    pub fn apply_attack_affixes(&self, player: &mut Player) {
        for affix in &self.affixes {
            match affix {
                EnemyAffix::Fire => player.burn += 3, 
                EnemyAffix::Poison => player.poison += 2, 
                EnemyAffix::Ice => player.weakness += 1, 
                _ => {} 
            }
        }
    }
}

/// 天象环境UI面板标记
#[derive(Component)]
pub struct EnvironmentPanel;

/// 天象环境文本标记
#[derive(Component)]
pub struct EnvironmentText;

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

impl Default for CombatState {
    fn default() -> Self {
        Self {
            phase: TurnPhase::PlayerStart,
            cards_drawn_this_turn: false,
        }
    }
}

// ============================================================================
// 敌人组件
// ============================================================================


// ============================================================================
// UI 悬停面板标记组件
// ============================================================================

/// 卡牌悬停详情面板标记
#[derive(Component)]
pub struct CardHoverPanelMarker;

/// 遗物悬停详情面板标记
#[derive(Component)]
pub struct RelicHoverPanelMarker;
