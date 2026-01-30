use bevy::prelude::*;
use bevy_card_battler::components::particle::{EffectType, SpawnEffectEvent};
use bevy_card_battler::states::GameState;

#[path = "test_utils.rs"]
mod test_utils;
use test_utils::*;

#[test]
fn test_combat_high_load_particle_stability() {
    let mut app = create_test_app();
    
    // 1. 进入战斗状态
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.update();
    
    // 2. 模拟触发极高频率的粒子爆发（万剑归宗 x 3）
    for _ in 0..3 {
        app.world_mut().send_event(SpawnEffectEvent::new(
            EffectType::WanJian, 
            Vec3::new(0.0, 0.0, 0.5)
        ).burst(100)); // 每次迸发100个
    }
    
    // 3. 运行多帧，模拟高负载渲染
    for _ in 0..10 {
        app.update();
    }
    
    // 4. 验证粒子数量上限及清理逻辑
    let particle_count = count_particles(&mut app);
    assert!(particle_count > 0, "应生成大量粒子实体");
    assert!(particle_count <= 500, "粒子数量不应无限膨胀"); // 检查是否有基本的内存保护
    
    info!("高负载稳定性测试通过：当前粒子实体数 {}", particle_count);
}

#[test]
fn test_asset_prewarming_exists() {
    let mut app = create_test_app();
    app.update();
    
    // 验证 CharacterAssets 资源是否已在 Startup 阶段注入
    use bevy_card_battler::components::sprite::CharacterAssets;
    assert!(app.world().get_resource::<CharacterAssets>().is_some(), "资产预热资源应已就绪");
}
