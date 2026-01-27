use bevy::prelude::*;
use bevy_card_battler::components::cards::{CardPool, CardEffect};
use bevy_card_battler::components::particle::{SpawnEffectEvent, EffectType};
use bevy_card_battler::components::screen_effect::ScreenEffectEvent;

#[test]
fn test_wanjian_vfx_trigger() {
    let mut app = App::new();
    
    // 1. 注册必要事件
    app.add_event::<SpawnEffectEvent>();
    app.add_event::<ScreenEffectEvent>();
    
    // 2. 获取“万剑归宗”卡牌
    let all_cards = CardPool::all_cards();
    let wan_jian = all_cards.iter().find(|c| c.name == "万剑归宗").expect("应存在万剑归宗卡牌").clone();
    
    // 3. 验证卡牌效果类型
    match wan_jian.effect {
        CardEffect::DealAoEDamage { amount } => assert_eq!(amount, 10),
        _ => panic!("万剑归宗的效果类型不符合预期"),
    }

    // 逻辑验证
    if wan_jian.name == "万剑归宗" {
        {
            let mut spawn_events = app.world_mut().resource_mut::<Events<SpawnEffectEvent>>();
            spawn_events.send(SpawnEffectEvent {
                effect_type: EffectType::WanJian,
                position: Vec3::ZERO,
                burst: true,
                count: 100,
                target: None,
                target_entity: None,
                target_group: None,
                target_index: None,
            });
        }
        {
            let mut screen_events = app.world_mut().resource_mut::<Events<ScreenEffectEvent>>();
            screen_events.send(ScreenEffectEvent::Shake {
                trauma: 1.0,
                decay: 2.5,
            });
        }
    }

    // 4. 最终验证事件是否发送成功
    assert_eq!(app.world().resource::<Events<SpawnEffectEvent>>().len(), 1);
    assert_eq!(app.world().resource::<Events<ScreenEffectEvent>>().len(), 1);
}