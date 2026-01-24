//! 战斗开始直接胜利Bug还原测试
//!
//! 问题描述：有一定机率进入战斗时直接跳转到胜利界面
//!
//! 可能原因：
//! 1. 敌人初始化时HP为0
//! 2. check_combat_end系统在战斗开始前就检测到敌人死亡
//! 3. 状态转换时机问题

use crate::test_utils::*;
use bevy::prelude::*;
use bevy_card_battler::states::GameState;
use bevy_card_battler::components::Enemy;

#[test]
fn bug_combat_001_enemy_hp_on_combat_enter() {
    let mut app = create_test_app();

    // 进入战斗状态
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.world_mut().run_schedule(StateTransition);

    // 运行一帧让系统初始化
    advance_frames(&mut app, 1);

    // 检查敌人HP
    let world = app.world_mut();
    let enemy = match world.query::<&Enemy>().get_single(world) {
        Ok(e) => e,
        Err(e) => {
            panic!("找不到敌人实体: {:?}", e);
        }
    };

    println!("敌人 HP: {}/{}", enemy.hp, enemy.max_hp);
    assert!(enemy.hp > 0, "敌人初始HP应该大于0，当前: {}", enemy.hp);
    assert!(!enemy.is_dead(), "敌人初始状态不应该是死亡状态");
}

#[test]
fn bug_combat_002_victory_delay_not_active_on_start() {
    let mut app = create_test_app();

    // 进入战斗状态
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.world_mut().run_schedule(StateTransition);

    // 运行一帧让系统初始化
    advance_frames(&mut app, 1);

    // 胜利延迟不应该激活
    assert!(!is_victory_delay_active(&app), "战斗开始时胜利延迟不应该激活");
}

#[test]
fn bug_combat_003_multiple_combat_entries() {
    // 测试多次进入战斗的情况
    for i in 0..10 {
        let mut app = create_test_app();

        // 进入战斗状态
        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
        app.world_mut().run_schedule(StateTransition);

        // 运行一帧让系统初始化
        advance_frames(&mut app, 1);

        // 检查状态
        let state = get_current_state(&app);
        let world = app.world_mut();
        let enemy = world.query::<&Enemy>().get_single(world);

        match enemy {
            Ok(e) => {
                if e.is_dead() {
                    panic!("迭代 {}: 敌人在战斗开始时就已经死亡! HP: {}/{}",
                        i, e.hp, e.max_hp);
                }
            }
            Err(e) => {
                panic!("迭代 {}: 找不到敌人: {:?}", i, e);
            }
        }

        if state != GameState::Combat {
            panic!("迭代 {}: 战斗开始后状态应该是Combat，实际是: {:?}", i, state);
        }
    }

    println!("✓ 多次进入战斗测试通过（10次迭代）");
}

#[test]
fn bug_combat_004_check_enemy_entity_creation() {
    let mut app = create_test_app();

    // 在进入战斗前检查敌人是否存在
    let world = app.world_mut();
    let before_enemy_count = world.query::<&Enemy>().iter(world).count();

    println!("进入战斗前敌人数量: {}", before_enemy_count);

    // 进入战斗状态
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.world_mut().run_schedule(StateTransition);

    // 检查进入战斗后的敌人数量
    let world = app.world_mut();
    let after_enemy_count = world.query::<&Enemy>().iter(world).count();

    println!("进入战斗后敌人数量: {}", after_enemy_count);

    assert_eq!(after_enemy_count, 1, "进入战斗后应该有1个敌人");

    // 运行一帧让系统初始化
    advance_frames(&mut app, 1);

    // 再次检查
    let world = app.world_mut();
    let final_enemy_count = world.query::<&Enemy>().iter(world).count();
    let enemy = world.query::<&Enemy>().get_single(world);

    println!("初始化后敌人数量: {}", final_enemy_count);

    match enemy {
        Ok(e) => {
            println!("敌人详情: HP={}/{}, is_dead={}", e.hp, e.max_hp, e.is_dead());
            assert!(!e.is_dead(), "敌人不应该在初始化时死亡");
        }
        Err(e) => {
            panic!("找不到敌人: {:?}", e);
        }
    }
}

#[test]
fn bug_combat_005_state_before_first_update() {
    // 测试第一帧更新前的状态
    let mut app = create_test_app();

    // 进入战斗状态
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.world_mut().run_schedule(StateTransition);

    // 在运行任何帧之前检查状态
    let state_before = get_current_state(&app);
    let victory_delay_before = is_victory_delay_active(&app);

    println!("第一帧前 - 状态: {:?}, 胜利延迟: {}",
        state_before, victory_delay_before);

    // 运行第一帧
    advance_frames(&mut app, 1);

    let state_after = get_current_state(&app);
    let victory_delay_after = is_victory_delay_active(&app);
    let enemy_hp = {
        let world = app.world_mut();
        world.query::<&Enemy>().get_single(world).map(|e| e.hp).ok()
    };

    println!("第一帧后 - 状态: {:?}, 胜利延迟: {}, 敌人HP: {:?}",
        state_after, victory_delay_after, enemy_hp);

    assert_eq!(state_after, GameState::Combat, "第一帧后应该仍在战斗状态");
    assert!(!victory_delay_after, "第一帧后胜利延迟不应该激活");
    assert!(enemy_hp.is_some(), "应该有敌人实体");
    assert!(enemy_hp.unwrap() > 0, "敌人HP应该大于0");
}

#[test]
fn bug_combat_006_enemy_creation_timing() {
    // 详细追踪敌人创建时机
    let mut app = create_test_app();

    // 阶段0：进入战斗前
    let world = app.world_mut();
    let count_0 = world.query::<&Enemy>().iter(world).count();

    println!("阶段0（进入前）：敌人数量 = {}", count_0);

    // 阶段1：设置状态并运行转换
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.world_mut().run_schedule(StateTransition);

    let count_1 = {
        let world = app.world_mut();
        world.query::<&Enemy>().iter(world).count()
    };
    let state_1 = get_current_state(&app);
    println!("阶段1（状态转换后）：敌人数量 = {}, 状态 = {:?}", count_1, state_1);

    // 阶段2：运行第一帧
    advance_frames(&mut app, 1);

    let (count_2, enemy_hp) = {
        let world = app.world_mut();
        let count = world.query::<&Enemy>().iter(world).count();
        let enemy = world.query::<&Enemy>().get_single(world).ok().map(|e| (e.hp, e.max_hp, e.name.clone()));
        (count, enemy)
    };
    let state_2 = get_current_state(&app);

    println!("阶段2（第一帧后）：敌人数量 = {}, 状态 = {:?}", count_2, state_2);

    match enemy_hp {
        Some((hp, max_hp, name)) => {
            println!("  敌人: HP={}/{}, name={}", hp, max_hp, name);
            if hp <= 0 {
                panic!("❌ BUG确认：敌人在第一帧后死亡！HP: {}/{}", hp, max_hp);
            }
        }
        None => {
            panic!("❌ BUG确认：第一帧后找不到敌人！");
        }
    }
}

#[test]
fn bug_combat_007_stress_test_reproduce_bug() {
    // 压力测试，尝试重现随机bug
    let mut failures = Vec::new();

    for i in 0..100 {
        let mut app = create_test_app();

        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
        app.world_mut().run_schedule(StateTransition);
        advance_frames(&mut app, 1);

        let state = get_current_state(&app);
        let world = app.world_mut();
        let enemy_result = world.query::<&Enemy>().get_single(world);

        let is_bug = match enemy_result {
            Ok(e) => e.is_dead() || state != GameState::Combat,
            Err(_) => state != GameState::Combat,
        };

        if is_bug {
            failures.push((i, state, enemy_result.map(|e| (e.hp, e.max_hp)).ok()));
        }
    }

    if !failures.is_empty() {
        println!("❌ 发现 {} 次失败：", failures.len());
        for (i, state, hp) in &failures {
            println!("  迭代 {}: 状态={:?}, HP={:?}", i, state, hp);
        }
        panic!("Bug已重现！失败率: {}/100", failures.len());
    } else {
        println!("✓ 100次迭代未发现bug");
    }
}

/// 测试连续多次战斗场景
#[test]
fn bug_combat_008_sequential_battles() {
    let mut app = create_test_app();

    // 第一场战斗
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.world_mut().run_schedule(StateTransition);
    advance_frames(&mut app, 1);

    // 检查第一场战斗
    let state_1 = get_current_state(&app);
    let enemy_1 = {
        let world = app.world_mut();
        world.query::<&Enemy>().get_single(world).ok().map(|e| (e.hp, e.max_hp))
    };

    println!("第一场战斗: 状态={:?}, 敌人HP={:?}", state_1, enemy_1);

    // 模拟战斗胜利（杀死敌人）
    if let Some((entity, _)) = app.world_mut().query_filtered::<(Entity, &Enemy), ()>().iter(app.world_mut()).next() {
        if let Some(mut enemy) = app.world_mut().get_mut::<Enemy>(entity) {
            enemy.take_damage(100); // 确保敌人死亡
        }
    }

    // 运行几帧让胜利流程完成
    advance_frames(&mut app, 5);

    // 检查是否转换到奖励状态
    let state_after_victory = get_current_state(&app);
    println!("胜利后状态: {:?}", state_after_victory);

    // 返回地图
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Map);
    app.world_mut().run_schedule(StateTransition);
    advance_frames(&mut app, 1);

    // 第二场战斗
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.world_mut().run_schedule(StateTransition);
    advance_frames(&mut app, 1);

    // 检查第二场战斗
    let state_2 = get_current_state(&app);
    let enemy_2 = {
        let world = app.world_mut();
        world.query::<&Enemy>().get_single(world).ok().map(|e| (e.hp, e.max_hp, e.is_dead()))
    };

    println!("第二场战斗: 状态={:?}, 敌人={:?}", state_2, enemy_2);

    match enemy_2 {
        Some((hp, max_hp, is_dead)) => {
            assert!(!is_dead, "第二场战斗敌人不应该是死亡状态！HP: {}/{}", hp, max_hp);
            assert_eq!(state_2, GameState::Combat, "第二场战斗应该处于Combat状态");
        }
        None => {
            panic!("第二场战斗找不到敌人！");
        }
    }
}
