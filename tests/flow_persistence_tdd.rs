use bevy::prelude::*;
use bevy_card_battler::components::combat::{Player, PlayerDeck};
use bevy_card_battler::resources::save::GameStateSave;
use bevy_card_battler::components::cards::Card;
use bevy_card_battler::components::cultivation::Cultivation;
use bevy_card_battler::components::map::MapNode;

#[test]
fn test_physical_save_on_cleanup() {
    let mut app = App::new();
    
    // 1. 准备环境
    app.insert_resource(PlayerDeck::new());
    app.world_mut().spawn(Player { hp: 42, gold: 88, ..Default::default() });

    // 2. 模拟真实存档逻辑 (从 mod.rs 提取)
    let sync_and_save = |player_query: Query<&Player>, mut deck: ResMut<PlayerDeck>| {
        if let Ok(p) = player_query.get_single() {
            deck.update_from_player(p);
            
            // 构造 Save 对象并写入磁盘
            let save = GameStateSave {
                player: p.clone(),
                cultivation: Cultivation::new(),
                deck: deck.cards.clone(),
                relics: vec![],
                map_nodes: vec![],
                current_map_node_id: None,
                current_map_layer: 1,
            };
            let _ = save.save_to_disk();
        }
    };

    app.add_systems(Update, sync_and_save);
    app.update();

    // 3. 验证磁盘文件是否存在且内容正确
    assert!(GameStateSave::exists(), "存档文件应该被物理创建");
    let loaded = GameStateSave::load_from_disk().unwrap();
    assert_eq!(loaded.player.hp, 42, "读档后的 HP 应为 42");
    assert_eq!(loaded.player.gold, 88, "读档后的金石应为 88");

    println!("✅ 跨场景存档物理流转 TDD 验证通过");
    
    // 清理测试现场
    GameStateSave::delete_save();
}

#[test]
fn test_restart_clears_physical_save() {
    // 1. 先造一个假存档
    let save = GameStateSave {
        player: Player::default(),
        cultivation: Cultivation::new(),
        deck: vec![],
        relics: vec![],
        map_nodes: vec![],
        current_map_node_id: None,
        current_map_layer: 99,
    };
    save.save_to_disk().unwrap();
    assert!(GameStateSave::exists());

    // 2. 模拟重启逻辑 (调用物理删除)
    GameStateSave::delete_save();

    // 3. 验证
    assert!(!GameStateSave::exists(), "重新开始后，旧存档物理文件必须被彻底删除");
    println!("✅ 游戏重置物理清理 TDD 验证通过");
}