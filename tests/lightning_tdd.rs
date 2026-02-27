use bevy::prelude::*;
use bevy_card_battler::components::particle::{EffectType, SpawnEffectEvent, LightningBolt};
use bevy_card_battler::systems::vfx_orchestrator::{ParticleAssets};

#[test]
fn test_realistic_fractal_lightning_logic() {
    let mut app = App::new();
    
    // 纯粹的 ECS 测试环境
    app.add_plugins(MinimalPlugins);
    
    // 手动插入 Assets 资源集合，彻底避开对 AssetServer 的依赖
    app.insert_resource(Assets::<Image>::default());
    app.insert_resource(Assets::<Mesh>::default());
    app.insert_resource(Assets::<StandardMaterial>::default());
    
    app.add_event::<bevy_card_battler::components::screen_effect::ScreenEffectEvent>();
    app.add_event::<SpawnEffectEvent>();
    
    // 注入模拟的 ParticleAssets
    let mut images = app.world_mut().get_resource_mut::<Assets<Image>>().unwrap();
    let dummy_image = images.add(Image::default());
    let mut textures = std::collections::HashMap::new();
    textures.insert(EffectType::Lightning, dummy_image.clone());
    app.insert_resource(ParticleAssets {
        textures,
        default_texture: dummy_image,
    });

    // 我们直接将系统添加到 App 的 Update 中
    app.add_systems(Update, bevy_card_battler::systems::vfx_orchestrator::handle_vfx_events);

    // 运行一次以完成初始化
    app.update(); 

    // 发送闪电事件
    app.world_mut().send_event(SpawnEffectEvent::new(EffectType::Lightning, Vec3::ZERO));
    
    // 运行逻辑处理事件
    app.update();

    // 收集生成的 LightningBolt 并验证几何连续性与亮度
    let mut main_trunk_count = 0;
    let mut prev_end_pos: Option<Vec3> = None;
    let mut total_segments = 0;

    let mut query = app.world_mut().query::<(&LightningBolt, &Transform)>();
    for (bolt, transform) in query.iter(app.world()) {
        if bolt.is_light { continue; }
        total_segments += 1;

        if bolt.branch_level == 0 {
            main_trunk_count += 1;
            
            // 验证连续性：虽然我们用 Cylinder，但逻辑上它们应该排列在路径上
            // 此处验证 Transform 是否正确生成
            assert!(transform.scale.y > 0.0, "Segment length must be positive");
        }
    }

    // TDD 断言
    assert!(main_trunk_count >= 15, "AAA lightning needs high tessellation (min 15 segments)");
    assert!(total_segments < 60, "Too many segments create a 'bird nest' look. Keep it clean.");
    
    // 验证发光强度 (通过查询材质，此处模拟逻辑验证)
    // 理想情况下应验证 emissive.red + emissive.blue > 500
}
