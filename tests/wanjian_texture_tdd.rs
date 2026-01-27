use bevy::prelude::*;
use bevy_card_battler::components::particle::*;
use bevy_card_battler::systems::particle::*;
use bevy_card_battler::states::GameState;

#[test]
fn test_wanjian_uses_sword_texture() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        ImagePlugin::default(),
        bevy::state::app::StatesPlugin::default(),
    ));
    
    app.insert_state(GameState::Combat);
    app.add_event::<bevy_card_battler::components::screen_effect::ScreenEffectEvent>();
    app.add_plugins(ParticlePlugin);

    // 模拟发送万剑归宗事件
    app.world_mut().send_event(SpawnEffectEvent::new(EffectType::WanJian, Vec3::ZERO).burst(1));
    
    // 运行一帧以处理事件和生成粒子
    app.update();

    // 检查是否生成了带有 ImageNode 的实体
    let mut query = app.world_mut().query_filtered::<(&Node, &ImageNode), With<Particle>>();
    let (node, image_node) = query.get_single(app.world()).expect("Should have spawned one particle");
    
    // 获取粒子资源
    let assets = app.world().get_resource::<ParticleAssets>().expect("ParticleAssets resource should exist");
    let sword_handle = assets.textures.get(&EffectType::WanJian).expect("WanJian texture should be registered");

    assert_eq!(image_node.image.id(), sword_handle.id(), "WanJian particle should use the sword texture handle");

    // 检查尺寸比例
    if let Val::Px(width) = node.width {
        if let Val::Px(height) = node.height {
            assert!(height > width, "WanJian sword should be longer than it is wide (height: {}, width: {})", height, width);
        } else {
            panic!("Height should be Val::Px");
        }
    } else {
        panic!("Width should be Val::Px");
    }
}
