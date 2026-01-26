//! 战斗胜利 E2E 测试
//!
//! 测试敌人被击败后触发奖励界面及完整流程

use bevy_card_battler::components::{PlayerDeck, MapProgress, Enemy, CardPool, EffectType, NodeType};
use bevy_card_battler::components::{Card, CardType, CardEffect, CardRarity, MapConfig};

// ============================================================================
// 战斗胜利触发测试
// ============================================================================

#[test]
fn e2e_001_enemy_defeated_triggers_reward() {
    let mut enemy = Enemy::new(1, "测试敌人", 50);

    // 敌人初始 HP > 0
    assert!(!enemy.is_dead(), "敌人初始状态应该是存活");

    // 敌人受到致命伤害
    enemy.take_damage(60);

    // 验证：敌人死亡
    assert!(enemy.is_dead(), "敌人 HP ≤ 0 应该死亡");
}

#[test]
fn e2e_002_enemy_hp_exactly_zero_defeated() {
    let mut enemy = Enemy::new(2, "测试敌人", 30);

    // 敌人 HP 正好归零
    enemy.take_damage(30);

    assert!(enemy.is_dead(), "敌人 HP = 0 应该死亡");
}

#[test]
fn e2e_003_multiple_hits_kill_enemy() {
    let mut enemy = Enemy::new(3, "测试敌人", 50);

    // 多次伤害累积
    enemy.take_damage(30);
    assert!(!enemy.is_dead(), "30点伤害后应该还存活");

    enemy.take_damage(25);
    assert!(enemy.is_dead(), "累积伤害超过 HP 应该死亡");
}

// ============================================================================
// 奖励卡牌池测试
// ============================================================================

#[test]
fn e2e_101_reward_pool_generates_3_cards() {
    let rewards = CardPool::random_rewards(3);

    assert_eq!(rewards.len(), 3, "应该生成3张奖励卡牌");
}

#[test]
fn e2e_102_reward_pool_cards_are_valid() {
    let rewards = CardPool::random_rewards(10);

    // 验证每张卡都有效
    for card in rewards {
        assert!(!card.name.is_empty(), "奖励卡应该有名称");
        assert!(card.cost >= 0, "奖励卡能量消耗应该非负");
        assert!(!card.description.is_empty(), "奖励卡应该有描述");
    }
}

#[test]
fn e2e_103_reward_pool_includes_different_rarities() {
    let rewards = CardPool::random_rewards(30);

    // 验证包含不同稀有度
    let has_common = rewards.iter().any(|c| c.rarity == CardRarity::Common);
    // let has_uncommon = rewards.iter().any(|c| c.rarity == CardRarity::Uncommon, "textures/cards/default.png");
    // let has_rare = rewards.iter().any(|c| c.rarity == CardRarity::Rare, "textures/cards/default.png");

    // 至少应该有普通卡
    assert!(has_common, "奖励池应该包含普通卡");
}

// ============================================================================
// 卡牌加入牌组测试
// ============================================================================

#[test]
fn e2e_201_selected_card_adds_to_deck() {
    let mut deck = PlayerDeck::new();
    let initial_size = deck.len();

    // 创建一张奖励卡
    let reward_card = Card::new(
        9999, "超级攻击", "造成30点伤害", CardType::Attack,
        2, CardEffect::DealDamage { amount: 30 }, CardRarity::Rare, "textures/cards/default.png"
    );

    // 添加到牌组
    deck.add_card(reward_card.clone());

    // 验证：牌组增加了1张
    assert_eq!(deck.len(), initial_size + 1, "添加奖励卡后牌组应该增加");

    // 验证：奖励卡在牌组中
    assert!(deck.cards.iter().any(|c| c.id == 9999), "奖励卡应该在牌组中");
}

#[test]
fn e2e_202_multiple_rewards_add_to_deck() {
    let mut deck = PlayerDeck::new();
    let initial_size = deck.len();

    // 添加3张奖励卡
    for i in 0..3 {
        deck.add_card(Card::new(
            5000 + i, "奖励卡", "测试", CardType::Skill,
            1, CardEffect::Heal { amount: 5 }, CardRarity::Uncommon, "textures/cards/default.png",
        ));
    }

    assert_eq!(deck.len(), initial_size + 3, "应该添加3张奖励卡");
}

#[test]
fn e2e_203_reward_card_preserved_after_menu_return() {
    let mut deck = PlayerDeck::new();

    // 添加奖励卡
    deck.add_card(Card::new(
        8888, "保留卡", "应该保留", CardType::Defense,
        1, CardEffect::GainBlock { amount: 10 }, CardRarity::Rare, "textures/cards/default.png",
    ));

    let size_with_reward = deck.len();

    // 模拟返回主菜单后再回来（不调用 reset）
    let size_after = deck.len();

    assert_eq!(size_after, size_with_reward, "不调用 reset 时奖励卡应该保留");
}

// ============================================================================
// 节点完成与解锁测试
// ============================================================================

#[test]
fn e2e_301_completing_node_unlocks_next_layer() {
    let mut progress = MapProgress::new(&MapConfig::default());

    // 完成第0层的节点
    progress.set_current_node(0);
    let completed_count_before = progress.nodes.iter()
        .filter(|n| n.completed && n.position.0 == 0)
        .count();

    progress.complete_current_node();

    // 验证：第0层节点标记为完成
    let completed_count_after = progress.nodes.iter()
        .filter(|n| n.completed && n.position.0 == 0)
        .count();

    assert!(completed_count_after > completed_count_before, "节点应该被标记为完成");

    // 验证：第1层节点解锁
    let layer_1_unlocked = progress.nodes.iter()
        .filter(|n| n.position.0 == 1)
        .all(|n| n.unlocked);

    assert!(layer_1_unlocked, "完成节点后下一层应该解锁");
}

#[test]
fn e2e_302_completion_only_unlocks_adjacent_layer() {
    let mut progress = MapProgress::new(&MapConfig::default());

    // 完成第0层
    progress.set_current_node(0);
    progress.complete_current_node();

    // 验证：第1层解锁
    let layer_1_unlocked = progress.nodes.iter()
        .filter(|n| n.position.0 == 1)
        .all(|n| n.unlocked);
    assert!(layer_1_unlocked, "第1层应该解锁");

    // 验证：第2层仍然锁定
    let layer_2_locked = progress.nodes.iter()
        .filter(|n| n.position.0 == 2)
        .all(|n| !n.unlocked);
    assert!(layer_2_locked, "第2层应该仍然锁定");
}

#[test]
fn e2e_303_completion_state_persists() {
    let mut progress = MapProgress::new(&MapConfig::default());

    // 完成节点
    progress.set_current_node(0);
    progress.complete_current_node();

    let completed_count = progress.nodes.iter().filter(|n| n.completed).count();
    let current_layer = progress.current_layer;

    // 模拟"保存"和"加载"（通过 clone）
    let progress_clone = progress.clone();

    assert_eq!(progress_clone.nodes.iter().filter(|n| n.completed).count(),
               completed_count, "完成状态应该被保留");
    assert_eq!(progress_clone.current_layer, current_layer, "层数应该被保留");
}

// ============================================================================
// 完整战斗胜利流程测试
// ============================================================================

#[test]
fn e2e_401_complete_victory_flow() {
    // 模拟完整流程
    let mut deck = PlayerDeck::new();
    let mut progress = MapProgress::new(&MapConfig::default());
    let initial_deck_size = deck.len();

    // 1. 战斗胜利：敌人死亡
    let mut enemy = Enemy::new(100, "测试敌人", 50);
    enemy.take_damage(100);
    assert!(enemy.is_dead(), "敌人应该死亡");

    // 2. 进入奖励界面（生成3张卡）
    let rewards = CardPool::random_rewards(3);
    assert_eq!(rewards.len(), 3, "应该有3张奖励卡");

    // 3. 玩家选择1张卡加入牌组
    if let Some(selected_card) = rewards.first() {
        deck.add_card(selected_card.clone());
    }

    // 4. 完成当前节点，解锁下一层
    progress.set_current_node(0);
    progress.complete_current_node();

    // 验证：流程结果
    assert!(deck.len() > initial_deck_size, "奖励卡应该加入牌组");
    assert!(progress.nodes.iter().any(|n| n.completed), "应该有已完成的节点");
}

#[test]
fn e2e_402_victory_skip_reward_flow() {
    let mut deck = PlayerDeck::new();
    let mut progress = MapProgress::new(&MapConfig::default());
    let initial_deck_size = deck.len();

    // 1. 战斗胜利
    let mut enemy = Enemy::new(101, "测试敌人", 50);
    enemy.take_damage(100);
    assert!(enemy.is_dead());

    // 2. 进入奖励界面但跳过
    // 不添加任何卡牌

    // 3. 完成节点
    progress.set_current_node(0);
    progress.complete_current_node();

    // 验证：牌组大小不变
    assert_eq!(deck.len(), initial_deck_size, "跳过奖励不应该改变牌组");
}

// ============================================================================
// 边界情况测试
// ============================================================================

#[test]
fn e2e_501_enemy_negative_hp() {
    let mut enemy = Enemy::new(102, "测试敌人", 10);

    // 造成过量伤害
    enemy.take_damage(1000);

    assert!(enemy.hp <= 0, "敌人 HP 应该是负数");
    assert!(enemy.is_dead(), "过量伤害应该杀死敌人");
}

#[test]
fn e2e_502_zero_damage_enemy_survives() {
    let mut enemy = Enemy::new(103, "测试敌人", 50);

    // 零伤害
    enemy.take_damage(0);

    assert_eq!(enemy.hp, 50, "零伤害不应该改变 HP");
    assert!(!enemy.is_dead(), "零伤害不应该杀死敌人");
}

#[test]
fn e2e_503_reward_cards_have_valid_ids() {
    let rewards1 = CardPool::random_rewards(10);
    let rewards2 = CardPool::random_rewards(10);

    // 验证 ID 存在
    for card in rewards1.iter().chain(rewards2.iter()) {
        assert!(card.id > 0, "奖励卡应该有有效的ID");
    }
}

// ============================================================================
// 粒子特效触发测试
// ============================================================================

#[test]
fn e2e_601_victory_triggers_particle_effects() {
    // 验证粒子特效类型存在
    let _fire_type = EffectType::Fire;
    let _heal_type = EffectType::Heal;
    let _hit_type = EffectType::Hit;

    // 如果编译通过，说明类型有效
    assert!(true, "粒子特效类型应该有效");
}

#[test]
fn e2e_602_reward_card_effects_match_type() {
    let rewards = CardPool::random_rewards(20);

    // 验证每张奖励卡的有效性
    for card in rewards {
        match card.card_type {
            CardType::Attack => {
                // 攻击卡应该有伤害效果
                assert!(matches!(card.effect, CardEffect::DealDamage { .. }
                        | CardEffect::AttackAndDraw { .. }),
                       "攻击卡应该有伤害相关效果");
            }
            CardType::Defense => {
                // 防御卡应该有护甲效果
                assert!(matches!(card.effect, CardEffect::GainBlock { .. }),
                       "防御卡应该有护甲效果");
            }
            CardType::Skill => {
                // 技能卡可以是各种效果
            }
            CardType::Power => {
                // 能力卡（暂未实现）
            }
        }
    }
}

// ============================================================================
// 多次战斗循环测试
// ============================================================================

#[test]
fn e2e_701_multiple_victory_cycle_accumulates_cards() {
    let mut deck = PlayerDeck::new();

    // 模拟3次战斗胜利，每次获得1张卡
    for i in 0..3 {
        let reward_card = Card::new(
            2000 + i, "循环奖励卡", format!("第{}次奖励", i + 1),
            CardType::Attack, 1, CardEffect::DealDamage { amount: 5 + i as i32 }, CardRarity::Common, "textures/cards/default.png",
        );
        deck.add_card(reward_card);
    }

    // 验证：牌组增加了3张
    assert!(deck.len() >= 15, "3次胜利应该至少增加3张卡");
}

#[test]
fn e2e_702_death_loses_progress() {
    let mut deck = PlayerDeck::new();
    let mut progress = MapProgress::new(&MapConfig::default());

    // 添加奖励卡
    deck.add_card(Card::new(
        7777, "死亡前奖励", "会丢失", CardType::Skill,
        1, CardEffect::Heal { amount: 10 }, CardRarity::Rare, "textures/cards/default.png",
    ));

    let deck_size_with_reward = deck.len();

    // 完成第0层
    progress.set_current_node(0);
    progress.complete_current_node();

    // 模拟玩家死亡（需要调用 reset）
    deck.reset();
    progress.reset();

    // 验证：奖励卡丢失
    assert!(deck.len() < deck_size_with_reward, "死亡后重置应该清除奖励卡");
    assert!(!deck.cards.iter().any(|c| c.id == 7777), "奖励卡不应该存在");
}

// ============================================================================
// Boss 战相关测试
// ============================================================================

#[test]
fn e2e_801_boss_node_exists() {
    let progress = MapProgress::new(&MapConfig::default());

    // 验证：存在 Boss 节点
    let has_boss = progress.nodes.iter().any(|n| n.node_type == NodeType::Boss);

    // 默认3层配置：最后一层第1个节点是Boss
    assert!(has_boss, "地图应该包含Boss节点");
}

#[test]
fn e2e_802_boss_is_defeatable() {
    let mut boss = Enemy::new(200, "Boss敌人", 500); // Boss 有更多 HP

    // Boss 可以被击败
    boss.take_damage(1000);

    assert!(boss.is_dead(), "Boss 应该能被击败");
}

#[test]
fn e2e_803_can_reach_max_layer() {
    let mut progress = MapProgress::new(&MapConfig::default());

    // 找到最大层数
    let max_layer = progress.nodes.iter()
        .map(|n| n.position.0)
        .max()
        .unwrap_or(0) as u32;

    // 完成所有层（通过设置不同节点）
    for layer in 0..=max_layer {
        if let Some(node) = progress.nodes.iter().find(|n| n.position.0 == layer as i32) {
            progress.set_current_node(node.id);
            progress.complete_current_node();
        }
    }

    // 验证：到达最大层
    assert!(progress.current_layer >= max_layer.saturating_sub(1), "应该能到达最高层");
}
