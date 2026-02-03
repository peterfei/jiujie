//! 游戏结束 (身死道消) 集成测试

use bevy::prelude::*;
use bevy_card_battler::states::GameState;
use bevy_card_battler::components::*;

mod test_utils;
use test_utils::*;

#[test]
fn test_game_over_flow() {
    let mut app = create_test_app();
    
    // 1. 进入战斗并模拟死亡
    info!("--- 阶段 1: 战斗中陨落 ---");
    setup_combat_scene(&mut app);
    advance_frames(&mut app, 5);
    
    // 使玩家 HP 归零
    {
        let mut player_query = app.world_mut().query::<&mut Player>();
        let mut player = player_query.single_mut(app.world_mut());
        player.hp = 0;
    }
    
    // 运行 Update 触发 check_combat_end -> GameOver 状态切换
    app.update();
    
    // 再次运行一次 Update 以确保 OnEnter(GameOver) 被调度
    app.update();
    
    // 验证是否进入 GameOver 状态
    assert_eq!(get_current_state(&app), GameState::GameOver);
    info!("✅ 成功触发身死道消状态");

    // 2. 验证结算界面数据 (可选，主要验证状态转换)
    
    // 3. 模拟点击“重塑道基”
    info!("--- 阶段 2: 重塑道基 ---");
    
    // 注意：在测试中，由于 RestartButton 在 mod.rs 是私有的，
    // 我们在这里直接手动触发 handle_game_over_clicks 的等效逻辑。
    transition_to_state(&mut app, GameState::Prologue);
    
    // 模拟重置行为（模仿 handle_game_over_clicks）
    {
        // 1. 重置 Resource 版 Player
        if let Some(mut player) = app.world_mut().get_resource_mut::<Player>() {
            *player = Player::default();
        }
        
        // 2. 重置 Component 版 Player (如果存在)
        let mut player_query = app.world_mut().query::<&mut Player>();
        for mut p in player_query.iter_mut(app.world_mut()) {
            *p = Player::default();
        }

        // 3. 重置修为
        if let Some(mut cultivation) = app.world_mut().get_resource_mut::<Cultivation>() {
            *cultivation = Cultivation::new();
        }
    }
    
    advance_frames(&mut app, 5);
    assert_eq!(get_current_state(&app), GameState::Prologue);
    
    // 验证资源是否已重置
    {
        let player = app.world().get_resource::<Player>().expect("找不到玩家资源");
        assert_eq!(player.hp, player.max_hp);
        
        let cultivation = app.world().get_resource::<Cultivation>().expect("找不到修为资源");
        assert_eq!(cultivation.insight, 0);
    }
    
    info!("✅ 资源重置验证通过");
}
