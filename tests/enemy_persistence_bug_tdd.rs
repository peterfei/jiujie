use bevy::prelude::*;
use bevy_card_battler::components::combat::{Enemy};
use bevy_card_battler::components::{Player, PlayerDeck};
use bevy_card_battler::plugins::cleanup_combat_ui;

#[test]
fn test_enemy_entities_persist_after_cleanup_bug() {
    let mut app = App::new();
    app.insert_resource(Player::default());
    app.init_resource::<PlayerDeck>();
    app.add_plugins(MinimalPlugins);
    
    // 1. 准备环境
    app.insert_resource(PlayerDeck::new());
    
    // 2. 生成一个敌人
    let enemy_ent = app.world_mut().spawn(Enemy::new(1, "旧敌人", 100)).id();
    
    // 3. 运行目前的清理系统
    // 注意：我们必须手动调用它，或者模拟 OnExit(Combat)
    app.add_systems(Update, cleanup_combat_ui);
    app.update();

    // 4. 验证：旧敌人应该被销毁了
    let enemy_still_exists = app.world().get_entity(enemy_ent).is_ok();
    assert!(!enemy_still_exists, "修复验证：旧敌人实体已被彻底清理");
    
    println!("✅ 验证成功：Enemy 实体残留 Bug 已根除");
}
