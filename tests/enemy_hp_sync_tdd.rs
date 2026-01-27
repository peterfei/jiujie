use bevy::prelude::*;
use bevy_card_battler::components::combat::{Enemy, EnemyHpText, EnemyIntentText, Player, PlayerHpText, PlayerEnergyText, PlayerBlockText, TopBarHpText, TopBarGoldText, EnemyIntent};

// 模拟【修复冲突后】的新同步逻辑：使用统一的 ParamSet
fn mock_fixed_sync_system(
    enemy_query: Query<&Enemy>,
    mut text_queries: ParamSet<(
        Query<(&EnemyHpText, &mut Text)>,
        Query<(&EnemyIntentText, &mut Text)>,
    )>,
) {
    // 1. 同步更新所有敌人的 HP
    for (marker, mut text) in text_queries.p0().iter_mut() {
        if let Ok(enemy) = enemy_query.get(marker.owner) {
            text.0 = format!("HP: {}/{}", enemy.hp, enemy.max_hp);
        }
    }
}

#[test]
fn test_hp_sync_works_with_param_set() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    let enemy_id = 1;
    let enemy_ent = app.world_mut().spawn(Enemy::new(enemy_id, "测试怪", 100)).id();

    // 构建嵌套 UI 树
    let row = app.world_mut().spawn(Node::default()).id();
    let text_ent = app.world_mut().spawn((
        Text::new("HP: 100/100"), 
        EnemyHpText { owner: enemy_ent } 
    )).id();
    app.world_mut().entity_mut(row).add_child(text_ent);

    // 3. 修改 HP
    if let Some(mut e) = app.world_mut().get_mut::<Enemy>(enemy_ent) { e.hp = 50; }

    // 4. 运行系统
    app.add_systems(Update, mock_fixed_sync_system);
    app.update();

    // 5. 验证结果
    let text = app.world().get::<Text>(text_ent).unwrap();
    assert!(text.0.contains("50"), "使用 ParamSet 后，HP 应该能正常更新！当前: {}", text.0);
    println!("✅ 验证通过：ParamSet 成功解决了查询冲突并实现了数值同步");
}
