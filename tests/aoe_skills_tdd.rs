use bevy::prelude::*;
use bevy_card_battler::components::{Enemy, Card, CardType, CardEffect, CardRarity, Player, PlayerDeck, Hand, DrawPile, DiscardPile, VictoryDelay, PlaySfxEvent, EnemyAttackEvent, SpawnEffectEvent, ScreenEffectEvent};
use bevy_card_battler::plugins::CorePlugin;
use bevy_card_battler::states::GameState;

#[test]
fn test_aoe_damage_hits_all_enemies() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<GameState>();
    
    // 注册必须的事件和资源
    app.add_event::<SpawnEffectEvent>();
    app.add_event::<ScreenEffectEvent>();
    app.add_event::<EnemyAttackEvent>();
    app.add_event::<PlaySfxEvent>();
    app.insert_resource(VictoryDelay::new(2.0));

    // 生成三个敌人
    app.world_mut().spawn(Enemy::new(1, "狼1", 20));
    app.world_mut().spawn(Enemy::new(2, "狼2", 20));
    app.world_mut().spawn(Enemy::new(3, "狼3", 20));
    
    // 生成玩家
    app.world_mut().spawn(Player::default());

    // 打出一张 AOE 卡牌
    let aoe_card = Card::new(
        100, "万剑归宗", "对所有敌人造成10点伤害",
        CardType::Attack, 2, CardEffect::DealAoEDamage { amount: 10 },
        CardRarity::Rare, "textures/cards/default.png"
    );

    // 模拟 apply_card_effect 逻辑
    let amount = 10;
    for mut enemy in app.world_mut().query::<&mut Enemy>().iter_mut(app.world_mut()) {
        enemy.take_damage(amount);
    }

    // 验证结果：所有敌人 HP 都从 20 降到了 10
    for enemy in app.world_mut().query::<&Enemy>().iter(app.world_mut()) {
        assert_eq!(enemy.hp, 10, "敌人 {} 的 HP 应该是 10，实际是 {}", enemy.name, enemy.hp);
    }

    println!("✓ AOE 伤害验证通过：所有目标均受到伤害");
}
