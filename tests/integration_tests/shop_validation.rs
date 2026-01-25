//! 商店系统专项测试
//!
//! 测试商店相关的 bug 和场景
//! 继承自 commit 2a818d5 测试框架

use crate::test_utils::*;
use bevy::prelude::*;
use bevy_card_battler::components::{Player, ShopCardButton, ShopExitButton};
use bevy_card_battler::states::GameState;
use bevy_card_battler::components::shop::ShopUiRoot;

// ============================================================================
// 测试1: 组件结构验证 - 商店UI按钮必须有标记组件
// ============================================================================

#[test]
fn test_shop_ui_purchase_buttons_have_markers() {
    // 场景：验证商店UI中的购买按钮有正确的标记组件
    // 预防：购买按钮只有 Button 组件，缺少 ShopCardButton/ShopRelicButton 标记

    let mut app = create_test_app();
    setup_shop_scene(&mut app);
    advance_frames(&mut app, 1);

    // 验证：购买按钮应该同时有 Button 和标记组件
    let mut card_btn_query = app.world_mut().query::<(&ShopCardButton, &Button)>();
    let card_btn_count = card_btn_query.iter(&app.world()).count();

    println!("✓ 商店卡牌按钮数量: {}", card_btn_count);
    assert!(card_btn_count > 0, "商店UI中应该有带完整标记的购买按钮");

    // 验证：返回按钮有标记
    let mut exit_btn_query = app.world_mut().query::<(&ShopExitButton, &Button)>();
    let exit_btn_count = exit_btn_query.iter(&app.world()).count();

    println!("✓ 返回按钮数量: {}", exit_btn_count);
    assert!(exit_btn_count > 0, "商店UI中应该有返回按钮");
}

// ============================================================================
// 测试2: 系统注册验证 - update_gold_display 是否运行
// ============================================================================

#[test]
fn test_shop_update_gold_system_runs() {
    // 场景：验证 update_gold_display 系统真的在运行
    // 预防：ShopPlugin 未注册或系统注册顺序错误

    let mut app = create_test_app();
    setup_shop_scene(&mut app);
    advance_frames(&mut app, 1);

    // 获取初始金币文本
    let gold_before = get_player_gold(&mut app);
    println!("初始金币: {}", gold_before);
    assert_eq!(gold_before, 100, "初始应显示100金币");

    // 修改玩家金币（模拟购买）
    {
        let world = app.world_mut();
        if let Ok(mut player) = world.query::<&mut Player>().get_single_mut(world) {
            player.gold -= 30;
        }
    }

    // 运行多帧让 update_gold_display 处理
    advance_frames(&mut app, 5);

    // 验证：金币文本应该更新
    let gold_after = get_player_gold(&mut app);
    println!("修改后金币: {}", gold_after);
    assert_eq!(gold_after, 70, "金币UI应该更新为70");

    println!("✓ update_gold_display 系统正常运行");
}

// ============================================================================
// 测试3: 商店金币初始显示验证
// ============================================================================

#[test]
fn test_shop_initial_gold_display() {
    // 场景：验证商店显示正确的初始金币
    // 预防：UI 显示 "金币: 0" 因为 Player 尚未创建或 Commands 延迟

    let mut app = create_test_app();
    setup_shop_scene(&mut app);
    advance_frames(&mut app, 1);

    // 检查：Player 应该被创建
    let player_count = app.world_mut().query::<&Player>().iter(&app.world()).count();
    println!("✓ Player 实体数量: {}", player_count);
    assert!(player_count > 0, "Player 应该被创建");

    // 检查：应该显示正确的金币（不是0）
    let gold = get_player_gold(&mut app);
    println!("金币显示: {}", gold);
    assert_eq!(gold, 100, "应该显示100金币，不是0");

    println!("✓ 商店初始金币显示正确");
}

// ============================================================================
// 测试4: 重复进入商店不重复创建实体
// ============================================================================

#[test]
fn test_multiple_shop_entries_doesnt_duplicate() {
    // 场景：多次进入商店状态不应重复创建 Player 实体
    // 预防：每次 OnEnter 都创建新 Player

    let mut app = create_test_app();

    // 第一次进入商店
    setup_shop_scene(&mut app);
    advance_frames(&mut app, 1);

    let player_count_1 = app.world_mut().query::<&Player>().iter(&app.world()).count();
    println!("✓ 第一次进入商店后 Player 数量: {}", player_count_1);
    assert_eq!(player_count_1, 1, "第一次进入应该创建1个Player");

    // 退出到地图
    transition_to_state(&mut app, GameState::Map);
    advance_frames(&mut app, 1);

    // 第二次进入商店
    setup_shop_scene(&mut app);
    advance_frames(&mut app, 1);

    let player_count_2 = app.world_mut().query::<&Player>().iter(&app.world()).count();
    println!("✓ 第二次进入商店后 Player 数量: {}", player_count_2);
    assert_eq!(player_count_2, 1, "第二次进入不应该创建新Player，仍然只有1个");

    println!("✓ 重复进入商店不重复创建实体测试通过");
}
