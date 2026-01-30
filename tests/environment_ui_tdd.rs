use bevy::prelude::*;
use bevy_card_battler::components::combat::{Environment, EnvironmentText, EnvironmentPanel};
use bevy_card_battler::states::GameState;

#[path = "test_utils.rs"]
mod test_utils;
use test_utils::*;

#[test]
fn test_environment_ui_creation() {
    let mut app = create_test_app();
    
    // 1. 进入战斗状态
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.update(); // 触发 OnEnter(Combat)
    
    // 2. 验证环境面板是否存在
    let mut panel_query = app.world_mut().query::<&EnvironmentPanel>();
    let panel_count = panel_query.iter(app.world()).count();
    assert_eq!(panel_count, 1, "应该创建一个天象环境面板");
    
    // 3. 验证环境文本是否包含默认值
    let mut text_query = app.world_mut().query::<&Text>();
    let texts: Vec<String> = text_query.iter(app.world()).map(|t| t.0.clone()).collect();
    
    assert!(texts.iter().any(|t| t.contains("当前天象")), "应显示环境名称文本");
}

#[test]
fn test_environment_ui_update() {
    let mut app = create_test_app();
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Combat);
    app.update();
    
    // 1. 修改环境资源为雷暴
    app.world_mut().insert_resource(Environment::thunder_storm());
    app.update(); // 运行一帧以触发 update_environment_ui
    
    // 2. 验证 UI 文本是否更新
    let mut text_query = app.world_mut().query::<&Text>();
    let texts: Vec<String> = text_query.iter(app.world()).map(|t| t.0.clone()).collect();
    
    assert!(texts.iter().any(|t| t.contains("【雷暴】")), "环境面板应显示【雷暴】");
}
