use bevy_card_battler::components::combat::{Player, Enemy, EnemyIntent};
use bevy_card_battler::components::cards::{Card, CardType, CardEffect};
use rand::seq::SliceRandom;
use rand::thread_rng;

// --- 模拟器基础结构 ---

#[derive(Clone)]
struct SimState {
    player: Player,
    enemy: Enemy,
    hand: Vec<Card>,
    draw_pile: Vec<Card>,
    discard_pile: Vec<Card>,
    turn: u32,
    log: Vec<String>,
}

impl SimState {
    fn new(enemy_hp: i32, enemy_damage: i32) -> Self {
        let mut enemy = Enemy::new(1, "模拟怪", enemy_hp, 0);
        enemy.intent = EnemyIntent::Attack { damage: enemy_damage };
        Self {
            player: Player { hp: 100, max_hp: 100, energy: 3, max_energy: 3, ..Default::default() },
            enemy,
            hand: vec![],
            draw_pile: create_starter_deck(),
            discard_pile: vec![],
            turn: 1,
            log: vec![],
        }
    }

    fn run_turn(&mut self) {
        // 1. 抽牌 (模拟每回合抽 5 张)
        for _ in 0..5 {
            if self.draw_pile.is_empty() {
                self.draw_pile = self.discard_pile.clone();
                self.draw_pile.shuffle(&mut thread_rng());
                self.discard_pile.clear();
            }
            if let Some(card) = self.draw_pile.pop() {
                self.hand.push(card);
            }
        }

        // 2. 玩家出牌策略 (简单的贪婪策略：能打就打，优先攻击)
        // 排序：攻击牌优先
        self.hand.sort_by(|a, b| {
            if a.card_type == CardType::Attack && b.card_type != CardType::Attack { std::cmp::Ordering::Less }
            else { std::cmp::Ordering::Equal }
        });

        let mut energy = self.player.energy;
        let mut played_indices = vec![];

        for (i, card) in self.hand.iter().enumerate() {
            if card.cost <= energy {
                energy -= card.cost;
                played_indices.push(i);
                
                // 结算效果
                match card.effect {
                    CardEffect::DealDamage { amount } => {
                        let dmg = self.player.calculate_outgoing_damage(amount);
                        self.enemy.take_damage(dmg);
                    }
                    CardEffect::GainBlock { amount } => {
                        self.player.block += amount;
                    }
                    _ => {}
                }
            }
        }

        // 移除非手牌 (模拟 discard)
        // [关键修复] 必须先将手牌移入弃牌堆，否则两回合后就没牌了
        self.discard_pile.extend(self.hand.drain(..));
        
        self.player.energy = 3; // 重置能量
    }

    fn enemy_turn(&mut self) {
        if self.enemy.hp <= 0 { return; }
        
        match self.enemy.intent {
            EnemyIntent::Attack { damage } => {
                let dmg = self.enemy.calculate_outgoing_damage(damage);
                self.player.take_damage(dmg);
            }
            _ => {}
        }
    }
}

fn create_starter_deck() -> Vec<Card> {
    let mut deck = vec![];
    // 5 攻 5 防
    for _ in 0..5 {
        deck.push(Card { 
            id: 0,
            name: "攻击".to_string(), 
            cost: 1, 
            card_type: CardType::Attack, 
            effect: CardEffect::DealDamage { amount: 6 },
            description: "造成6点伤害".to_string(),
            image_path: "".to_string(),
            rarity: bevy_card_battler::components::cards::CardRarity::Common,
            upgraded: false,
        });
    }
    for _ in 0..5 {
        deck.push(Card { 
            id: 0,
            name: "防御".to_string(), 
            cost: 1, 
            card_type: CardType::Skill, 
            effect: CardEffect::GainBlock { amount: 5 },
            description: "获得5点护甲".to_string(),
            image_path: "".to_string(),
            rarity: bevy_card_battler::components::cards::CardRarity::Common,
            upgraded: false,
        });
    }
    deck
}

// --- TDD 测试用例 ---

#[test]
fn tdd_balance_spider_fight() {
    // 模拟战：基础毒蛛 (40 HP, 6 攻)
    let mut wins = 0;
    let mut total_hp_loss = 0;
    let runs = 100;

    for _ in 0..runs {
        let mut state = SimState::new(40, 6);
        for _turn in 0..20 { // 最多 20 回合
            state.run_turn();
            if state.enemy.hp <= 0 {
                wins += 1;
                total_hp_loss += (100 - state.player.hp);
                break;
            }
            state.enemy_turn();
            if state.player.hp <= 0 { break; }
        }
    }

    let avg_loss = total_hp_loss as f32 / wins as f32;
    println!("【平衡报告 - 毒蛛】胜率: {}%, 平均耗血: {:.1}", (wins as f32 / runs as f32) * 100.0, avg_loss);

    // 验证标准：简单小怪不应造成太大威胁
    assert!(wins == runs, "初始卡组打小怪胜率必须 100%");
    assert!(avg_loss < 15.0, "打小怪平均耗血不应超过 15 (当前: {:.1})", avg_loss);
}

#[test]
fn tdd_balance_wolf_fight() {
    // 模拟战：魔狼 (60 HP, 8 攻)
    let mut wins = 0;
    let mut total_hp_loss = 0;
    let runs = 100;

    for _ in 0..runs {
        let mut state = SimState::new(60, 8);
        for _turn in 0..20 {
            state.run_turn();
            if state.enemy.hp <= 0 {
                wins += 1;
                total_hp_loss += (100 - state.player.hp);
                break;
            }
            state.enemy_turn();
            if state.player.hp <= 0 { break; }
        }
    }

    let avg_loss = total_hp_loss as f32 / wins as f32;
    println!("【平衡报告 - 魔狼】胜率: {}%, 平均耗血: {:.1}", (wins as f32 / runs as f32) * 100.0, avg_loss);

    assert!(wins == runs, "魔狼虽然强，但初始卡组也应能过");
    assert!(avg_loss < 30.0, "打魔狼平均耗血不应超过 30 (当前: {:.1})", avg_loss);
}
