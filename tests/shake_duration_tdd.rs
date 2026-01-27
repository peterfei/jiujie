use bevy::prelude::*;
use bevy_card_battler::components::screen_effect::ScreenEffectEvent;

#[test]
fn test_shake_duration_calculation() {
    // 模拟万剑归宗的参数
    let trauma = 1.0f32;
    let decay = 0.45f32; // 预期衰减率
    
    let duration = trauma / decay;
    
    // 预期：持续时间应接近特效总时长 (2.2s)
    assert!(duration >= 2.0, "震屏持续时间太短: {:.2}s, 预期 >= 2.0s", duration);
}
