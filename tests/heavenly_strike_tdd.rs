use bevy::prelude::*;
use bevy::ecs::system::RunSystemOnce;
use bevy_card_battler::components::{Player, Enemy, HeavenlyStrikeCinematic, EnemySpriteMarker, Cultivation};

#[test]
fn test_heavenly_strike_delayed_damage_v2() {
    let mut app = App::new();
    
    app.init_resource::<Time>();
    app.add_event::<bevy_card_battler::components::SpawnEffectEvent>();
    app.add_event::<bevy_card_battler::components::ScreenEffectEvent>();
    app.add_event::<bevy_card_battler::components::PlaySfxEvent>();
    
    let mut cinematic = HeavenlyStrikeCinematic::default();
    cinematic.start(5, "雷暴".to_string());
    app.insert_resource(cinematic);
    
    app.world_mut().spawn((Player::default(), Cultivation::new()));
    let enemy_ent = app.world_mut().spawn((
        Enemy::new(1, "受难者", 50),
        EnemySpriteMarker { id: 1 },
        Transform::from_xyz(2.5, 0.0, 0.0),
    )).id();

    let dt = std::time::Duration::from_millis(200);
    
    for i in 0..22 { // 4.4s 循环
        {
            let mut cinematic = app.world_mut().resource_mut::<HeavenlyStrikeCinematic>();
            cinematic.timer.tick(dt);
            cinematic.effect_timer.tick(dt);
        }
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(dt);
        }

        let _ = app.world_mut().run_system_once(bevy_card_battler::plugins::process_heavenly_strike_cinematic);
        
        let (elapsed, active, applied, flash_count) = {
            let res = app.world().resource::<HeavenlyStrikeCinematic>();
            (res.timer.elapsed_secs(), res.active, res.damage_applied, res.flash_count)
        };
        let hp = app.world().get::<Enemy>(enemy_ent).unwrap().hp;
        
        println!("Tick {}: elapsed={:.2}, active={}, flashes={}, hp={}", i, elapsed, active, flash_count, hp);

        // 2.8s 是最终结算阈值
        if elapsed < 2.7 {
            assert_eq!(hp, 50, "在 2.8s 之前不应有伤害");
        } else if elapsed >= 2.9 && elapsed < 3.5 {
            assert_eq!(hp, 45, "在 2.8s 天罚降临后伤害应已结算");
            assert!(applied);
        }
    }

    let final_active = app.world().resource::<HeavenlyStrikeCinematic>().active;
    assert!(!final_active, "4.0s 后演出应圆满结束并关闭");
}
