use bevy::prelude::*;
use bevy::ecs::system::RunSystemOnce;
use bevy_card_battler::components::{LightningBolt, CombatUiRoot};
use bevy_card_battler::components::particle::{SpawnEffectEvent, EffectType};
use bevy_card_battler::systems::particle::ParticleAssets;

#[test]
fn test_lightning_visual_system_integration() {
    let mut app = App::new();
    
    // 1. 初始化视觉系统所需的基础资源
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.add_event::<SpawnEffectEvent>();
    app.insert_resource(ParticleAssets {
        textures: std::collections::HashMap::new(),
        default_texture: Handle::default(),
    });

    // 2. 发送一个真实的“雷击”请求事件
    let target_pos = Vec3::new(2.0, 0.0, 0.0);
    app.world_mut().resource_mut::<Events<SpawnEffectEvent>>().send(
        SpawnEffectEvent::new(EffectType::Lightning, target_pos)
    );

    // 3. 手动运行效果处理系统 (handle_effect_events 会处理 Lightning 类型并调用 spawn_real_lightning)
    let _ = app.world_mut().run_system_once(bevy_card_battler::systems::particle::handle_effect_events);

    // 4. 核心验证：检查是否生成了折线闪电 Mesh 实体
    // 程序化闪电每条折线段都是一个带有 LightningBolt 组件的实体
    let mut query = app.world_mut().query::<&LightningBolt>();
    let bolt_count = query.iter(app.world()).count();
    
    println!("生成的闪电段数量: {}", bolt_count);
    assert!(bolt_count > 0, "视觉系统应根据 Lightning 事件生成程序化折线 Mesh 实体");

    // 5. 验证是否挂载了清理标记
    let mut root_query = app.world_mut().query::<&CombatUiRoot>();
    assert!(root_query.iter(app.world()).count() > 0, "生成的闪电实体应带有 CombatUiRoot 标记以便自动清理");
}