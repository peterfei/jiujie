use bevy::prelude::*;
use bevy_card_battler::components::combat::{Enemy, EnemyStatusUi};

#[test]
fn test_changed_filter_causes_ui_genocide() {
    let mut app = App::new();
    
    let enemy_ent = app.world_mut().spawn(Enemy::new(1, "幸存者", 100)).id();
    let ui_ent = app.world_mut().spawn(EnemyStatusUi { owner: enemy_ent }).id();

    // 1. 模拟错误的清理系统 (使用了 Changed 过滤器)
    fn flawed_cleanup_system(
        mut commands: Commands,
        enemy_query: Query<&Enemy, Changed<Enemy>>, 
        ui_query: Query<(Entity, &EnemyStatusUi)>,
    ) {
        for (ui_entity, status_ui) in ui_query.iter() {
            let mut found = false;
            if enemy_query.get(status_ui.owner).is_ok() {
                found = true; 
            }
            if !found {
                commands.entity(ui_entity).despawn_recursive();
            }
        }
    }

    app.add_systems(Update, flawed_cleanup_system);

    // 2. 第一帧：刚生成，Enemy 是 Changed 的
    app.update(); 
    // 此时 UI 应该还在，因为第一帧 Enemy 被视为 Changed
    assert!(app.world().get_entity(ui_ent).is_ok(), "第一帧 UI 幸存");

    // 3. 第二帧：Enemy 没有变化
    app.update();
    // 此时 enemy_query 为空，UI 被误杀
    assert!(app.world().get_entity(ui_ent).is_err(), "Bug 复现：第二帧因无变化，UI 被误杀");
    
    println!("✅ 成功复现：Changed 过滤器导致静态敌人被误判死亡");
}

#[test]
fn test_without_changed_filter_is_safe() {
    let mut app = App::new();
    let enemy_ent = app.world_mut().spawn(Enemy::new(1, "幸存者", 100)).id();
    let ui_ent = app.world_mut().spawn(EnemyStatusUi { owner: enemy_ent }).id();

    // 2. 修正后的清理系统 (移除 Changed)
    fn correct_cleanup_system(
        mut commands: Commands,
        enemy_query: Query<&Enemy>, 
        ui_query: Query<(Entity, &EnemyStatusUi)>,
    ) {
        for (ui_entity, status_ui) in ui_query.iter() {
            if enemy_query.get(status_ui.owner).is_err() {
                commands.entity(ui_entity).despawn_recursive();
            }
        }
    }

    app.add_systems(Update, correct_cleanup_system);

    // 运行两帧
    app.update();
    app.update();

    // 验证 UI 依然健在
    assert!(app.world().get_entity(ui_ent).is_ok(), "修正后：UI 在静止状态下依然存活");
    println!("✅ 验证通过：移除 Changed 过滤器可保护 UI");
}