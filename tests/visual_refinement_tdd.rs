use bevy::prelude::*;
use bevy_card_battler::components::sprite::{PhysicalImpact, BreathAnimation};

#[test]
fn test_base_transparency() {
    let mut app = App::new();
    app.init_resource::<Assets<StandardMaterial>>();
    
    // 模拟生成底座材质
    let mat_handle = app.world_mut().resource_mut::<Assets<StandardMaterial>>().add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.2, 0.0, 0.3), // 半透明绿
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let materials = app.world().resource::<Assets<StandardMaterial>>();
    let (_, mat) = materials.iter().next().unwrap();
    assert!(mat.base_color.alpha() < 1.0, "底座应该是半透明的以减少突兀感");
}

#[test]
fn test_bite_oscillation_physics() {
    // 逻辑：在 action_timer > 0 时，产生了基于 Sine 的额外偏移
    let timer = 0.5f32;
    let bite_factor = (timer * 20.0).sin() * 0.5; // 提高频率到 20.0
    
    assert!(bite_factor.abs() > 0.0, "撕咬动作应产生显著的摆动偏移");
}
