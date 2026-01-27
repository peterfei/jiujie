use bevy::prelude::*;
use bevy_card_battler::components::{Enemy, Hand, DrawPile, DiscardPile};
use bevy_card_battler::components::cards::{Card, CardType, CardEffect, CardRarity};
use bevy_card_battler::components::particle::*;
use bevy_card_battler::systems::particle::*;
use bevy_card_battler::states::GameState;

#[test]
fn test_wanjian_aoe_spawns_with_correct_texture() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        ImagePlugin::default(),
        bevy::state::app::StatesPlugin::default(),
    ));
    app.init_asset::<Font>();
    app.insert_state(GameState::Combat);
    app.add_event::<bevy_card_battler::components::screen_effect::ScreenEffectEvent>();
    
    // 初始化粒子资源
    app.add_plugins(ParticlePlugin);

    // 模拟敌人
    app.world_mut().spawn((
        Enemy::new(1, "蜘蛛", 10),
        bevy_card_battler::components::sprite::EnemySpriteMarker { id: 1 },
        Transform::default(),
    ));

    // 模拟卡牌：万剑归宗
    let _wanjian_card = Card::new(
        999, "万剑归宗", "描述", 
        CardType::Skill, 3, 
        CardEffect::DealAoEDamage { amount: 10 }, 
        CardRarity::Rare, ""
    );

    // 我们需要模拟打出卡牌的逻辑，或者直接调用 apply_card_effect
    // 关键是检查执行后产生的粒子实体
    
    // 这里我们直接运行一帧以确保资产加载
    app.update();

    // 手动调用 apply_card_effect 的简化还原逻辑 (参照 src/plugins/mod.rs)
    // 注意：src 里的逻辑是硬编码 spawn 粒子的
    // 我们检查是否有 ZIndex(1) 且带有 Particle 组件的实体，其 ImageNode 是否为空
    
    // 发送万剑归宗事件
    app.world_mut().send_event(
        SpawnEffectEvent::new(EffectType::WanJian, Vec3::ZERO)
            .burst(1)
            .with_target(Vec2::new(100.0, 100.0))
    );
    
    // 运行一帧以处理事件
    app.update();

    // 检查产生的粒子
    let mut query = app.world_mut().query_filtered::<&ImageNode, With<Particle>>();
    let image_node = query.get_single(app.world()).expect("Should have spawned one particle via event");
    
    // 获取资源以进行对比
    let assets = app.world().get_resource::<ParticleAssets>().expect("ParticleAssets should exist");
    let expected_handle = assets.textures.get(&EffectType::WanJian).unwrap();

    assert_eq!(image_node.image.id(), expected_handle.id(), "WanJian AoE particles MUST use the texture handle from ParticleAssets");
}
