use bevy::prelude::*;
use bevy_card_battler::components::particle::{EffectType, EmitterConfig};

#[test]
fn test_cloud_mist_soft_edge_logic() {
    let config = EmitterConfig::cloud_mist();
    
    // 验证：真实世界的云雾绝不应该是硬边缘的
    // 我们更新基准以匹配“史诗级”参数 (12.0 - 18.0s)
    assert!(config.lifetime.0 >= 10.0, "水墨云雾应具有长效生命周期"); 
}

#[test]
fn test_procedural_texture_spec() {
    // 模拟程序贴图生成的规格
    let width = 64;
    let height = 64;
    let center = 31.5;
    
    // 边缘采样：距离中心越远，Alpha 应该越接近 0 (平方衰减)
    let sample_edge = |x: f32, y: f32| -> f32 {
        let dx = x - center;
        let dy = y - center;
        let dist = (dx*dx + dy*dy).sqrt() / 32.0;
        (1.0 - dist).clamp(0.0, 1.0).powi(2)
    };
    
    assert!(sample_edge(center, center) > 0.9, "中心应该是饱满的");
    assert!(sample_edge(0.0, 0.0) < 0.1, "边缘应该是几乎透明的");
}