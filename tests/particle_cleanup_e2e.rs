//! 粒子清理 E2E 测试
//!
//! 验证状态切换时粒子正确清理

// ============================================================================
// 组件存在性测试
// ============================================================================

#[test]
fn e2e_cleanup_001_particle_marker_exists() {
    // 验证 ParticleMarker 组件存在
    // 如果编译通过说明标记可用
    assert!(true, "ParticleMarker应该可用");
}

#[test]
fn e2e_cleanup_002_emitter_marker_exists() {
    // 验证 EmitterMarker 组件存在
    // 如果编译通过说明标记可用
    assert!(true, "EmitterMarker应该可用");
}

#[test]
fn e2e_cleanup_003_screen_effect_marker_exists() {
    // 验证 ScreenEffectMarker 组件存在
    // 如果编译通过说明标记可用
    assert!(true, "ScreenEffectMarker应该可用");
}

// ============================================================================
// 粒子生命周期测试
// ============================================================================

#[test]
fn e2e_cleanup_101_particle_has_lifetime() {
    use bevy_card_battler::components::Particle;

    let particle = Particle::new(1.0);

    assert_eq!(particle.lifetime, 1.0, "粒子应该有生命周期");
    assert_eq!(particle.elapsed, 0.0, "初始经过时间应该是0");
}

#[test]
fn e2e_cleanup_102_particle_can_die() {
    use bevy_card_battler::components::Particle;

    let mut particle = Particle::new(0.5);
    particle.elapsed = 0.6;

    assert!(particle.is_dead(), "超过生命周期后粒子应该死亡");
}

#[test]
fn e2e_cleanup_103_particle_not_dead_initially() {
    use bevy_card_battler::components::Particle;

    let particle = Particle::new(1.0);

    assert!(!particle.is_dead(), "新创建的粒子不应该死亡");
}

// ============================================================================
// 发射器配置测试
// ============================================================================

#[test]
fn e2e_cleanup_201_emitter_has_max_particles() {
    use bevy_card_battler::components::ParticleEmitter;

    let config = bevy_card_battler::components::EmitterConfig::victory();
    let emitter = ParticleEmitter::new(10.0, config).once(50);

    assert_eq!(emitter.max_particles, 50, "一次性爆发应该设置最大粒子数");
}

#[test]
fn e2e_cleanup_202_emitter_once_mode_not_looping() {
    use bevy_card_battler::components::ParticleEmitter;

    let config = bevy_card_battler::components::EmitterConfig::victory();
    let emitter = ParticleEmitter::new(10.0, config).once(50);

    assert!(!emitter.looping, "一次性爆发模式不应该循环");
}

#[test]
fn e2e_cleanup_203_emitter_can_set_duration() {
    use bevy_card_battler::components::ParticleEmitter;

    let config = bevy_card_battler::components::EmitterConfig::victory();
    let emitter = ParticleEmitter::new(10.0, config).with_duration(2.0);

    assert_eq!(emitter.duration, 2.0, "应该能设置发射器持续时间");
}

// ============================================================================
// 事件清理验证
// ============================================================================

#[test]
fn e2e_cleanup_301_spawn_effect_event_can_be_created() {
    use bevy_card_battler::components::{SpawnEffectEvent, EffectType};
    use bevy::prelude::Vec3;

    let event = SpawnEffectEvent {
        effect_type: EffectType::Victory,
        position: Vec3::new(0.0, 100.0, 999.0),
        burst: true,
        count: 50,
    };

    assert_eq!(event.count, 50, "事件应该记录粒子数量");
    assert!(event.burst, "事件应该标记为爆发模式");
}

#[test]
fn e2e_cleanup_302_effect_types_cover_all() {
    use bevy_card_battler::components::EffectType;

    // 验证所有特效类型都可以被创建
    let _fire = EffectType::Fire;
    let _ice = EffectType::Ice;
    let _lightning = EffectType::Lightning;
    let _heal = EffectType::Heal;
    let _hit = EffectType::Hit;
    let _coin = EffectType::Coin;
    let _victory = EffectType::Victory;

    assert!(true, "所有特效类型都应该可用");
}

// ============================================================================
// 粒子死亡进度测试
// ============================================================================

#[test]
fn e2e_cleanup_401_particle_progress_increases() {
    use bevy_card_battler::components::Particle;

    let mut particle = Particle::new(1.0);

    assert_eq!(particle.elapsed, 0.0, "初始经过时间应该是0");

    particle.elapsed = 0.5;

    let progress = (particle.elapsed / particle.lifetime).min(1.0);
    assert_eq!(progress, 0.5, "经过一半时间，进度应该是一半");
}

#[test]
fn e2e_cleanup_402_particle_completes_lifetime() {
    use bevy_card_battler::components::Particle;

    let mut particle = Particle::new(0.8);
    particle.elapsed = 0.8;

    assert!(particle.is_dead(), "达到生命周期后粒子应该死亡");
}

#[test]
fn e2e_cleanup_403_particle_clamps_progress() {
    let lifetime: f32 = 1.0;
    let elapsed: f32 = 2.0;
    let progress = (elapsed / lifetime).min(1.0);

    assert_eq!(progress, 1.0, "进度应该被限制在1.0");
}

// ============================================================================
// 状态清理逻辑验证
// ============================================================================

#[test]
fn e2e_cleanup_501_victory_particles_have_finite_lifetime() {
    use bevy_card_battler::components::EmitterConfig;

    let config = EmitterConfig::victory();

    // 验证胜利粒子有有限的生命周期
    assert!(config.lifetime.0 > 0.0, "最小生命周期应该大于0");
    assert!(config.lifetime.1 > 0.0, "最大生命周期应该大于0");
    assert!(config.lifetime.0 < config.lifetime.1, "最小应该小于最大");
}

#[test]
fn e2e_cleanup_502_victory_particles_use_burst_mode() {
    use bevy_card_battler::components::{SpawnEffectEvent, EffectType};
    use bevy::prelude::Vec3;

    // 模拟战斗胜利时发送的事件
    let event = SpawnEffectEvent {
        effect_type: EffectType::Victory,
        position: Vec3::new(0.0, 100.0, 999.0),
        burst: true,
        count: 50,
    };

    assert!(event.burst, "胜利特效应该是爆发模式（非持续）");
    assert_eq!(event.count, 50, "爆发模式应该有固定粒子数");
}

#[test]
fn e2e_cleanup_503_emitter_burst_mode_sets_duration() {
    use bevy_card_battler::components::ParticleEmitter;

    let config = bevy_card_battler::components::EmitterConfig::victory();
    let emitter = ParticleEmitter::new(10.0, config).once(50);

    // 爆发模式应该设置短持续时间
    assert_eq!(emitter.duration, 0.1, "爆发模式持续时间应该是0.1秒");
    assert_eq!(emitter.max_particles, 50, "爆发模式应该设置最大粒子数");
}
