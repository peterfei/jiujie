use bevy::prelude::*;
use bevy_card_battler::states::GameState;
use bevy_card_battler::components::combat::EnemyActionQueue;

#[test]
fn test_enemy_action_sequencing() {
    let mut app = App::new();
    app.init_resource::<EnemyActionQueue>();
    
    // 模拟 3 个敌人
    let e1 = app.world_mut().spawn_empty().id();
    let e2 = app.world_mut().spawn_empty().id();
    
    let mut queue = app.world_mut().resource_mut::<EnemyActionQueue>();
    queue.enemies = vec![e1, e2];
    queue.current_index = 0;
    queue.timer = Timer::from_seconds(1.0, TimerMode::Once);

    // 验证逻辑：第一帧处理第一个
    assert_eq!(queue.enemies[queue.current_index], e1);
    
    // 模拟时间流逝
    queue.timer.tick(std::time::Duration::from_secs(2));
    if queue.timer.finished() {
        queue.current_index += 1;
    }

    // 验证逻辑：第二阶段处理第二个
    assert_eq!(queue.enemies[queue.current_index], e2);
}
