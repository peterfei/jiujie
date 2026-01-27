use bevy::prelude::*;
use bevy_card_battler::components::cards::Card;
use bevy_card_battler::components::particle::*;
use bevy_card_battler::systems::particle::*;
use bevy_card_battler::states::GameState;
use bevy_card_battler::plugins::{CombatUiRoot, HandCard};

#[test]
fn test_hand_cards_not_despawned_by_particles() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        ImagePlugin::default(),
        bevy::state::app::StatesPlugin::default(),
    ));
    
    app.insert_state(GameState::Combat);
    app.add_event::<SpawnEffectEvent>();
    app.add_event::<bevy_card_battler::components::screen_effect::ScreenEffectEvent>();
    app.add_plugins(ParticlePlugin);

    // 1. 创建手牌
    let card_id = app.world_mut().spawn((
        Node::default(),
        HandCard {
            card_id: 1,
            base_bottom: 0.0,
            base_rotation: 0.0,
            index: 0,
        },
        CombatUiRoot, // 手牌通常属于战斗UI根
    )).id();

    // 2. 触发万剑归宗
    app.world_mut().send_event(SpawnEffectEvent::new(EffectType::WanJian, Vec3::ZERO).burst(50));
    
    // 运行几帧让粒子生成并更新
    for _ in 0..10 {
        app.update();
    }

    // 3. 验证手牌是否还在
    assert!(app.world().get_entity(card_id).is_ok(), "Hand card should not be despawned during particle animation");

    // 4. 验证 ZIndex (关键假设：遮挡问题)
    let mut particle_query = app.world_mut().query_filtered::<&ZIndex, With<Particle>>();
    let particle_z = particle_query.iter(app.world()).next().expect("Should have particles").0;
    
    // 我们现在在 setup_combat_ui 中通过 HandArea 间接给手牌提供了层级，
    // 但在测试中我们直接创建了手牌并手动给了它 CombatUiRoot。
    // 为了模拟真实情况，我们检查 HandCard 的 ZIndex。
    // 注意：真实代码里是 HandArea 带有 ZIndex(50)，子节点默认继承或相对。
    // 这里我们直接测试 HandArea 的逻辑或直接修改测试实体。
    
    let card_z = app.world().get::<ZIndex>(card_id).map(|z| z.0).unwrap_or(0);
    
    println!("Card ZIndex: {}, Particle ZIndex: {}", card_z, particle_z);
    // 在真实场景中，HandArea(50) 包含了 HandCard。
    // 在这个简化测试中，我们验证粒子 ZIndex 已经从 1000 降到了 5。
    assert_eq!(particle_z, 5, "Particle ZIndex should be lowered to 5");
}
