pub mod test_utils;
use crate::test_utils::*;
use bevy::prelude::*;
use bevy_card_battler::states::GameState;
use bevy_card_battler::systems::event::EventChoiceButton;
use bevy_card_battler::systems::rest::UpgradeButton;
use bevy_card_battler::components::{Player, Cultivation};

#[test]
fn reproduce_rest_hang() {
    let mut app = create_test_app();
    
    // 0. 初始化玩家实体 (修复 Panic: NoEntities)
    app.world_mut().spawn((Player::default(), Cultivation::new()));

    // 1. 进入休息状态
    transition_to_state(&mut app, GameState::Rest);
    advance_frames(&mut app, 5); // 等待 UI 生成

    // 2. 模拟多次点击"功法精进"按钮
    for i in 0..5 {
        println!("【Test】点击功法精进按钮 第 {} 次", i + 1);
        
        // 尝试找到按钮
        let button_entity_opt = app.world_mut()
            .query_filtered::<Entity, With<UpgradeButton>>()
            .iter(app.world())
            .next();

        if let Some(button_entity) = button_entity_opt {
            // 模拟按下
            if let Some(mut interaction) = app.world_mut().get_mut::<Interaction>(button_entity) {
                *interaction = Interaction::Pressed;
            }
            
            // 运行系统处理点击
            app.update();
            
            // 模拟松开
            if let Some(mut interaction) = app.world_mut().get_mut::<Interaction>(button_entity) {
                *interaction = Interaction::None;
            }
            app.update();
        } else {
            // 第一次点击后按钮应该还在（只是不可见？）或者已被销毁？
            // 在 rest 系统逻辑中，点击后会隐藏选项区域，所以按钮实体应该还在，只是 Interaction 不可见了？
            // 实际上 Bevy 的 Query 即使不可见也能查到，除非组件被移除了。
            // 我们的测试是为了复现 Hang，如果没 Hang 且能跑完循环，说明至少没有死循环。
            println!("【Test】警告：第 {} 次未找到按钮，可能 UI 已变化", i + 1);
        }
    }
}

#[test]
fn reproduce_event_hang() {
    let mut app = create_test_app();
    
    // 0. 初始化玩家实体
    app.world_mut().spawn((Player::default(), Cultivation::new()));

    // 1. 进入机缘状态
    transition_to_state(&mut app, GameState::Event);
    advance_frames(&mut app, 5);

    // 2. 查找"虔诚祈祷"按钮
    let mut heal_btn_entity = None;
    
    for (entity, choice) in app.world_mut().query::<(Entity, &EventChoiceButton)>().iter(app.world()) {
        if let EventChoiceButton::Heal(_) = choice {
            heal_btn_entity = Some(entity);
            break;
        }
    }
    
    assert!(heal_btn_entity.is_some(), "未找到虔诚祈祷按钮");
    let btn = heal_btn_entity.unwrap();

    // 3. 模拟点击
    println!("【Test】点击虔诚祈祷按钮");
    if let Some(mut interaction) = app.world_mut().get_mut::<Interaction>(btn) {
        *interaction = Interaction::Pressed;
    }
    
    // 4. 运行更新
    // 如果逻辑有死循环，这里会卡住
    app.update();
    app.update();
    
    // 5. 验证是否切换到了地图状态
    let current_state = get_current_state(&app);
    assert_eq!(current_state, GameState::Map, "点击祈祷后应返回地图，当前状态: {:?}", current_state);
}