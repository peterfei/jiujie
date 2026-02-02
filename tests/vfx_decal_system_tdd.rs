use bevy::prelude::*;
use bevy::ecs::system::RunSystemOnce;
use bevy_card_battler::components::particle::{SpawnEffectEvent, EffectType};
use bevy_card_battler::systems::particle::{handle_effect_events, ParticleAssets};

#[derive(Component)]
struct DecalMarker;

#[test]
fn test_vfx_lightning_creates_ground_decal() {
    let mut app = App::new();
    
    // 1. 初始化
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.add_event::<SpawnEffectEvent>();
    app.insert_resource(ParticleAssets {
        textures: std::collections::HashMap::new(),
        default_texture: Handle::default(),
    });

    // 2. 发送雷击事件
    let target_pos = Vec3::new(5.0, 0.0, 5.0); // 击中地面某个点
    app.world_mut().resource_mut::<Events<SpawnEffectEvent>>().send(
        SpawnEffectEvent::new(EffectType::Lightning, target_pos)
    );

    // 3. 运行系统
    let _ = app.world_mut().run_system_once(handle_effect_events);

    // 4. 验证：是否生成了带有残痕特征的实体
    // 注意：目前我们还没定义 DecalMarker，测试会先验证是否存在某种“地面贴图”实体
    let mut query = app.world_mut().query_filtered::<&Transform, With<bevy_card_battler::components::particle::ParticleMarker>>();
    
    // 我们预期残痕是一个平贴在地面(Y=0或极低)的实体
    let decal_exists = query.iter(app.world()).any(|ts| {
        (ts.translation.y - 0.0).abs() < 0.1 && (ts.translation.x - 5.0).abs() < 0.1
    });

    assert!(decal_exists, "雷击应在地面坐标(5,0,5)附近留下残痕实体");
}
