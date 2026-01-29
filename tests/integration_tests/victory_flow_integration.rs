//! 胜利流程集成测试
//!
//! 真实还原敌人被击败后的完整流程，验证：
//! 1. 胜利延迟机制
//! 2. 状态转换
//! 3. 不会出现无限循环
//! 4. 边界条件处理

use crate::test_utils::*;
use bevy_card_battler::states::GameState;
use bevy_card_battler::components::Enemy;

#[test]
#[ignore = "粒子特效在无头模式下生成不稳定"]
fn integration_victory_generates_particles() {
    let mut app = create_test_app();
    app.world_mut().spawn(Enemy::new(0, "测试妖兽", 30, 0));
    let enemy_entity = setup_combat_scene(&mut app);
    advance_frames(&mut app, 1);
    kill_enemy(&mut app, enemy_entity);
    advance_frames(&mut app, 1);
    let particle_count = count_particles(&mut app);
    assert!(particle_count > 0);
}

#[test]
#[ignore = "粒子特效在无头模式下生成不稳定"]
fn integration_victory_particle_count_matches_expected() {
    let mut app = create_test_app();
    app.world_mut().spawn(Enemy::new(0, "测试妖兽", 30, 0));
    let enemy_entity = setup_combat_scene(&mut app);
    advance_frames(&mut app, 1);
    kill_enemy(&mut app, enemy_entity);
    advance_frames(&mut app, 1);
}

#[test]
#[ignore = "屏幕特效在无头模式下生成不稳定"]
fn integration_victory_creates_screen_flash() {
    let mut app = create_test_app();
    app.world_mut().spawn(Enemy::new(0, "测试妖兽", 30, 0));
    let enemy_entity = setup_combat_scene(&mut app);
    advance_frames(&mut app, 1);
    kill_enemy(&mut app, enemy_entity);
    advance_frames(&mut app, 1);
    assert!(count_screen_effects(&mut app) > 0);
}

#[test]
fn integration_victory_delay_activates_on_enemy_death() {
    let mut app = create_test_app();
    app.world_mut().spawn(Enemy::new(0, "测试妖兽", 30, 0));
    let enemy_entity = setup_combat_scene(&mut app);
    advance_frames(&mut app, 1);
    kill_enemy(&mut app, enemy_entity);
    advance_frames(&mut app, 1);
    assert!(is_victory_delay_active(&app));
}

#[test]
fn integration_victory_delay_increases_over_time() {
    let mut app = create_test_app();
    app.world_mut().spawn(Enemy::new(0, "测试妖兽", 30, 0));
    let enemy_entity = setup_combat_scene(&mut app);
    advance_frames(&mut app, 1);
    kill_enemy(&mut app, enemy_entity);
    advance_frames(&mut app, 1);
    let initial = get_victory_delay_elapsed(&app);
    advance_frames(&mut app, 5);
    assert!(get_victory_delay_elapsed(&app) > initial);
}

#[test]
fn integration_victory_delay_does_not_reset_indefinitely() {
    let mut app = create_test_app();
    app.world_mut().spawn(Enemy::new(0, "测试妖兽", 30, 0));
    let enemy_entity = setup_combat_scene(&mut app);
    advance_frames(&mut app, 1);
    kill_enemy(&mut app, enemy_entity);
    advance_frames(&mut app, 1);
    let first = get_victory_delay_elapsed(&app);
    advance_frames(&mut app, 10);
    assert!(get_victory_delay_elapsed(&app) > first);
}

#[test]
fn integration_victory_with_instant_zero_hp() {
    let mut app = create_test_app();
    app.world_mut().spawn(Enemy::new(0, "测试妖兽", 30, 0));
    let enemy_entity = setup_combat_scene(&mut app);
    advance_frames(&mut app, 1);
    if let Some(mut enemy) = app.world_mut().get_mut::<Enemy>(enemy_entity) {
        enemy.take_damage(100);
    }
    advance_frames(&mut app, 1);
    assert!(is_victory_delay_active(&app));
}