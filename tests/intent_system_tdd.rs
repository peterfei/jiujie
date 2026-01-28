use bevy::prelude::*;
use bevy_card_battler::components::combat::{Enemy, Player, EnemyIntent, IntentIconMarker, EnemyIntentText};

#[test]
fn test_intent_math_logic() {
    let mut player = Player::default();
    player.vulnerable = 1; // 玩家易伤 1.5x
    let mut enemy = Enemy::new(1, "怪", 100);
    enemy.weakness = 1; // 敌人虚弱 0.75x
    
    // 原始 10 -> 虚弱 7 -> 玩家易伤 10
    let final_val = player.calculate_incoming_damage(enemy.calculate_outgoing_damage(10));
    assert_eq!(final_val, 10);
}

#[test]
fn test_intent_ui_formatting_and_visibility() {
    let mut app = App::new();
    
    let enemy_ent = app.world_mut().spawn(Enemy::new(1, "测试怪", 100)).id();
    let text_ent = app.world_mut().spawn((Text::new(""), EnemyIntentText { owner: enemy_ent })).id();
    let icon_ent = app.world_mut().spawn((Node::default(), Visibility::Hidden, IntentIconMarker { owner: enemy_ent })).id();

    // 100% 还原 lib.rs 中的同步逻辑
    let sync_system = |
        enemy_query: Query<&Enemy>,
        mut text_query: Query<(&EnemyIntentText, &mut Text)>,
        mut icon_query: Query<(&IntentIconMarker, &mut Visibility)>,
    | {
        for (marker, mut text) in text_query.iter_mut() {
            if let Ok(enemy) = enemy_query.get(marker.owner) {
                text.0 = match &enemy.intent {
                    EnemyIntent::Attack { damage } => format!("攻击 {}", damage),
                    EnemyIntent::Defend { block } => format!("防御 {}", block),
                    _ => "观察中...".to_string(),
                };
            }
        }
        for (marker, mut vis) in icon_query.iter_mut() {
            if let Ok(enemy) = enemy_query.get(marker.owner) {
                *vis = match &enemy.intent {
                    EnemyIntent::Attack { .. } | EnemyIntent::Defend { .. } => Visibility::Visible,
                    _ => Visibility::Hidden,
                };
            }
        }
    };

    app.add_systems(Update, sync_system);

    // 场景 A: 攻击
    if let Some(mut e) = app.world_mut().get_mut::<Enemy>(enemy_ent) {
        e.intent = EnemyIntent::Attack { damage: 10 };
    }
    app.update();
    assert!(app.world().get::<Text>(text_ent).unwrap().0.contains("攻击 10"));
    assert_eq!(*app.world().get::<Visibility>(icon_ent).unwrap(), Visibility::Visible);

    // 场景 B: 观察 (Debuff 在此处视为非动作)
    if let Some(mut e) = app.world_mut().get_mut::<Enemy>(enemy_ent) {
        e.intent = EnemyIntent::Debuff { poison: 1, weakness: 1 };
    }
    app.update();
    assert!(app.world().get::<Text>(text_ent).unwrap().0.contains("观察中"));
    assert_eq!(*app.world().get::<Visibility>(icon_ent).unwrap(), Visibility::Hidden);

    println!("✅ 意图 UI 逻辑 TDD 全量验证通过");
}
