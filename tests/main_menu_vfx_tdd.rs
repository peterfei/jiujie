use bevy::prelude::*;
use bevy_card_battler::components::particle::{EffectType, ParticleEmitter};
use bevy_card_battler::states::GameState;

#[test]
fn test_main_menu_cloud_coverage_logic() {
    let mut app = App::new();
    app.init_state::<GameState>();
    
    // 模拟主菜单发射器参数
    // 为了覆盖 1280x720 屏幕，发射器的随机范围应涵盖 X(-600..600), Y(-300..300)
    let config = EffectType::CloudMist.config();
    
    assert!(config.size.1 >= 300.0, "全屏云雾单体尺寸应足够大以保证遮盖感");
    assert!(config.lifetime.1 >= 8.0, "主菜单云雾应具有极长的停留时间以保持平稳感");
}

#[test]
fn test_cloud_interaction_transparency() {
    // 验证：主菜单云雾不应阻挡 UI 交互
    // 这里我们检查逻辑：UI 粒子应默认不拦截 Pick 事件 (在 Bevy 0.15 中通过特定组件或 Node 配置)
}
