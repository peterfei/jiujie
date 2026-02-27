use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use bevy_card_battler::components::particle::{EffectType, SpawnEffectEvent, Particle};
use bevy_card_battler::systems::gpu_particle::{GpuParticlePlugin, GpuParticleAssets};
use bevy_card_battler::systems::particle::{ParticlePlugin, ParticleAssets};
use bevy_card_battler::states::GameState;

#[test]
fn test_cloud_mist_gpu_migration_final() {
    let mut app = App::new();
    
    app.add_plugins(MinimalPlugins);
    
    // 1. 初始化所有必要的资产存储
    app.init_asset::<EffectAsset>();
    app.init_asset::<Image>();
    app.init_asset::<Mesh>();
    app.init_asset::<StandardMaterial>();
    
    // 2. 注入模拟状态和事件
    app.insert_resource(State::new(GameState::Combat));
    app.add_event::<SpawnEffectEvent>();
    app.add_event::<bevy_card_battler::components::screen_effect::ScreenEffectEvent>();

    // 3. 手动创建并注入 ParticleAssets 以避开文件 IO 和 AssetServer 缺失
    let mut images = app.world_mut().get_resource_mut::<Assets<Image>>().unwrap();
    let dummy_image = images.add(Image::default());
    let mut textures = std::collections::HashMap::new();
    textures.insert(EffectType::CloudMist, dummy_image.clone());
    app.insert_resource(ParticleAssets {
        textures,
        default_texture: dummy_image,
    });

    // 4. 添加我们要测试的插件
    // 我们手动调用 setup 系统，而不是依赖 Startup，因为 MinimalPlugins 调度不同
    app.add_plugins(GpuParticlePlugin);
    
    // 显式添加 ParticlePlugin 需要的系统，但不添加整个插件（因为它会尝试 Startup 加载贴图）
    app.add_systems(Update, bevy_card_battler::systems::particle::handle_effect_events);

    // 5. 运行初始化 (setup_gpu_effects 是 OnEnter)
    // 模拟进入 Combat 状态
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    
    // 运行几帧
    app.update(); 

    // 6. 验证 CloudMist 是否已注册到 GPU 资产中
    let gpu_assets = app.world().get_resource::<GpuParticleAssets>()
        .expect("GpuParticleAssets resource missing");
    
    assert!(gpu_assets.effects.contains_key(&EffectType::CloudMist), 
        "CloudMist should be registered in GPU effects");

    // 7. 发送 CloudMist 事件
    app.world_mut().send_event(SpawnEffectEvent::new(EffectType::CloudMist, Vec3::ZERO));
    
    // 运行一帧
    app.update();

    // 8. 验证是否生成了 GPU 粒子效果组件 (ParticleEffect)
    let mut gpu_effect_query = app.world_mut().query::<&ParticleEffect>();
    let gpu_effect_count = gpu_effect_query.iter(app.world()).count();
    assert!(gpu_effect_count > 0, "Should have spawned a GPU ParticleEffect for CloudMist");

    // 9. 验证是否**没有**生成 CPU 粒子实体 (Particle)
    let mut cpu_particle_query = app.world_mut().query::<&Particle>();
    let cpu_particle_count = cpu_particle_query.iter(app.world()).count();
    assert_eq!(cpu_particle_count, 0, "Should NOT spawn CPU Particle entities for CloudMist");
}
