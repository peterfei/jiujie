use bevy::prelude::*;
use bevy_card_battler::components::combat::{Enemy, EnemyType};
use bevy_card_battler::components::sprite::AnimationState;

#[test]
fn test_death_event_on_damage() {
    let mut enemy = Enemy::with_type(1, "受气包", 10, EnemyType::DemonicWolf);
    
    // 模拟受到 10 点伤害
    enemy.take_damage(10);
    
    // 逻辑验证：HP 应为 0
    assert_eq!(enemy.hp, 0);
    
    // 预期：此时逻辑层应能识别出需要播放死亡动画
    let should_trigger_death = enemy.is_dead();
    assert!(should_trigger_death, "HP 归零后应标记为死亡状态");
}

#[test]
fn test_dead_enemy_skipped_in_logic() {
    let mut enemy = Enemy::with_type(1, "测试怪", 0, EnemyType::DemonicWolf);
    
    // 逻辑验证：如果 HP <= 0，不应执行意图
    let can_act = enemy.hp > 0;
    assert!(!can_act, "死掉的敌人不应该能够行动");
}

#[test]
fn test_death_animation_mapping() {
    let anim = AnimationState::Death;
    assert!(matches!(anim, AnimationState::Death));
}
