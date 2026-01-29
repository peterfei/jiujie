//! 战斗开始直接胜利Bug还原测试
//!
//! 问题描述：有一定机率进入战斗时直接跳转到胜利界面

use crate::test_utils::*;
use bevy::prelude::*;
use bevy_card_battler::states::GameState;
use bevy_card_battler::components::Enemy;

#[test]
fn bug_combat_001_enemy_hp_on_combat_enter() {
    let mut app = create_test_app();
    app.world_mut().spawn(Enemy::new(0, "测试妖兽", 30, 0));

    // 进入战斗状态
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.world_mut().run_schedule(StateTransition);

    advance_frames(&mut app, 1);

    let world = app.world_mut();
    let enemy = world.query::<&Enemy>().iter(world).next().expect("找不到敌人实体");

    println!("敌人 HP: {}/{}", enemy.hp, enemy.max_hp);
    assert!(enemy.hp > 0);
    assert!(!enemy.is_dead());
}

#[test]
fn bug_combat_002_victory_delay_not_active_on_start() {
    let mut app = create_test_app();
    app.world_mut().spawn(Enemy::new(0, "测试妖兽", 30, 0));

    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.world_mut().run_schedule(StateTransition);

    advance_frames(&mut app, 1);

    assert!(!is_victory_delay_active(&app));
}

#[test]
fn bug_combat_003_multiple_combat_entries() {
    for i in 0..10 {
        let mut app = create_test_app();
        app.world_mut().spawn(Enemy::new(0, "测试妖兽", 30, 0));

        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
        app.world_mut().run_schedule(StateTransition);

        advance_frames(&mut app, 1);

        let state = get_current_state(&app);
        let world = app.world_mut();
        let enemy = world.query::<&Enemy>().iter(world).next();

        match enemy {
            Some(e) => {
                if e.is_dead() {
                    panic!("迭代 {}: 敌人在战斗开始时就已经死亡!", i);
                }
            }
            None => {
                panic!("迭代 {}: 找不到敌人", i);
            }
        }

        assert_eq!(state, GameState::Combat);
    }
}

#[test]
fn bug_combat_004_check_enemy_entity_creation() {
    let mut app = create_test_app();
    app.world_mut().spawn(Enemy::new(0, "测试妖兽", 30, 0));

    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.world_mut().run_schedule(StateTransition);

    let world = app.world_mut();
    let after_enemy_count = world.query::<&Enemy>().iter(world).count();

    assert!(after_enemy_count >= 1);

    advance_frames(&mut app, 1);

    let world = app.world_mut();
    let final_enemy_count = world.query::<&Enemy>().iter(world).count();
    assert!(final_enemy_count >= 1);
}

#[test]
fn bug_combat_005_state_before_first_update() {
    let mut app = create_test_app();
    app.world_mut().spawn(Enemy::new(0, "测试妖兽", 30, 0));

    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.world_mut().run_schedule(StateTransition);

    advance_frames(&mut app, 1);

    let state_after = get_current_state(&app);
    let enemy_hp = {
        let world = app.world_mut();
        world.query::<&Enemy>().iter(world).next().map(|e| e.hp)
    };

    assert_eq!(state_after, GameState::Combat);
    assert!(enemy_hp.is_some());
    assert!(enemy_hp.unwrap() > 0);
}

#[test]
fn bug_combat_006_enemy_creation_timing() {
    let mut app = create_test_app();
    app.world_mut().spawn(Enemy::new(0, "测试妖兽", 30, 0));

    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.world_mut().run_schedule(StateTransition);

    advance_frames(&mut app, 1);

    let world = app.world_mut();
    let enemy = world.query::<&Enemy>().iter(world).next();

    assert!(enemy.is_some());
    assert!(enemy.unwrap().hp > 0);
}

#[test]
fn bug_combat_007_stress_test_reproduce_bug() {
    for _ in 0..50 {
        let mut app = create_test_app();
        app.world_mut().spawn(Enemy::new(0, "测试妖兽", 30, 0));

        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
        app.world_mut().run_schedule(StateTransition);
        advance_frames(&mut app, 1);

        let state = get_current_state(&app);
        let world = app.world_mut();
        let enemy = world.query::<&Enemy>().iter(world).next();

        assert_eq!(state, GameState::Combat);
        assert!(enemy.is_some());
        assert!(!enemy.unwrap().is_dead());
    }
}

#[test]
fn bug_combat_008_sequential_battles() {
    let mut app = create_test_app();

    // 第一场战斗
    app.world_mut().spawn(Enemy::new(0, "妖兽1", 30, 0));
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.world_mut().run_schedule(StateTransition);
    advance_frames(&mut app, 1);

    // 模拟胜利
    let enemies: Vec<Entity> = app.world_mut().query_filtered::<Entity, With<Enemy>>().iter(app.world_mut()).collect();
    for entity in enemies {
        if let Some(mut enemy) = app.world_mut().get_mut::<Enemy>(entity) {
            enemy.take_damage(100);
        }
    }

    advance_frames(&mut app, 10); // 等待胜利流程完成

    // 返回地图
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Map);
    app.world_mut().run_schedule(StateTransition);
    advance_frames(&mut app, 1);

    // 第二场战斗
    app.world_mut().spawn(Enemy::new(0, "妖兽2", 30, 0));
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.world_mut().run_schedule(StateTransition);
    advance_frames(&mut app, 1);

    let state_2 = get_current_state(&app);
    let world = app.world_mut();
    let enemy_2 = world.query::<&Enemy>().iter(world).next();

    assert_eq!(state_2, GameState::Combat);
    assert!(enemy_2.is_some());
    assert!(!enemy_2.unwrap().is_dead());
}