//! 战斗连招 (剑意涌动) TDD 测试

use bevy::prelude::*;
use bevy_card_battler::components::*;

mod test_utils;
use test_utils::*;

#[test]
fn test_sword_intent_accumulation() {
    let mut app = create_test_app();
    setup_combat_scene(&mut app);
    
    // 1. 检查初始剑意
    {
        let player = app.world().get_resource::<Player>().expect("找不到玩家资源");
        assert_eq!(player.sword_intent, 0);
    }
    
    // 2. 模拟打出 3 张攻击卡
    {
        let mut player = app.world_mut().get_resource_mut::<Player>().unwrap();
        player.add_sword_intent(1);
        player.add_sword_intent(1);
        player.add_sword_intent(1);
        
        assert_eq!(player.sword_intent, 3);
        // 验证伤害加成 (3层剑意应为 +2)
        assert_eq!(player.get_intent_damage_bonus(), 2);
    }
    
    // 3. 验证伤害计算是否包含剑意加成
    {
        let player = app.world().get_resource::<Player>().unwrap();
        let final_damage = player.calculate_outgoing_damage(10);
        // 10 基础 + 2 剑意 = 12
        assert_eq!(final_damage, 12);
    }
    
    // 4. 模拟打出防御卡，重置剑意
    {
        let mut player = app.world_mut().get_resource_mut::<Player>().unwrap();
        player.reset_sword_intent();
        assert_eq!(player.sword_intent, 0);
        assert_eq!(player.get_intent_damage_bonus(), 0);
    }
}

#[test]
fn test_sword_intent_tiers() {
    let mut player = Player::default();
    
    // Tier 0: 0-2 (Bonus 0)
    player.sword_intent = 2;
    assert_eq!(player.get_intent_damage_bonus(), 0);
    
    // Tier 1: 3-4 (Bonus 2)
    player.sword_intent = 3;
    assert_eq!(player.get_intent_damage_bonus(), 2);
    player.sword_intent = 4;
    assert_eq!(player.get_intent_damage_bonus(), 2);
    
    // Tier 2: 5 (Bonus 5)
    player.sword_intent = 5;
    assert_eq!(player.get_intent_damage_bonus(), 5);
}

#[test]
fn test_sword_intent_max_tier() {
    let mut app = create_test_app();
    setup_combat_scene(&mut app);
    
    {
        let mut player = app.world_mut().get_resource_mut::<Player>().unwrap();
        
        // 强行充满剑意 (5层)
        player.add_sword_intent(10); // 应被限制在 5
        assert_eq!(player.sword_intent, 5);
        
        // 验证巅峰加成 (+5)
        assert_eq!(player.get_intent_damage_bonus(), 5);
        
        let final_damage = player.calculate_outgoing_damage(10);
        // 10 基础 + 5 剑意 = 15
        assert_eq!(final_damage, 15);
    }
}