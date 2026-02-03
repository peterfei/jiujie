pub mod test_utils;
use crate::test_utils::*;
use bevy::prelude::*;
use bevy_card_battler::states::GameState;
use bevy_card_battler::components::map::{MapNodeButton, MapNode, NodeType, MapProgress};
use bevy_card_battler::components::{Player, Cultivation};
use bevy_card_battler::components::cultivation::Realm;

#[test]
fn reproduce_map_layout_and_interaction() {
    let mut app = create_test_app();
    
    // 1. 初始化玩家 (炼气期，视野应该小)
    let mut cultivation = Cultivation::new();
    cultivation.realm = Realm::QiRefining;
    app.world_mut().spawn((Player::default(), cultivation));

    // 2. 初始化地图 (10层)
    let mut nodes = Vec::new();
    let layers = 10;
    let mut id_counter = 0;
    
    for layer in 0..layers {
        for _ in 0..3 { // 每层3个
            nodes.push(MapNode {
                id: id_counter,
                node_type: if layer == 9 { NodeType::Boss } else { NodeType::Normal },
                position: (layer as i32, 0),
                unlocked: layer == 0, // 只有第0层解锁
                completed: false,
                next_nodes: vec![],
            });
            id_counter += 1;
        }
    }
    
    app.insert_resource(MapProgress {
        nodes,
        current_node_id: None, // 初始状态
        current_layer: 0,
        game_completed: false,
    });

    // 3. 进入地图
    transition_to_state(&mut app, GameState::Map);
    app.update();
    app.update(); // 运行多帧确保 UI 生成

    // 4. 验证节点生成情况
    let mut layer_0_count = 0;
    let mut layer_1_count = 0;
    let mut layer_2_count = 0; // 应该被剔除
    let mut boss_layer_count = 0;

    let nodes = app.world().resource::<MapProgress>().nodes.clone();

    {
        let world = app.world_mut();
        let mut button_query = world.query::<(&MapNodeButton, &Parent)>(); 
        
        for (btn, _) in button_query.iter(world) {
            if let Some(node) = nodes.iter().find(|n| n.id == btn.node_id) {
                match node.position.0 {
                    0 => layer_0_count += 1,
                    1 => layer_1_count += 1,
                    2 => layer_2_count += 1,
                    9 => boss_layer_count += 1,
                    _ => {}
                }
            }
        }
    }

    println!("Layer 0 buttons: {}", layer_0_count);
    println!("Layer 1 buttons: {}", layer_1_count);
    println!("Layer 2 buttons: {}", layer_2_count);
    println!("Boss layer buttons: {}", boss_layer_count);

    assert!(layer_0_count > 0, "Layer 0 should be visible");
    assert!(layer_1_count > 0, "Layer 1 should be visible (QiRefining vision=1, range 0..=1)");
    assert_eq!(layer_2_count, 0, "Layer 2 should be culled (vision=1)");
    assert!(boss_layer_count > 0, "Boss layer should always be visible");

    // 5. 验证点击交互
    let btn_id = nodes.iter().find(|n| n.position.0 == 0).expect("No layer 0 node").id;
    
    let layer_0_btn_entity = app.world_mut().query::<(Entity, &MapNodeButton)>()
        .iter(app.world())
        .find(|(_, btn)| btn.node_id == btn_id)
        .map(|(e, _)| e)
        .expect("No layer 0 button entity found");

    println!("Simulating click on node layer 0");
    *app.world_mut().get_mut::<Interaction>(layer_0_btn_entity).unwrap() = Interaction::Pressed;
    app.update(); // 系统运行，设置 NextState
    app.update(); // 应用状态转换
    
    let current_state = get_current_state(&app);
    assert_eq!(current_state, GameState::Combat, "Should enter Combat after clicking unlocked node");
}
