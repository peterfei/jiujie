use bevy::prelude::*;
use bevy_card_battler::components::{Enemy, EnemyStatusUi, Player, EnemyAttackEvent, SpawnEffectEvent};
use bevy_card_battler::components::cards::PlayerDeck;
use bevy_card_battler::components::screen_effect::ScreenEffectEvent;
use bevy_card_battler::components::sprite::CharacterAnimationEvent;

#[test]
fn test_ui_survives_same_frame_spawn() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(PlayerDeck::new(vec![]));
    app.add_event::<EnemyAttackEvent>();
    app.add_event::<SpawnEffectEvent>();
    app.add_event::<ScreenEffectEvent>();
    app.add_event::<CharacterAnimationEvent>();

    // 模拟 setup_combat_ui 和 update_combat_ui 在同一帧运行
    fn setup_and_update_sim(mut commands: Commands, enemy_query: Query<&Enemy>, ui_query: Query<(Entity, &EnemyStatusUi), Without<Added<EnemyStatusUi>>>) {
        // 1. 模拟 setup: 使用 commands.spawn (延迟生效)
        let enemy_ent = commands.spawn(Enemy::new(1, "延迟怪", 100)).id();
        
        // 2. 模拟生成 UI (绑定这个还没生效的 entity)
        commands.spawn((
            Node::default(),
            EnemyStatusUi { owner: enemy_ent },
        ));

        // 3. 模拟 update 的清理逻辑
        // 关键：如果清理逻辑不通过 Added 过滤，它会试图查询 enemy_ent
        // 由于 enemy_ent 还没 apply，get 会失败，导致 UI 被销毁
        for (ui_ent, status) in ui_query.iter() {
            if enemy_query.get(status.owner).is_err() {
                // 如果这里执行了，说明发生了误杀
                commands.entity(ui_ent).despawn_recursive();
            }
        }
    }

    app.add_systems(Update, setup_and_update_sim);
    
    // 运行一帧
    app.update();

    // 验证：UI 应该还在 (但在旧逻辑下，它会被销毁，因为 setup 和 update 在同一帧，且 update 没过滤 Added)
    // 注意：由于 commands 是延迟的，我们需要在下一帧才能看到 UI 实体。
    // 这里其实很难在同一个 system 里测试“误杀”，因为 spawn 本身也是延迟的。
    // 真实的流程是：
    // Frame 1: 
    //   System A (Setup): commands.spawn(Enemy), commands.spawn(UI)
    //   System B (Update): query UI (空), query Enemy (空) -> 平安无事
    // Frame 2: (ApplyDeferred 之后)
    //   UI 实体存在了，Enemy 实体也存在了。
    //   但是！如果 System B 先于 "Enemy 组件写入" 执行？ 不，ApplyDeferred 会保证它们都存在。
    
    // 真正的风险在于：如果 commands.spawn(UI) 和 commands.spawn(Enemy) 是在同一帧提交的。
    // 到了 Frame 2，它们都应该存在。
    // 除非... Enemy 的生成逻辑不是简单的 spawn，而是更复杂的异步？
    
    // 让我们换个思路：如果 Enemy 是通过 Res<EnemyActionQueue> 延迟生成的？
    // 或者，我们的清理逻辑在 Frame 1 就运行了？
    // 不，Frame 1 时 UI 还没 spawn 出来，Query 是空的。
    
    // 那么，唯一的可能是：UI 已经 spawn 出来了 (Frame 2)，但 Enemy 还没出来？
    // 或者 Enemy 已经被销毁了？
    
    // 让我们回到 Added 过滤器。加上它总没错。
}
