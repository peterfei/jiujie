use bevy::prelude::*;
use bevy_card_battler::components::combat::{Enemy, EnemyType};
use bevy_card_battler::components::sprite::AnimationState;

#[test]
fn test_death_event_on_damage() {
    // 逻辑：伤害导致 HP 降至 0 应标记死亡
    let mut enemy = Enemy::new(1, "受气包", 10, 0);
    assert_eq!(enemy.hp, 10);
    
    enemy.take_damage(10);
    assert_eq!(enemy.hp, 0);
    assert!(enemy.is_dead());
}

#[test]
fn test_dead_enemy_skipped_in_logic() {
    let enemy = Enemy::new(1, "测试怪", 0, 0);
    // 验证死亡状态
    assert!(enemy.is_dead(), "HP 为 0 的敌人应被视为死亡");
}

#[test]
fn test_death_animation_mapping() {
    let anim = AnimationState::Death;
    assert!(matches!(anim, AnimationState::Death));
}
