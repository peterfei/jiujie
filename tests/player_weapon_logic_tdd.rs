use bevy::prelude::*;
use bevy::app::App;
use bevy_card_battler::components::sprite::{PlayerSpriteMarker, PlayerWeapon, PhysicalImpact, ActionType};
use bevy_card_battler::systems::sprite::update_weapon_animation;

#[test]
fn test_player_weapon_orientation_and_animation() {
    let mut app = App::new();
    
    app.add_plugins((
        MinimalPlugins,
        bevy::hierarchy::HierarchyPlugin,
    ));

    // 1. 准备环境：创建一个带武器的玩家
    let player_entity = app.world_mut().spawn((
        PlayerSpriteMarker,
        PhysicalImpact::default(),
        Transform::from_xyz(0.0, 0.0, 0.0),
    )).id();

    let weapon_entity = app.world_mut().spawn((
        PlayerWeapon,
        Transform::from_rotation(Quat::IDENTITY), // 模拟“朝天”或初始状态
    )).set_parent(player_entity).id();

    // 2. 注册系统
    app.add_systems(Update, update_weapon_animation);

    // 3. 测试【待机态】
    app.update();
    
    let weapon_transform = app.world().get::<Transform>(weapon_entity).expect("武器应该有 Transform");
    // 验证旋转不再是 IDENTITY (即我们修正后的初始姿态)
    assert_ne!(weapon_transform.rotation, Quat::IDENTITY, "武器初始旋转不应为默认(朝天)");

    let idle_rotation = weapon_transform.rotation;

    // 4. 测试【攻击态】
    // 模拟攻击：设置 PhysicalImpact 计时器
    if let Some(mut impact) = app.world_mut().get_mut::<PhysicalImpact>(player_entity) {
        impact.action_timer = 0.5;
        impact.action_type = ActionType::Dash;
    }

    // 运行多次，确保动画生效
    app.update();
    
    let attack_rotation = app.world().get::<Transform>(weapon_entity).unwrap().rotation;
    
    // 验证旋转发生了变化（相对于待机态）
    assert_ne!(attack_rotation, idle_rotation, "攻击时武器旋转应该发生动态变化");
    
    println!("待机旋转: {:?}", idle_rotation);
    println!("攻击旋转: {:?}", attack_rotation);
}
