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

    // 发送闪电事件
    app.world_mut().send_event(SpawnEffectEvent::new(EffectType::Lightning, Vec3::ZERO));
    app.update();

    let mut main_trunk_count = 0;
    let mut max_horizontal_spread: f32 = 0.0;

    let mut query = app.world_mut().query::<(&LightningBolt, &Transform)>();
    for (bolt, transform) in query.iter(app.world()) {
        if bolt.is_light { continue; }
        if bolt.branch_level == 0 {
            main_trunk_count += 1;
            // 计算横向扩散（远离中心线）
            let spread = Vec2::new(transform.translation.x, transform.translation.z).length();
            max_horizontal_spread = max_horizontal_spread.max(spread);
        }
    }

    // TDD 红灯断言
    assert!(main_trunk_count >= 15, "AAA lightning needs detail");
    
    // 物理限制：闪电主干横向偏移不能超过 4.0。当前代码由于 0.45 系数会远超此值。
    assert!(max_horizontal_spread < 4.0, "Lightning path is TOO HORIZONTAL! Spread: {}", max_horizontal_spread);
}
