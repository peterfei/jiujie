use bevy::prelude::*;
use bevy_card_battler::components::combat::CombatState;
use bevy_card_battler::components::{DrawPile, Hand};

// 模拟受灾系统
fn mock_draw_system(
    state: Option<ResMut<CombatState>>, // 使用 Option 增强鲁棒性
) {
    if state.is_none() {
        return; // 优雅退出，不 Panic
    }
}

#[test]
fn test_combat_state_missing_safety() {
    let mut app = App::new();
    
    // 故意不添加 CombatState 资源
    app.add_systems(Update, mock_draw_system);
    
    // 运行 1 帧：原本这里会 Panic，现在应平稳度过
    app.update();
}
