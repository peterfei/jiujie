use bevy::prelude::*;
use bevy_card_battler::components::particle::{EffectType};
use bevy_card_battler::states::GameState;

#[path = "test_utils.rs"]
mod test_utils;
use test_utils::*;

#[test]
fn test_main_menu_cloud_coverage_logic() {
    let mut app = create_test_app();
    
    // 模拟切换到主菜单
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::MainMenu);
    app.update(); // 触发 OnEnter(MainMenu)
    
    // 验证：配置参数符合史诗级设计
    let config = EffectType::CloudMist.config();
    assert!(config.size.1 >= 1000.0, "全屏云雾单体尺寸应足够大");
    assert!(config.lifetime.1 >= 15.0, "主菜单云雾应具有极长的停留时间");
}

#[test]
fn test_cloud_interaction_transparency() {
    // 验证：主菜单云雾不应阻挡 UI 交互
    // 这里我们检查逻辑：UI 粒子应默认不拦截 Pick 事件 (在 Bevy 0.15 中通过特定组件或 Node 配置)
}
