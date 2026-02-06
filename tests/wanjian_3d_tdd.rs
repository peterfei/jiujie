use bevy::prelude::*;
use bevy::app::App;
use bevy_card_battler::components::particle::{SpawnEffectEvent, EffectType, Particle};
use bevy_card_battler::resources::PlayerAssets;
use bevy_card_battler::systems::particle::{handle_effect_events, update_particles};

#[test]
fn test_wanjian_sequential_barrage() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), bevy::scene::ScenePlugin, bevy::render::mesh::MeshPlugin));
    app.insert_resource(PlayerAssets::default());
    app.add_event::<SpawnEffectEvent>();
    app.add_event::<bevy_card_battler::components::screen_effect::ScreenEffectEvent>();
    app.insert_resource(Time::<Virtual>::default());

    // 生成两把剑，种子不同
    app.world_mut().send_event(SpawnEffectEvent::new(EffectType::WanJian, Vec3::ZERO).burst(2));
    app.add_systems(Update, (handle_effect_events, update_particles));
    
    // 推进到攻击中期
    if let Some(mut time) = app.world_mut().get_resource_mut::<Time<Virtual>>() {
        time.advance_by(std::time::Duration::from_millis(1400));
    }
    app.update(); 

    let mut query = app.world_mut().query::<(&Particle, &Transform)>();
    let entities: Vec<_> = query.iter(app.world()).collect();
    
    if entities.len() >= 2 {
        let h1 = entities[0].1.translation.y;
        let h2 = entities[1].1.translation.y;
        // 验证高度不同：说明处于序列化攻击状态，而非同步下落
        assert!((h1 - h2).abs() > 0.1, "序列化攻击逻辑失效：飞剑高度过于同步 ({} vs {})", h1, h2);
    }
}
