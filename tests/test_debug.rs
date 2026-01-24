//! 调试集成测试

use crate::test_utils::*;
use bevy_card_battler::components::Enemy;
use bevy::prelude::*;

#[test]
fn debug_time_progression() {
    let mut app = create_test_app();

    // 设置战斗场景（setup_combat_ui 会创建一个 HP: 30 的敌人）
    let enemy_entity = setup_combat_scene(&mut app);

    // 运行一帧让系统初始化
    advance_frames(&mut app, 1);

    // 杀死敌人
    kill_enemy(&mut app, enemy_entity);

    // 运行一帧触发胜利流程
    advance_frames(&mut app, 1);

    println!("初始延迟时间: {}", get_victory_delay_elapsed(&app));

    // 运行多帧并检查时间
    for i in 0..10 {
        advance_frames(&mut app, 1);
        let elapsed = get_victory_delay_elapsed(&app);
        let active = is_victory_delay_active(&app);
        println!("帧 {}: 经过时间={}, 激活={}", i+1, elapsed, active);
    }

    // 运行更多帧
    advance_frames(&mut app, 100);
    let elapsed = get_victory_delay_elapsed(&app);
    let active = is_victory_delay_active(&app);
    println!("运行100帧后: 经过时间={}, 激活={}", elapsed, active);
}

#[test]
fn debug_enemy_entity_exists() {
    let mut app = create_test_app();

    // 设置战斗场景（setup_combat_ui 会创建一个 HP: 30 的敌人）
    let enemy_entity = setup_combat_scene(&mut app);

    // 杀死敌人
    kill_enemy(&mut app, enemy_entity);

    // 运行一帧让系统处理
    advance_frames(&mut app, 1);

    // 检查敌人是否存在且已死亡
    let world = app.world_mut();
    let enemies = world.query::<&Enemy>().iter(world).collect::<Vec<_>>();

    println!("敌人实体数量: {}", enemies.len());
    for enemy in enemies {
        println!("敌人 HP: {}, is_dead: {}", enemy.hp, enemy.is_dead());
    }

    // 检查 get_single 是否工作
    let mut enemy_query = world.query::<&Enemy>();
    match enemy_query.get_single(world) {
        Ok(enemy) => println!("get_single 成功: HP={}, is_dead={}", enemy.hp, enemy.is_dead()),
        Err(e) => println!("get_single 失败: {:?}", e),
    }

    // 再次检查
    drop(world);
    let particle_count = count_particles(&mut app);
    println!("粒子数量: {}", particle_count);
}
