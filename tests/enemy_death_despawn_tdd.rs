use bevy::prelude::*;
use bevy_card_battler::components::combat::{Enemy, EnemyStatusUi, EnemyHpText};

#[test]
fn test_ui_despawns_when_enemy_dies() {
    let mut app = App::new();
    
    // 1. 创建敌人和对应的 UI
    let enemy_ent = app.world_mut().spawn(Enemy::new(1, "僵尸怪", 100)).id();
    let ui_ent = app.world_mut().spawn(EnemyStatusUi { owner: enemy_ent }).id();
    
    // 手动关联：为了测试方便，我们假设 EnemyStatusUi 逻辑上关联了 enemy_ent
    // 在真实系统中，我们可能需要通过 enemy_id 查找，或者 UI 上有 owner 字段
    // 这里我们模拟 update_combat_ui 中的检查逻辑
    
    // 2. 模拟清理系统
    fn auto_cleanup_dead_enemy_ui(
        mut commands: Commands,
        enemy_query: Query<&Enemy>,
        // 假设我们通过某种方式知道 UI 对应哪个敌人，这里简化为通过 ID 匹配
        ui_query: Query<(Entity, &EnemyStatusUi)>, 
    ) {
        for (ui_entity, status_ui) in ui_query.iter() {
            // 查找对应的敌人
            let mut found = false;
            for enemy in enemy_query.iter() {
                if enemy.id == status_ui.enemy_id {
                    found = true;
                    if enemy.hp <= 0 {
                        commands.entity(ui_entity).despawn_recursive();
                    }
                }
            }
            // 如果连敌人都找不到了，说明敌人实体已经被 despawn 了，UI 也该走了
            if !found {
                commands.entity(ui_entity).despawn_recursive();
            }
        }
    }

    app.add_systems(Update, auto_cleanup_dead_enemy_ui);

    // 3. 场景 A: 敌人活着
    app.update();
    assert!(app.world().get_entity(ui_ent).is_ok(), "敌人活着时 UI 应存在");

    // 4. 场景 B: 敌人 HP 归零
    if let Some(mut e) = app.world_mut().get_mut::<Enemy>(enemy_ent) {
        e.hp = 0;
    }
    app.update();
    assert!(app.world().get_entity(ui_ent).is_err(), "敌人HP归零后 UI 应被销毁");

    println!("✅ 敌人死亡 UI 联动清理逻辑验证通过");
}