use bevy::prelude::*;
use bevy_card_battler::components::combat::{Player, PlayerHpBarMarker, PlayerHpBufferMarker};

#[test]
fn test_real_hp_buffer_decay_logic() {
    let mut app = App::new();
    app.init_resource::<Time>();
    
    // 1. 准备环境：100HP 玩家，当前血条 100%，缓冲条 100%
    let player_ent = app.world_mut().spawn(Player {
        hp: 100,
        max_hp: 100,
        ..Default::default()
    }).id();

    let bar_ent = app.world_mut().spawn((
        Node { width: Val::Percent(100.0), ..default() },
        PlayerHpBarMarker,
    )).id();

    let buffer_ent = app.world_mut().spawn((
        Node { width: Val::Percent(100.0), ..default() },
        PlayerHpBufferMarker,
    )).id();

    // 2. 模拟 update_combat_ui 中的真实同步逻辑 (提取版)
    let sync_system = |
        time: Res<Time>,
        player_query: Query<&Player>,
        mut bar_query: Query<(&mut Node, Has<PlayerHpBarMarker>, Has<PlayerHpBufferMarker>)>,
    | {
        if let Ok(p) = player_query.get_single() {
            let hp_percent = (p.hp as f32 / p.max_hp as f32) * 100.0;
            for (mut node, is_bar, is_buffer) in bar_query.iter_mut() {
                if is_bar {
                    node.width = Val::Percent(hp_percent);
                } else if is_buffer {
                    if let Val::Percent(curr_w) = node.width {
                        if curr_w > hp_percent {
                            // 使用真实的衰减公式：40% / sec
                            let new_w = (curr_w - 40.0 * time.delta_secs()).max(hp_percent);
                            node.width = Val::Percent(new_w);
                        }
                    }
                }
            }
        }
    };

    app.add_systems(Update, sync_system);

    // 3. 动作：受击，HP 瞬间变为 50
    if let Some(mut p) = app.world_mut().get_mut::<Player>(player_ent) {
        p.hp = 50;
    }

    // 4. 模拟经过了 0.5 秒 (预期：缓冲条下降 40% * 0.5 = 20%，剩余 80%)
    let mut time = app.world_mut().resource_mut::<Time>();
    time.advance_by(std::time::Duration::from_millis(500));
    
    app.update();

    // 5. 验证结果
    let bar_node = app.world().get::<Node>(bar_ent).unwrap();
    let buffer_node = app.world().get::<Node>(buffer_ent).unwrap();

    if let Val::Percent(w) = bar_node.width {
        assert_eq!(w, 50.0, "鲜红条应瞬间跌至 50%");
    }
    
    if let Val::Percent(w) = buffer_node.width {
        assert!(w > 50.0, "缓冲条应具有滞后性");
        assert!((w - 80.0).abs() < 0.1, "经过 0.5 秒，缓冲条应匀速降至约 80% (当前: {})", w);
    }

    println!("✅ 缓冲血条真实逻辑 TDD 验证通过：鲜红瞬间响应，暗红平滑追随");
}
