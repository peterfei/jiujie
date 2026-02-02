use bevy::prelude::*;
use bevy::ecs::system::RunSystemOnce;
use bevy_card_battler::components::particle::{SpawnEffectEvent, EffectType};
use bevy_card_battler::systems::particle::{handle_effect_events, ParticleAssets};

#[test]
fn test_vfx_lightning_creates_point_light() {
    let mut app = App::new();
    
    // 1. 初始化基础资源
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.add_event::<SpawnEffectEvent>();
    app.insert_resource(ParticleAssets {
        textures: std::collections::HashMap::new(),
        default_texture: Handle::default(),
    });

    // 2. 模拟发送雷击事件
    let target_pos = Vec3::new(0.0, 0.0, 0.0);
    app.world_mut().resource_mut::<Events<SpawnEffectEvent>>().send(
        SpawnEffectEvent::new(EffectType::Lightning, target_pos)
    );

    // 3. 运行效果处理系统
    let _ = app.world_mut().run_system_once(handle_effect_events);

    // 4. 验证：场景中是否生成了 PointLight 组件
    let mut query = app.world_mut().query::<&PointLight>();
    let light_count = query.iter(app.world()).count();
    
    println!("检测到的动态光源数量: {}", light_count);
    
    // 预期：程序化闪电至少会产生一个光源来照亮环境
    assert!(light_count > 0, "雷击特效应同步产生 PointLight 以照亮 3D 场景");
    
    // 5. 验证：光源的颜色应为闪电色（偏蓝紫）
    if let Some(light) = query.iter(app.world()).next() {
        assert!(light.color.to_srgba().blue > 0.5, "闪电光源应具有蓝紫色调");
    }
}
