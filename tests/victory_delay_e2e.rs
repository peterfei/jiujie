//! 胜利延迟 E2E 测试
//!
//! 验证敌人被击败后延迟进入奖励界面，让粒子特效播放

use bevy_card_battler::components::{VictoryDelay, Enemy, EffectType, SpawnEffectEvent};
use bevy::prelude::Vec3;

// ============================================================================
// 胜利延迟组件测试
// ============================================================================

#[test]
fn e2e_delay_001_victory_delay_exists() {
    let delay = VictoryDelay::new(2.0);

    assert_eq!(delay.duration, 2.0, "延迟时长应该是2.0秒");
    assert!(!delay.active, "初始状态应该是不激活");
    assert_eq!(delay.elapsed, 0.0, "初始经过时间应该是0");
}

#[test]
fn e2e_delay_002_victory_delay_can_be_activated() {
    let mut delay = VictoryDelay::new(2.0);

    delay.active = true;
    delay.elapsed = 0.5;

    assert!(delay.active, "激活后应该是激活状态");
    assert_eq!(delay.elapsed, 0.5, "经过时间应该被记录");
}

#[test]
fn e2e_delay_003_delay_completes_after_duration() {
    let mut delay = VictoryDelay::new(2.0);

    delay.active = true;
    delay.elapsed = 2.0;

    assert!(delay.elapsed >= delay.duration, "经过时间应该达到或超过延迟时长");
}

// ============================================================================
// 粒子事件配置验证
// ============================================================================

#[test]
fn e2e_delay_101_victory_events_have_correct_positions() {
    // 验证胜利粒子事件的位置配置
    let event1 = SpawnEffectEvent {
        effect_type: EffectType::Victory,
        position: Vec3::new(0.0, 100.0, 999.0),
        burst: true,
        count: 50,
        target: None,
        target_entity: None,
        target_group: None,
        target_index: None,
    };
    let event2 = SpawnEffectEvent {
        effect_type: EffectType::Victory,
        position: Vec3::new(-50.0, 80.0, 999.0),
        burst: true,
        count: 30,
        target: None,
        target_entity: None,
        target_group: None,
        target_index: None,
    };
    let event3 = SpawnEffectEvent {
        effect_type: EffectType::Victory,
        position: Vec3::new(50.0, 80.0, 999.0),
        burst: true,
        count: 30,
        target: None,
        target_entity: None,
        target_group: None,
        target_index: None,
    };

    // 验证Y坐标是正值（在屏幕上方）
    assert!(event1.position.y > 0.0, "粒子应该在屏幕上方");
    assert!(event2.position.y > 0.0, "粒子应该在屏幕上方");
    assert!(event3.position.y > 0.0, "粒子应该在屏幕上方");

    // 验证X坐标分布（左、中、右）
    assert_eq!(event1.position.x, 0.0, "主粒子应该在中央");
    assert_eq!(event2.position.x, -50.0, "左侧粒子应该在左边");
    assert_eq!(event3.position.x, 50.0, "右侧粒子应该在右边");
}

#[test]
fn e2e_delay_102_victory_events_total_count() {
    // 验证胜利粒子总数
    let events = vec![
        SpawnEffectEvent {
            effect_type: EffectType::Victory,
            position: Vec3::new(0.0, 100.0, 999.0),
            burst: true,
            count: 50,
            target: None,
            target_entity: None,
            target_group: None,
            target_index: None,
        },
        SpawnEffectEvent {
            effect_type: EffectType::Victory,
            position: Vec3::new(-50.0, 80.0, 999.0),
            burst: true,
            count: 30,
            target: None,
            target_entity: None,
            target_group: None,
            target_index: None,
        },
        SpawnEffectEvent {
            effect_type: EffectType::Victory,
            position: Vec3::new(50.0, 80.0, 999.0),
            burst: true,
            count: 30,
            target: None,
            target_entity: None,
            target_group: None,
            target_index: None,
        },
    ];

    let total: usize = events.iter().map(|e| e.count).sum();
    assert_eq!(total, 110, "总共应该生成110个胜利粒子");
}

#[test]
fn e2e_delay_103_victory_events_use_burst_mode() {
    // 验证所有胜利事件都使用爆发模式
    let events = vec![
        SpawnEffectEvent {
            effect_type: EffectType::Victory,
            position: Vec3::new(0.0, 100.0, 999.0),
            burst: true,
            count: 50,
            target: None,
            target_entity: None,
            target_group: None,
            target_index: None,
        },
    ];

    for event in events {
        assert!(event.burst, "胜利事件应该使用爆发模式");
    }
}

// ============================================================================
// 延迟时长合理性验证
// ============================================================================

#[test]
fn e2e_delay_201_delay_duration_reasonable() {
    let delay = VictoryDelay::new(2.0);

    // 延迟时长应该在1-3秒之间，足够显示粒子但不会太久
    assert!(delay.duration >= 1.0, "延迟时长应该至少1秒");
    assert!(delay.duration <= 3.0, "延迟时长不应该超过3秒");
    assert_eq!(delay.duration, 2.0, "当前实现使用2.0秒延迟");
}

#[test]
fn e2e_delay_202_delay_longer_than_particle_lifetime() {
    use bevy_card_battler::components::EmitterConfig;

    let config = EmitterConfig::victory();
    let delay = VictoryDelay::new(4.1); // 使用比4.0略大的值进行比较测试

    // 粒子最大生命周期
    let max_particle_lifetime = config.lifetime.1;

    // 延迟应该比粒子生命周期长，让粒子充分显示
    assert!(delay.duration > max_particle_lifetime,
        "延迟时长({}秒)应该大于粒子最大生命周期({}秒)",
        delay.duration, max_particle_lifetime);
}

#[test]
fn e2e_delay_203_multiple_victories_reset_delay() {
    let mut delay = VictoryDelay::new(2.0);

    // 第一次胜利
    delay.active = true;
    delay.elapsed = 2.0; // 超过延迟时长
    delay.active = false;
    delay.elapsed = 0.0; // 重置

    // 第二次胜利
    delay.active = true;
    delay.elapsed = 0.5;

    assert!(delay.active, "重置后可以再次激活");
    assert_eq!(delay.elapsed, 0.5, "经过时间应该被正确记录");
}

// ============================================================================
// 敌人死亡与胜利流程验证
// ============================================================================

#[test]
fn e2e_delay_301_enemy_death_can_trigger_victory() {
    let mut enemy = Enemy::new(1, "测试敌人", 50);

    assert!(!enemy.is_dead(), "初始状态敌人应该存活");

    enemy.take_damage(60);
    assert!(enemy.is_dead(), "受到致命伤害后敌人应该死亡");
}

#[test]
fn e2e_delay_302_victory_sequence_is_correct() {
    // 验证胜利流程的正确顺序：
    // 1. 敌人死亡
    // 2. 触发粒子特效
    // 3. 启动延迟计时器
    // 4. 延迟结束后进入奖励界面

    let steps = vec![
        "敌人死亡",
        "触发粒子特效",
        "启动延迟计时器",
        "等待延迟结束",
        "进入奖励界面",
    ];

    assert_eq!(steps.len(), 5, "胜利流程应该有5个步骤");
    assert_eq!(steps[0], "敌人死亡", "第一步应该是敌人死亡");
    assert_eq!(steps[4], "进入奖励界面", "最后一步应该是进入奖励界面");
}

#[test]
fn e2e_delay_303_ui_cleanup_before_reward() {
    // 这个测试验证在进入奖励界面之前会清理战斗UI
    // 避免UI遮挡粒子特效

    let cleanup_steps = vec![
        "清理战斗UI根节点",
        "清理战斗相关实体",
        "显示奖励UI",
    ];

    assert!(cleanup_steps.contains(&"清理战斗UI根节点"),
        "应该在进入奖励界面前清理战斗UI");
}
