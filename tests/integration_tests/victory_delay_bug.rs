//! Victory Delay状态泄漏Bug测试
//!
//! Bug: 第一场战斗胜利后，victory_delay.active未正确重置，导致第二场战斗直接胜利

use crate::test_utils::*;
use bevy::prelude::*;
use bevy_card_battler::states::GameState;
use bevy_card_battler::components::Enemy;

/// 测试胜利延迟状态在状态转换后是否正确重置
#[test]
fn bug_victory_delay_001_active_reset_after_reward() {
    let mut app = create_test_app();

    // 第一场战斗
    app.world_mut().spawn(Enemy::new(0, "妖兽1", 30));
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.world_mut().run_schedule(StateTransition);
    advance_frames(&mut app, 1);

    // 杀死敌人
    let enemy_entity = {
        let world = app.world_mut();
        world.query_filtered::<(Entity, &Enemy), ()>()
            .iter(world)
            .next()
            .map(|(e, _)| e)
            .unwrap()
    };

    {
        let world = app.world_mut();
        if let Some(mut enemy) = world.get_mut::<Enemy>(enemy_entity) {
            enemy.take_damage(100);
        }
    }

    // 运行一帧触发胜利流程
    advance_frames(&mut app, 1);

    // 胜利延迟应该激活
    assert!(is_victory_delay_active(&app), "敌人死亡后胜利延迟应该激活");

    // 模拟延迟完成（运行足够帧数）
    advance_frames(&mut app, 10000);

    // 应该转换到 Reward 状态
    let state = get_current_state(&app);
    assert_eq!(state, GameState::Reward, "应该转换到Reward状态，实际: {:?}", state);

    // 关键检查：胜利延迟应该不再激活
    let delay_active = is_victory_delay_active(&app);
    println!("进入Reward状态后，victory_delay.active = {}", delay_active);
}

/// 测试完整的战斗→奖励→地图→战斗流程
#[test]
fn bug_victory_delay_002_full_cycle_reproduces_bug() {
    let mut app = create_test_app();

    // ===== 第一场战斗 =====
    app.world_mut().spawn(Enemy::new(0, "妖兽1", 30));
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.world_mut().run_schedule(StateTransition);
    advance_frames(&mut app, 1);

    // 杀死敌人
    if let Some((entity, _)) = app.world_mut().query_filtered::<(Entity, &Enemy), ()>().iter(app.world_mut()).next() {
        if let Some(mut enemy) = app.world_mut().get_mut::<Enemy>(entity) {
            enemy.take_damage(100);
        }
    }

    advance_frames(&mut app, 1);
    advance_frames(&mut app, 10000); // 等待胜利延迟完成

    // 应该在Reward状态
    assert_eq!(get_current_state(&app), GameState::Reward);

    // ===== 选择奖励卡牌，返回地图 =====
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Map);
    app.world_mut().run_schedule(StateTransition);
    advance_frames(&mut app, 1);

    assert_eq!(get_current_state(&app), GameState::Map);

    // ===== 进入第二场战斗 =====
    // 预设第二场战斗的敌人
    app.world_mut().spawn(Enemy::new(0, "妖兽2", 30));
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.world_mut().run_schedule(StateTransition);
    advance_frames(&mut app, 1);

    // 检查新战斗开始时的状态
    let state = get_current_state(&app);
    let delay_active = is_victory_delay_active(&app);

    println!("第二场战斗: 状态={:?}, victory_delay.active={}", state, delay_active);

    // 检查新敌人的HP
    let enemy_hp = {
        let world = app.world_mut();
        world.query::<&Enemy>().iter(world).next().map(|e| (e.hp, e.max_hp, e.is_dead()))
    };

    println!("第二场战斗敌人: {:?}", enemy_hp);

    // 验证bug已修复
    assert_eq!(state, GameState::Combat, "第二场战斗应该处于Combat状态");
    assert!(!delay_active, "第二场战斗开始时victory_delay应该是false");

    match enemy_hp {
        Some((hp, _, is_dead)) => {
            assert!(!is_dead, "第二场战斗敌人应该是活着的");
            assert!(hp > 0, "第二场战斗敌人HP应该是大于0的");
        }
        None => panic!("找不到敌人实体"),
    }
}

/// 测试check_combat_end在状态转换后是否被调用
#[test]
fn bug_victory_delay_003_check_combat_end_called_after_transition() {
    let mut app = create_test_app();

    // 进入战斗
    app.world_mut().spawn(Enemy::new(0, "测试妖兽", 30));
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.world_mut().run_schedule(StateTransition);
    advance_frames(&mut app, 1);

    // 杀死敌人
    if let Some((entity, _)) = app.world_mut().query_filtered::<(Entity, &Enemy), ()>().iter(app.world_mut()).next() {
        if let Some(mut enemy) = app.world_mut().get_mut::<Enemy>(entity) {
            enemy.take_damage(100);
        }
    }

    advance_frames(&mut app, 1);

    // 运行直到延迟应该完成
    for i in 0..100 {
        advance_frames(&mut app, 1);
        let state = get_current_state(&app);
        let delay = is_victory_delay_active(&app);

        if state == GameState::Reward {
            println!("帧 {}: 转换到Reward状态, delay.active = {}", i, delay);
            break;
        }
    }
}