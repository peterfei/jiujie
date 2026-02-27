use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use bevy_card_battler::components::particle::{EffectType, SpawnEffectEvent, Particle};
use bevy_card_battler::systems::gpu_particle::{GpuParticlePlugin, GpuParticleAssets};
use bevy_card_battler::systems::vfx_orchestrator::{VfxOrchestratorPlugin, ParticleAssets};
use bevy_card_battler::states::GameState;

#[test]
fn test_vfx_migration_verification() {
    let mut app = App::new();
    
    // 最小化环境，不启动 IO
    app.add_plugins(MinimalPlugins);
    
    // 初始化资产存储
    app.init_asset::<EffectAsset>();
    app.init_asset::<Image>();
    app.init_asset::<Mesh>();
    app.init_asset::<StandardMaterial>();
    
    // 注入模拟状态和事件
    app.insert_resource(State::new(GameState::Combat));
    app.add_event::<SpawnEffectEvent>();
    app.add_event::<bevy_card_battler::components::screen_effect::ScreenEffectEvent>();

    // 注入必要的 Mock 资源以满足 handle_vfx_events 系统签名
    let mut images = app.world_mut().get_resource_mut::<Assets<Image>>().unwrap();
    let dummy_image = images.add(Image::default());
    let mut textures = std::collections::HashMap::new();
    textures.insert(EffectType::CloudMist, dummy_image.clone());
    app.insert_resource(ParticleAssets {
        textures,
        default_texture: dummy_image,
    });

    // 添加我们要测试的插件
    app.add_plugins(GpuParticlePlugin);
    
    // 手动添加编排器处理逻辑系统
    app.add_systems(Update, bevy_card_battler::systems::vfx_orchestrator::handle_vfx_events);

    // 触发初始化 (OnEnter GameState::Combat)
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    
    // 运行逻辑
    app.update(); 

    // --- 断言 1: 资产注册验证 ---
    let gpu_assets = app.world().get_resource::<GpuParticleAssets>()
        .expect("GpuParticleAssets resource missing");
    
    assert!(gpu_assets.effects.contains_key(&EffectType::CloudMist), "CloudMist should be in GPU assets");
    assert!(gpu_assets.effects.contains_key(&EffectType::WolfSlash), "WolfSlash should be in GPU assets");

    // --- 断言 2: 行为分发验证 ---
    // 发送一个本该由 GPU 处理的事件
    app.world_mut().send_event(SpawnEffectEvent::new(EffectType::CloudMist, Vec3::ZERO));
    app.update();

    // 验证：产生了 GPU 组件
    let mut gpu_query = app.world_mut().query::<&ParticleEffect>();
    assert!(gpu_query.iter(app.world()).count() > 0, "Should spawn GPU effect");

    // 验证：没有产生 CPU 粒子实体
    let mut cpu_query = app.world_mut().query::<&Particle>();
    assert_eq!(cpu_query.iter(app.world()).count(), 0, "Should NOT spawn CPU particle");

    // --- 断言 3: 复杂逻辑验证 (万剑归宗) ---
    // 发送万剑归宗事件 (Orchestrator 应该处理它)
    app.world_mut().send_event(SpawnEffectEvent::new(EffectType::WanJian, Vec3::ZERO).burst(10));
    app.update();

    // 验证：万剑归宗产生了 CPU 粒子实体（因为编排器需要它们来跑状态机）
    let mut wanjian_query = app.world_mut().query::<&Particle>();
    assert!(wanjian_query.iter(app.world()).count() >= 10, "WanJian SHOULD spawn CPU entities for orchestration");
}
