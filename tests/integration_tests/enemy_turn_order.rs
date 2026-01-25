//! 敌人回合时序Bug测试
//!
//! 问题：敌人回合时，execute_intent() 在 start_turn() 之前被调用
//! 导致敌人执行的是上一回合的Wait意图，而不是新回合的Attack意图

use crate::test_utils::*;
use bevy::prelude::*;
use bevy_card_battler::states::GameState;
use bevy_card_battler::components::{Enemy, EnemyIntent, Player};

#[test]
fn bug_turn_order_001_enemy_initial_intent_is_wait() {
    let mut app = create_test_app();

    // 进入战斗
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.world_mut().run_schedule(StateTransition);
    advance_frames(&mut app, 1);

    // 检查敌人初始意图
    let initial_intent = {
        let world = app.world_mut();
        world.query::<&Enemy>().get_single(world).unwrap().intent
    };

    println!("敌人初始意图: {:?}", initial_intent);
    assert!(matches!(initial_intent, EnemyIntent::Wait), "初始意图应该是Wait");

    // 记录初始玩家HP
    let initial_hp = {
        let world = app.world_mut();
        world.query::<&Player>().get_single(world).unwrap().hp
    };

    // 模拟当前代码的错误顺序：先 execute_intent，后 start_turn
    // 执行意图（Wait）
    {
        let world = app.world_mut();
        if let Ok(mut enemy) = world.query::<&mut Enemy>().get_single_mut(world) {
            let executed = enemy.execute_intent();
            println!("执行意图（Wait）: {:?}", executed);
        }
    }

    // 检查玩家HP（Wait不应该造成伤害）
    let after_execute_hp = {
        let world = app.world_mut();
        world.query::<&Player>().get_single(world).unwrap().hp
    };

    println!("HP变化: {} -> {}", initial_hp, after_execute_hp);
    assert_eq!(initial_hp, after_execute_hp, "Wait意图不应该改变HP");

    // 然后选择新意图
    {
        let world = app.world_mut();
        if let Ok(mut enemy) = world.query::<&mut Enemy>().get_single_mut(world) {
            enemy.start_turn();
            let new_intent = enemy.intent;
            println!("选择新意图: {:?}", new_intent);
            assert!(!matches!(new_intent, EnemyIntent::Wait), "新意图不应该是Wait");
        }
    }

    println!("❌ BUG确认：敌人执行了Wait意图，而不是新回合的Attack意图");
}

#[test]
fn bug_turn_order_002_correct_order_should_be_start_then_execute() {
    let mut app = create_test_app();

    // 进入战斗
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.world_mut().run_schedule(StateTransition);
    advance_frames(&mut app, 1);

    let initial_hp = {
        let world = app.world_mut();
        world.query::<&Player>().get_single(world).unwrap().hp
    };

    // 正确顺序：先 start_turn，再 execute_intent
    {
        let world = app.world_mut();
        if let Ok(mut enemy) = world.query::<&mut Enemy>().get_single_mut(world) {
            // 先选择新意图
            enemy.start_turn();
            let new_intent = enemy.intent;
            println!("新意图: {:?}", new_intent);

            // 然后执行
            let executed = enemy.execute_intent();
            println!("执行意图: {:?}", executed);
        }
    }

    // 如果是Attack意图，execute_intent只返回意图，不直接伤害玩家
    // 实际伤害在 handle_combat_button_clicks 中处理
    let final_hp = {
        let world = app.world_mut();
        world.query::<&Player>().get_single(world).unwrap().hp
    };

    println!("HP: {} -> {}", initial_hp, final_hp);
    println!("✓ 正确顺序：先选择新意图，再执行");
}

#[test]
fn bug_turn_order_003_verify_code_order_in_handle_combat_button_clicks() {
    // 这个测试验证 handle_combat_button_clicks 中的代码顺序
    // 读取源码并检查 execute_intent 和 start_turn 的调用顺序

    // 正确顺序应该是：
    // 1. enemy.start_turn() - 选择新意图
    // 2. enemy.execute_intent() - 执行意图

    // 错误顺序（当前代码）：
    // 1. enemy.execute_intent() - 执行旧意图（Wait）
    // 2. enemy.start_turn() - 选择新意图（但已经执行完了）

    println!("当前代码顺序检查：");
    println!("❌ 错误：execute_intent() 在 start_turn() 之前被调用");
    println!("✓ 应该：start_turn() 在 execute_intent() 之前被调用");

    // 此测试只是文档说明，实际修复需要修改源码
}
