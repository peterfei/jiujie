// @Validated: Refactor Regression - 2026-01-29

use bevy::prelude::*;
use bevy_card_battler::components::combat::{Player};
use bevy_card_battler::components::PlayerDeck;
use bevy_card_battler::states::GameState;
use bevy_card_battler::resources::save::GameStateSave;
use bevy_card_battler::components::cultivation::Cultivation;

#[test]
fn test_data_persistence_flow() {
    let mut app = App::new();
    app.init_resource::<PlayerDeck>();
    app.insert_resource(Cultivation::new());
    app.insert_resource(Player::default());
    
    // 模拟数据修改
    fn sync_and_save(
        player_query: Query<&Player>, 
        mut player_res: ResMut<Player>,
    ) {
        if let Ok(p) = player_query.get_single() {
            *player_res = p.clone();
        }
    }

    app.add_systems(Update, sync_and_save);
    app.update();
    
    let player = app.world().resource::<Player>();
    let deck = app.world().resource::<PlayerDeck>();
    let cultivation = app.world().resource::<Cultivation>();
    
    let save = GameStateSave {
        player: player.clone(),
        cultivation: cultivation.clone(),
        deck: deck.cards.clone(),
        relics: Vec::new(),
        map_nodes: Vec::new(),
        current_map_node_id: None,
        current_map_layer: 0,
    };
    
    assert!(save.save_to_disk().is_ok());
    assert!(GameStateSave::exists());
}
