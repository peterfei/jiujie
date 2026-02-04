//! 蜘蛛急速爬行与丝迹特效 TDD 验证

use bevy::prelude::*;
use bevy_card_battler::components::sprite::{PhysicalImpact, ActionType, BreathAnimation};
use bevy_card_battler::components::particle::{SpawnEffectEvent, EffectType};
use bevy_card_battler::states::GameState;

#[test]
fn test_spider_skitter_movement_and_trails() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    
    // 模拟必要的事件和状态
    app.add_event::<SpawnEffectEvent>();
    app.init_state::<GameState>();
    app.insert_state(GameState::Combat);

    // 注册要验证的系统
    app.add_systems(Update, bevy_card_battler::systems::sprite::update_physical_impacts);

    // 1. 创建蜘蛛实体，处于“急速爬行”动作中
    let spider_pos = Vec3::new(5.0, 0.8, 0.0);
    let impact = PhysicalImpact {
        home_position: spider_pos,
        action_type: ActionType::SkitterApproach,
        action_direction: -1.0, // 向玩家（左侧）冲刺
        target_offset_dist: 4.0, // 目标冲刺距离
        action_timer: 1.0,
        ..default()
    };
    
    let spider_ent = app.world_mut().spawn((
        impact,
        BreathAnimation::default(),
        Transform::from_translation(spider_pos),
    )).id();

    // 2. 运行几帧模拟移动
    app.update();
    
    // 3. 验证位移是否包含扰动 (非纯线性)
    let transform_after = app.world().get::<Transform>(spider_ent).unwrap();
    
    // transform.translation.y 应该包含 sin 扰动
    // 初始高度 0.8, Skitter 逻辑会叠加 jerky_phase.sin() * 0.08
    assert!(transform_after.translation.y != 0.8, "急速爬行应该产生垂直扰动（当前高度: {}）", transform_after.translation.y);

    // 4. 验证丝迹特效事件生成
    // 运行更多帧以确保触发丝迹计时器
    for _ in 0..30 {
        app.update();
    }

    let silk_trail_count = {
        let events = app.world().resource::<Events<SpawnEffectEvent>>();
        let mut reader = events.get_cursor();
        let mut count = 0;
        for event in reader.read(events) {
            if let EffectType::SilkTrail = event.effect_type {
                count += 1;
            }
        }
        count
    };
    
    assert!(silk_trail_count > 0, "爬行过程中应该留下丝迹特效，实际生成: {}", silk_trail_count);

    // 5. 验证是否跑出了屏幕（终点检测）
    let transform_final = app.world().get::<Transform>(spider_ent).unwrap();
    let moved_dist = (transform_final.translation.x - 5.0).abs();
    // 目标位移是 4.0，我们允许 0.5 的误差范围，但不允许跑出 10.0 单位（穿透屏幕）
    assert!(moved_dist <= 4.5, "蜘蛛跑出了预期范围，实际位移: {}", moved_dist);
    
    println!("=== 蜘蛛急速爬行 TDD 验证通过：包含边界检测 ===");
}