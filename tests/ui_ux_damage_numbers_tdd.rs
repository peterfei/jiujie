use bevy::prelude::*;
use bevy_card_battler::components::combat::{Enemy, DamageEffectEvent};
use bevy_card_battler::components::DamageNumber;
use bevy_card_battler::states::GameState;
use bevy_card_battler::systems::ui::UiPlugin;

#[test]
fn test_damage_triggers_floating_number() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.add_plugins(UiPlugin); // 添加我们要测试的插件
    app.init_state::<GameState>();
    app.insert_state(GameState::Combat);

    // 1. 模拟一个敌人位置
    let enemy_pos = Vec2::new(200.0, 100.0);

    // 2. 直接发送 DamageEffectEvent (模拟逻辑触发)
    app.world_mut().send_event(DamageEffectEvent {
        position: enemy_pos,
        amount: 15,
    });
    
    // 运行一帧处理事件
    app.update();

    // 3. 验证是否生成了飘字实体
    let mut query = app.world_mut().query_filtered::<Entity, With<DamageNumber>>();
    let count = query.iter(app.world()).count();
    
    assert!(count > 0, "发送 DamageEffectEvent 后应该生成飘字实体");
    
    // 验证位置是否正确 (640 + 200, 360 - 100) = (840, 260)
    let (entity, dn, node, _) = app.world_mut().query::<(Entity, &DamageNumber, &Node, &TextColor)>().single(app.world());
    assert_eq!(dn.value, 15);
    if let Val::Px(left) = node.left {
        assert_eq!(left, 840.0);
    }
    if let Val::Px(top) = node.top {
        assert_eq!(top, 260.0);
    }

    // 4. 验证动画效果 (这里我们跳过复杂的 Time 模拟，只验证基本逻辑)
    app.update();
    
    let (_, _, node_after, _) = app.world_mut().query::<(Entity, &DamageNumber, &Node, &TextColor)>().single(app.world());
    assert!(node_after.top != Val::Auto, "飘字应该有位置信息");

    println!("✅ 受击飘字 UI/UX 优化 TDD 测试通过");
}
