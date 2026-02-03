//! 境界突破 (筑基) 集成测试

use bevy::prelude::*;
use bevy::ecs::system::RunSystemOnce;
use bevy_card_battler::states::GameState;
use bevy_card_battler::components::*;
use bevy_card_battler::components::relic::{Relic, RelicCollection, RelicId};

mod test_utils;
use test_utils::*;

#[test]
fn test_foundation_establishment_breakthrough() {
    let mut app = create_test_app();
    
    // 1. 准备：手动创建一个玩家实体并赋予圆满感悟
    {
        let mut world = app.world_mut();
        world.spawn((
            Player::default(),
            Cultivation {
                realm: bevy_card_battler::components::cultivation::Realm::QiRefining,
                insight: 50,
            },
        ));
        
        // 同时更新资源（系统可能读取资源）
        if let Some(mut cultivation_res) = world.get_resource_mut::<Cultivation>() {
            cultivation_res.insight = 50;
        }
    }
    
    // 2. 模拟进入渡劫
    info!("--- 阶段 1: 引动雷劫 ---");
    transition_to_state(&mut app, GameState::Tribulation);
    advance_frames(&mut app, 5);
    
    // 手动运行一次 teardown_tribulation 以验证逻辑
    info!("--- 阶段 2: 触发破境逻辑 ---");
    let _ = app.world_mut().run_system_once(bevy_card_battler::plugins::teardown_tribulation_wrapper);
    
    // 3. 验证属性飞跃
    {
        let mut world = app.world_mut();
        let mut query = world.query::<(&Player, &Cultivation)>();
        let (player, cultivation) = query.iter(world).next().expect("找不到玩家实体");
        
        let deck = world.get_resource::<PlayerDeck>().unwrap();
        
        info!("破境后状态: 境界={:?}, HP={}/{}, 灵石={}, 功法数={}, 法宝位={}", 
            cultivation.realm, player.hp, player.max_hp, player.gold, deck.cards.len(), cultivation.get_relic_slots());
            
        // A. 境界变更为筑基
        assert_eq!(cultivation.realm, bevy_card_battler::components::cultivation::Realm::FoundationEstablishment);
        
        // B. HP 上限增加 (默认 80 + 50 = 130)
        assert_eq!(player.max_hp, 130);
        assert_eq!(player.hp, 130);
        
        // C. 获得本命功法：青莲剑歌
        assert!(deck.cards.iter().any(|c| c.name == "青莲剑歌"));
        
        // D. 法宝位解锁为 2
        assert_eq!(cultivation.get_relic_slots(), 2);
        
        // E. 测试法宝位限制 (使用确定的唯一ID，避免随机冲突)
        let mut collection = RelicCollection::default();
        let r1 = Relic { id: RelicId::Custom(1), name: "法宝1".into(), description: "".into(), rarity: bevy_card_battler::components::relic::RelicRarity::Common, effects: vec![] };
        let r2 = Relic { id: RelicId::Custom(2), name: "法宝2".into(), description: "".into(), rarity: bevy_card_battler::components::relic::RelicRarity::Common, effects: vec![] };
        let r3 = Relic { id: RelicId::Custom(3), name: "法宝3".into(), description: "".into(), rarity: bevy_card_battler::components::relic::RelicRarity::Common, effects: vec![] };
        
        assert!(collection.add_relic(r1, cultivation), "应该允许添加第 1 个法宝");
        assert!(collection.add_relic(r2, cultivation), "应该允许添加第 2 个法宝");
        assert!(!collection.add_relic(r3, cultivation), "应该拒绝添加第 3 个法宝 (筑基上限为 2)");
    }
    
    info!("✅ 筑基期境界突破验证全项通过！");
}
