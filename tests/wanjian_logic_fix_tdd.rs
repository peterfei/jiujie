use bevy::prelude::*;
use bevy_card_battler::components::sprite::{PlayerWeapon, WanJianTriggerEvent};
use bevy_card_battler::components::particle::{Particle, EffectType};
use bevy_card_battler::components::HeavenlyStrikeCinematic;

#[test]
fn test_visibility_persistence_during_cinematic() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    
    // 1. 初始化资源和 Observer
    app.insert_resource(HeavenlyStrikeCinematic::default());
    app.add_observer(|trigger: Trigger<WanJianTriggerEvent>, mut query: Query<&mut Visibility, With<PlayerWeapon>>| {
        match trigger.event() {
            WanJianTriggerEvent::Start => { for mut v in query.iter_mut() { *v = Visibility::Hidden; } }
            WanJianTriggerEvent::End => { for mut v in query.iter_mut() { *v = Visibility::Visible; } }
        }
    });

    // 2. 创建武器
    let weapon_entity = app.world_mut().spawn((PlayerWeapon, Visibility::Visible)).id();

    // 3. 模拟引雷术开始
    app.world_mut().trigger(WanJianTriggerEvent::Start);
    app.world_mut().resource_mut::<HeavenlyStrikeCinematic>().active = true;
    app.update();
    
    assert_eq!(*app.world().get::<Visibility>(weapon_entity).unwrap(), Visibility::Hidden, "开始时应隐藏");

    // 4. 运行现有的检测系统 (我们要实现的版本应考虑 cinematic.active)
    // 模拟现有的系统逻辑：如果没有任何 WanJian 粒子且非 Active，则结束
    app.add_systems(Update, check_wanjian_end_v2);
    app.update();

    assert_eq!(*app.world().get::<Visibility>(weapon_entity).unwrap(), Visibility::Hidden, "演出期间不应提前恢复显示");

    // 5. 模拟演出结束
    app.world_mut().resource_mut::<HeavenlyStrikeCinematic>().active = false;
    app.update();

    assert_eq!(*app.world().get::<Visibility>(weapon_entity).unwrap(), Visibility::Visible, "演出结束后应恢复显示");
}

fn check_wanjian_end_v2(
    mut commands: Commands,
    query: Query<&Particle>,
    weapon_query: Query<&Visibility, With<PlayerWeapon>>,
    cinematic: Res<HeavenlyStrikeCinematic>,
) {
    let has_wanjian = query.iter().any(|p| p.effect_type == EffectType::WanJian);
    let is_hidden = weapon_query.iter().any(|v| matches!(*v, Visibility::Hidden));
    
    // 修正：只有当没有万剑粒子 且 没有正在进行的天象演出时，才结束
    if !has_wanjian && !cinematic.active && is_hidden {
        commands.trigger(WanJianTriggerEvent::End);
    }
}
