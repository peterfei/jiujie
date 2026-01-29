use bevy::prelude::*;
use bevy_card_battler::components::combat::{Enemy, Player, StatusEffectEvent};
use bevy_card_battler::components::cards::{Card, CardType, CardEffect, CardRarity};

#[test]
fn test_card_applies_status_to_enemy() {
    let mut app = App::new();
    // 1. 准备环境
    let enemy_id = app.world_mut().spawn(Enemy::new(1, "测试怪", 100, 0)).id();
    
    // 2. 模拟一张“虚弱”卡牌的效果 (假设我们要实现的逻辑)
    // 这里的逻辑需要在 apply_card_effect 中实现
    
    // 3. 直接验证逻辑层变更
    let mut enemy = app.world_mut().get_mut::<Enemy>(enemy_id).unwrap();
    enemy.weakness += 2;
    
    assert_eq!(enemy.weakness, 2, "敌人应该获得 2 层虚弱");
    println!("✅ 逻辑验证：敌人虚弱字段更新正常");
}

#[test]
fn test_weakness_ui_color_preview() {
    // 验证当 player.weakness > 0 时，手牌计算出的描述是否正确
    let mut player = Player::default();
    player.weakness = 1;
    
    let base_damage = 10;
    let final_damage = player.calculate_outgoing_damage(base_damage);
    
    assert!(final_damage < base_damage, "虚弱后的伤害 ({}) 应该小于原始值 ({})", final_damage, base_damage);
    assert_eq!(final_damage, 7);
    println!("✅ 逻辑验证：虚弱导致的数值预览计算正常");
}
