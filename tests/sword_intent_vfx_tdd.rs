//! 剑意爆发视觉效果 TDD 测试

use bevy::prelude::*;
use bevy_card_battler::components::*;
use bevy_card_battler::components::screen_effect::ScreenEffectEvent;
use bevy_card_battler::components::particle::SpawnEffectEvent;
use bevy_card_battler::plugins::HandCard;

mod test_utils;
use test_utils::*;

#[test]
fn test_sword_intent_burst_vfx() {
    let mut app = create_test_app();
    setup_combat_scene(&mut app);
    
    // 运行几帧让手牌系统生成实体
    advance_frames(&mut app, 10);
    
    // 1. 设置玩家为满剑意状态 (5层)
    {
        let mut player = app.world_mut().get_resource_mut::<Player>().unwrap();
        player.sword_intent = 5;
    }
    
    // 2. 模拟打出一张攻击卡
    {
        let world = app.world_mut();
        let mut card_query = world.query::<(Entity, &HandCard)>();
        let (card_entity, _) = card_query.iter(world).next().expect("找不到手牌实体");
        world.entity_mut(card_entity).insert(Interaction::Pressed);
    }
    
    // 3. 运行逻辑处理 (多运行几帧确保系统链执行完毕)
    app.update();
    app.update();
    
    // 4. 验证是否触发了爆发特效
    {
        let flash_events = app.world().resource::<Events<ScreenEffectEvent>>();
        let mut found_burst_flash = false;
        let mut found_burst_shake = false;
        
        // 检查所有当前存储的事件
        for event in flash_events.get_cursor().read(&flash_events) {
            match event {
                ScreenEffectEvent::Flash { color, .. } => {
                    let rgba = color.to_srgba();
                    info!("调试：发现闪烁事件，颜色: {:?}", rgba);
                    // 只要红色分量高，我们就认为是爆发闪光
                    if rgba.red > 0.9 {
                        found_burst_flash = true;
                    }
                }
                ScreenEffectEvent::Shake { trauma, .. } => {
                    info!("调试：发现震动事件，强度: {}", trauma);
                    if *trauma >= 0.8 {
                        found_burst_shake = true;
                    }
                }
                _ => {}
            }
        }
        
        assert!(found_burst_flash, "未触发人剑合一闪光特效");
        assert!(found_burst_shake, "未触发人剑合一强力震动");
        
        // 5. 验证粒子爆发 (count >= 50)
        let particle_events = app.world().resource::<Events<SpawnEffectEvent>>();
        let mut found_heavy_burst = false;
        for event in particle_events.get_cursor().read(&particle_events) {
            if event.count >= 50 {
                found_heavy_burst = true;
            }
        }
        assert!(found_heavy_burst, "未触发大规模粒子爆发");
        
        // 6. 验证剑意已重置
        let player = app.world().get_resource::<Player>().unwrap();
        assert_eq!(player.sword_intent, 0, "爆发后剑意应重置");
    }
    
    info!("✅ 人剑合一爆发特效验证通过！");
}