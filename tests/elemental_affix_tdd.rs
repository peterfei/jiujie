pub mod test_utils;
use bevy::prelude::*;
use bevy_card_battler::components::combat::{Enemy, EnemyAffix, Player};

#[test]
fn test_fire_affix_applies_burn() {
    let mut enemy = Enemy::new(1, "FireWolf", 100);
    enemy.affixes.push(EnemyAffix::Fire);
    
    let mut player = Player::default();
    assert_eq!(player.burn, 0);

    // 模拟攻击附带效果
    enemy.apply_attack_affixes(&mut player);

    assert_eq!(player.burn, 3, "Fire affix should apply 3 burn stacks");
}

#[test]
fn test_poison_affix_applies_poison() {
    let mut enemy = Enemy::new(1, "PoisonSpider", 100);
    enemy.affixes.push(EnemyAffix::Poison);
    
    let mut player = Player::default();
    
    enemy.apply_attack_affixes(&mut player);

    assert_eq!(player.poison, 2, "Poison affix should apply 2 poison stacks");
}

#[test]
fn test_ice_affix_applies_weakness() {
    let mut enemy = Enemy::new(1, "IceSpirit", 100);
    enemy.affixes.push(EnemyAffix::Ice);
    
    let mut player = Player::default();
    
    enemy.apply_attack_affixes(&mut player);

    assert_eq!(player.weakness, 1, "Ice affix should apply 1 weakness stack");
}

#[test]
fn test_mixed_affixes() {
    let mut enemy = Enemy::new(1, "EliteBoss", 500);
    enemy.affixes.push(EnemyAffix::Fire);
    enemy.affixes.push(EnemyAffix::Ice);
    
    let mut player = Player::default();
    
    enemy.apply_attack_affixes(&mut player);

    assert_eq!(player.burn, 3);
    assert_eq!(player.weakness, 1);
}
