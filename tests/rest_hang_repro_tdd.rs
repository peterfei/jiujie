use bevy::prelude::*;
use bevy_card_battler::states::GameState;
use bevy_card_battler::components::*;
use bevy_card_battler::components::map::MapProgress;
use bevy_card_battler::plugins::GamePlugin;
use bevy_card_battler::systems::rest::{LeaveButton, handle_leave_interaction};

#[test]
fn test_rest_node_completion_on_leave() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.add_plugins(bevy::audio::AudioPlugin::default());
    app.add_plugins(bevy::render::texture::ImagePlugin::default());
    app.add_plugins(bevy::text::TextPlugin::default());
    app.init_asset::<TextureAtlasLayout>(); // 必须添加
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.add_plugins(bevy::input::InputPlugin::default());
    app.add_event::<bevy::input::mouse::MouseWheel>(); // 必须添加
    app.add_plugins(GamePlugin);
    
    // 初始化必要资源
    app.init_resource::<NextState<GameState>>();
    app.insert_resource(State::new(GameState::Rest));
    
    // 模拟实体
    app.world_mut().spawn((
        Player {
            hp: 10,
            max_hp: 80,
            gold: 100,
            energy: 3,
            max_energy: 3,
            block: 0,
            turn: 1,
            vulnerable: 0,
            poison: 0,
            weakness: 0,
        }, 
        Cultivation::new()
    ));
    app.insert_resource(PlayerDeck::default());
    app.insert_resource(RelicCollection::default());
    
    // 模拟点击离开
    app.world_mut().spawn((Interaction::Pressed, LeaveButton));
    
    // 手动添加我们要测试的系统（防止插件顺序导致的系统未运行）
    app.add_systems(Update, handle_leave_interaction);
    
    app.update();
    
    // 验证状态转换是否触发（通常在下一帧生效）
    app.update(); 
    
    let current_state = app.world().resource::<State<GameState>>().get();
    assert_eq!(*current_state, GameState::Map, "离开休息节点后应返回地图");
}
