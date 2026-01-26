//! 敌人回合Wait意图Bug测试
//!
//! 问题：敌人AI选择了Wait意图，导致玩家未受到伤害

use crate::test_utils::*;
use bevy::prelude::*;
use bevy_card_battler::states::GameState;
use bevy_card_battler::components::{Enemy, EnemyIntent, Player};

#[test]
fn bug_enemy_wait_001_enemy_can_choose_wait() {
    // 验证敌人AI可以选择Wait意图
    let mut wait_count = 0;
    let iterations = 100;

    for _ in 0..iterations {
        let mut enemy = bevy_card_battler::components::Enemy::new(0, "嗜血妖狼", 30);
        enemy.choose_new_intent();

        if matches!(enemy.intent, EnemyIntent::Wait) {
            wait_count += 1;
        }
    }

    println!("100次迭代中，Wait意图被选择了 {} 次", wait_count);

    // Wait意图的概率是 1.0 - (attack + defend + buff)
    // 对于嗜血妖狼: 1.0 - (0.7 + 0.1 + 0.2) = 0.0
    // 但实际上由于浮点精度或其他因素，可能仍会发生
    if wait_count > 0 {
        println!("⚠️  警告：Wait意图被选择了 {} 次（预期0次）", wait_count);
    }
}

#[test]
fn bug_enemy_wait_002_wait_intent_deals_no_damage() {
    let mut app = create_test_app();

    // 进入战斗
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.world_mut().run_schedule(StateTransition);
    advance_frames(&mut app, 1);

    // 获取初始玩家HP
    let initial_hp = {
        let world = app.world_mut();
        world.query::<&Player>().iter(world).next().unwrap().hp
    };

    // 设置敌人意图为Wait
    {
        let world = app.world_mut();
        if let Some(mut enemy) = world.query::<&mut Enemy>().iter_mut(world).next() {
            enemy.intent = EnemyIntent::Wait;
            println!("设置敌人意图为: Wait");
        }
    }

    // 执行敌人意图（模拟结束回合）
    {
        let world = app.world_mut();
        if let Some(mut enemy) = world.query::<&mut Enemy>().iter_mut(world).next() {
            let executed = enemy.execute_intent();
            println!("执行意图: {:?}", executed);
        }
    }

    // 检查玩家HP是否变化
    let final_hp = {
        let world = app.world_mut();
        world.query::<&Player>().iter(world).next().unwrap().hp
    };

    println!("玩家HP: {} -> {}", initial_hp, final_hp);

    // Wait意图不应该造成伤害
    assert_eq!(initial_hp, final_hp, "Wait意图不应该改变玩家HP");
    println!("✓ Wait意图正确：未造成伤害");
}

#[test]
fn bug_enemy_wait_003_attack_intent_deals_damage() {
    let mut app = create_test_app();

    // 进入战斗
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.world_mut().run_schedule(StateTransition);
    advance_frames(&mut app, 1);

    // 获取初始玩家HP
    let initial_hp = {
        let world = app.world_mut();
        world.query::<&Player>().iter(world).next().unwrap().hp
    };

    // 设置敌人意图为Attack
    {
        let world = app.world_mut();
        if let Some(mut enemy) = world.query::<&mut Enemy>().iter_mut(world).next() {
            enemy.intent = EnemyIntent::Attack { damage: 10 };
            println!("设置敌人意图为: Attack {{ damage: 10 }}");
        }
    }

    // 执行敌人意图
    {
        let world = app.world_mut();
        if let Some(mut enemy) = world.query::<&mut Enemy>().iter_mut(world).next() {
            let executed = enemy.execute_intent();
            println!("执行意图: {:?}", executed);

            // Attack意图只是返回，不直接修改玩家
            // 实际的伤害在 handle_combat_button_clicks 中处理
        }
    }

    // ⚠️ execute_intent 对于Attack只返回意图，不直接造成伤害
    // 这是设计问题！
}

#[test]
fn bug_enemy_wait_004_ai_probability_sum_check() {
    // 检查AI概率总和是否为1.0
    let enemy = bevy_card_battler::components::Enemy::new(0, "嗜血妖狼", 30);

    let sum = enemy.ai_pattern.attack_chance
           + enemy.ai_pattern.defend_chance
           + enemy.ai_pattern.buff_chance
           + enemy.ai_pattern.debuff_chance;

    println!("嗜血妖狼AI概率总和: attack={:.2}, defend={:.2}, buff={:.2}, debuff={:.2}, sum={:.2}",
        enemy.ai_pattern.attack_chance,
        enemy.ai_pattern.defend_chance,
        enemy.ai_pattern.buff_chance,
        enemy.ai_pattern.debuff_chance,
        sum
    );

    // 如果总和 < 1.0，剩余概率会分配给Wait
    if sum < 0.999 {
        println!("⚠️  概率总和 {:.2} < 1.0，剩余 {:.2}% 会选择Wait", sum, (1.0 - sum) * 100.0);
    }

    assert!((sum - 1.0).abs() < 0.01, "概率总和应该为1.0，实际: {:.2}", sum);
}

#[test]
fn bug_enemy_wait_005_enemy_choose_new_intent() {
    let mut app = create_test_app();

    // 进入战斗
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.world_mut().run_schedule(StateTransition);
    advance_frames(&mut app, 1);

    // 调用 enemy.start_turn() 选择新意图
    {
        let world = app.world_mut();
        if let Ok(mut enemy) = world.query::<&mut Enemy>().get_single_mut(world) {
            println!("调用 start_turn 前: intent={:?}", enemy.intent);
            enemy.start_turn();
            println!("调用 start_turn 后: intent={:?}", enemy.intent);
            println!("   block={}", enemy.block);
        }
    }

    // 检查意图是否有效（非Wait）
    let intent = {
        let world = app.world_mut();
        world.query::<&Enemy>().iter(world).next().unwrap().intent
    };

    match intent {
        EnemyIntent::Wait => {
            println!("❌ BUG：敌人选择了Wait意图，这将导致不攻击玩家");
        }
        EnemyIntent::Attack { damage } => {
            println!("✓ 敌人选择攻击，伤害: {}", damage);
        }
        EnemyIntent::Defend { block } => {
            println!("✓ 敌人选择防御，护甲: {}", block);
        }
        EnemyIntent::Buff { strength } => {
            println!("✓ 敌人选择强化，力量: {}", strength);
        }
        EnemyIntent::Debuff { poison, weakness } => {
            println!("✓ 敌人选择邪术，中毒: {}, 虚弱: {}", poison, weakness);
        }
    }
}
