use bevy::prelude::*;
use bevy_card_battler::components::{Enemy, EnemyType, EnemyIntent, VictoryDelay};
use bevy_card_battler::components::particle::{SpawnEffectEvent, EffectType};
use bevy_card_battler::components::sprite::Ghost;
use bevy_card_battler::states::GameState;
use bevy_card_battler::components::combat::{EnemyActionQueue};
use crate::test_utils::*;

mod test_utils;

/// 测试蜘蛛 Debuff 意图时是否发送 WebShot 事件
#[test]
fn test_spider_attack_triggers_web_effects() {
    let mut app = create_test_app();

    println!("=== 开始测试：蜘蛛 Debuff 吐丝特效 ===");

    // 1. 设置场景：进入Combat状态
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.world_mut().run_schedule(StateTransition);

    // 手动运行一次 update 来触发 OnEnter 系统，创建必要的实体
    app.update();

    // 验证状态是否正确设置
    let current_state = app.world().get_resource::<State<GameState>>();
    println!("✓ 进入 Combat 状态, 当前状态: {:?}", current_state.map(|s| s.get()));

    // 验证必需的实体已创建
    {
        let world = app.world_mut();
        let player_count = world.query_filtered::<Entity, With<bevy_card_battler::components::Player>>().iter(world).count();
        let hand_count = world.query_filtered::<Entity, With<bevy_card_battler::components::Hand>>().iter(world).count();
        println!("✓ 检测到 Player: {}, Hand: {}", player_count, hand_count);
    }

    // 清理默认生成的敌人，生成特定的蜘蛛
    {
        let world = app.world_mut();
        let entities: Vec<Entity> = world.query_filtered::<Entity, With<Enemy>>().iter(world).collect();
        println!("清理 {} 个默认敌人", entities.len());
        for e in entities {
            app.world_mut().entity_mut(e).despawn_recursive();
        }
    }

    // 2. 创建蜘蛛敌人
    let spider_id = app.world_mut().spawn((
        Enemy::with_type(1, "剧毒蛛", 20, EnemyType::PoisonSpider),
        Transform::from_xyz(3.0, 0.0, 0.0),
    )).id();
    println!("✓ 创建蜘蛛敌人 (entity: {:?})", spider_id);

    // 3. 创建 EnemySpriteMarker（模拟渲染系统创建的标记）
    let spider_sprite_id = app.world_mut().spawn((
        bevy_card_battler::components::sprite::EnemySpriteMarker { id: 1 },
        Transform::from_xyz(3.0, 0.0, 0.0),
    )).id();
    println!("✓ 创建 EnemySpriteMarker (id: 1, entity: {:?})", spider_sprite_id);

    // 4. 设置战斗状态为敌人回合
    {
        let mut combat_state = app.world_mut().resource_mut::<bevy_card_battler::components::CombatState>();
        combat_state.phase = bevy_card_battler::components::TurnPhase::EnemyTurn;
        println!("✓ 设置战斗阶段: EnemyTurn");
    }

    // 5. 设置敌人行动队列
    app.world_mut().insert_resource(EnemyActionQueue {
        enemies: vec![spider_id],
        processing: true,
        current_index: 0,
        timer: bevy::time::Timer::from_seconds(0.001, bevy::time::TimerMode::Once),
    });
    println!("✓ 设置敌人行动队列: processing=true, 1个敌人");

    // 6. 运行足够帧数以确保计时器完成
    println!("开始运行 30 帧以确保特效触发...");
    for _ in 0..30 {
        app.update();
    }
    println!("✓ 完成更新");

    // 检查队列状态
    let queue = app.world().get_resource::<EnemyActionQueue>();
    if let Some(q) = queue {
        println!("队列状态: current_index={}, processing={}", q.current_index, q.processing);
    }

    // 7. 验证是否发送了 WebShot 事件
    let effect_events = app.world().resource::<Events<SpawnEffectEvent>>();
    let mut cursor = effect_events.get_cursor();

    let mut total_events = 0;
    let mut web_shot_count = 0;
    let mut total_particles = 0;
    for event in cursor.read(effect_events) {
        total_events += 1;
        if event.effect_type == EffectType::WebShot {
            web_shot_count += 1;
            total_particles += event.count; // count 是 burst() 设置的粒子数量
            println!("事件: WebShot, burst={}, position={:?}", event.count, event.position);
        }
    }

    println!("共检测到 {} 个 SpawnEffectEvent，其中 WebShot: {} 个，总计 {} 个粒子", total_events, web_shot_count, total_particles);

    // 蜘蛛的 Debuff 意图应该发送 WebShot 粒子（至少 20 个）
    assert!(total_particles >= 20, "蜘蛛 Debuff 时应该发送至少 20 个 WebShot 粒子，实际: {}", total_particles);

    // 8. 验证是否生成了全屏蛛网 Ghost 实体
    // 注意：Debuff 意图不会生成 Ghost，只有 Attack 意图才会
    // 所以这里我们只检查 WebShot 事件

    println!("=== 测试通过 ===");
}
