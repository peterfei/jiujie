use bevy::prelude::*;
use bevy_card_battler::components::particle::{EffectType, SpawnEffectEvent, LightningBolt};
use bevy_card_battler::systems::vfx_orchestrator::{ParticleAssets};

#[test]
fn test_realistic_fractal_lightning_logic() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(Assets::<Image>::default());
    app.insert_resource(Assets::<Mesh>::default());
    app.insert_resource(Assets::<StandardMaterial>::default());
    app.add_event::<bevy_card_battler::components::screen_effect::ScreenEffectEvent>();
    app.add_event::<SpawnEffectEvent>();
    
    let mut images = app.world_mut().get_resource_mut::<Assets<Image>>().unwrap();
    let dummy_image = images.add(Image::default());
    let mut textures = std::collections::HashMap::new();
    textures.insert(EffectType::Lightning, dummy_image.clone());
    app.insert_resource(ParticleAssets { textures, default_texture: dummy_image });

    app.add_systems(Update, bevy_card_battler::systems::vfx_orchestrator::handle_vfx_events);
    app.update(); 

    app.world_mut().send_event(SpawnEffectEvent::new(EffectType::Lightning, Vec3::ZERO));
    app.update();

    let mut trunk_segments: Vec<(f32, f32)> = Vec::new(); // (y_pos, radius_scale)

    let mut query = app.world_mut().query::<(&LightningBolt, &Transform)>();
    for (bolt, transform) in query.iter(app.world()) {
        if bolt.is_light || bolt.branch_level > 0 { continue; }
        trunk_segments.push((transform.translation.y, transform.scale.x));
    }

    // 按高度排序（从高到低）
    trunk_segments.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    // TDD 断言 1：段数
    assert!(trunk_segments.len() >= 15, "AAA lightning needs detail");
    
    // TDD 断言 2：渐变（Tapering）。顶部的粗度必须大于底部的粗度。
    let top_radius = trunk_segments.first().unwrap().1;
    let bottom_radius = trunk_segments.last().unwrap().1;
    assert!(top_radius > bottom_radius, "Lightning must taper! Top: {}, Bottom: {}", top_radius, bottom_radius);
}
