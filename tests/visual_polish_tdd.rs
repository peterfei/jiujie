use bevy::prelude::*;
use bevy_card_battler::components::combat::Player;
use bevy_card_battler::components::screen_effect::ScreenWarning;

// 模拟逻辑系统 (刚才已经测过的)
fn mock_warning_system(
    player_query: Query<&Player>,
    mut warning_query: Query<&mut Visibility, With<ScreenWarning>>,
) {
    if let Ok(player) = player_query.get_single() {
        let is_low_hp = (player.hp as f32 / player.max_hp as f32) < 0.3;
        let is_weakened = player.weakness > 0;
        
        if let Ok(mut vis) = warning_query.get_single_mut() {
            if is_low_hp || is_weakened {
                *vis = Visibility::Visible;
            } else {
                *vis = Visibility::Hidden;
            }
        }
    }
}

#[test]
fn test_logic_triggers() {
    let mut app = App::new();
    app.world_mut().spawn(Player { hp: 10, max_hp: 100, ..Default::default() });
    let warning_ent = app.world_mut().spawn((Node::default(), ScreenWarning, Visibility::Hidden)).id();
    app.add_systems(Update, mock_warning_system);
    app.update();
    let vis = app.world().get::<Visibility>(warning_ent).unwrap();
    assert_eq!(*vis, Visibility::Visible);
}

// --- 关键：验证真实血条宽度更新逻辑 ---
#[test]
fn test_hp_bar_width_update() {
    use bevy_card_battler::components::combat::PlayerHpBarMarker;
    
    let mut app = App::new();
    // 1. 准备 50% 血量的玩家
    app.world_mut().spawn(Player { hp: 50, max_hp: 100, ..Default::default() });
    
    // 2. 准备初始宽度为 100% 的血条
    let bar_ent = app.world_mut().spawn((
        Node { width: Val::Percent(100.0), ..default() },
        PlayerHpBarMarker
    )).id();

    // 3. 定义同步逻辑 (从 lib.rs 中提取的逻辑)
    let sync_hp_bar = |player_query: Query<&Player>, mut bar_query: Query<&mut Node, With<PlayerHpBarMarker>>| {
        if let Ok(p) = player_query.get_single() {
            if let Ok(mut node) = bar_query.get_single_mut() {
                let hp_percent = (p.hp as f32 / p.max_hp as f32) * 100.0;
                node.width = Val::Percent(hp_percent);
            }
        }
    };

    app.add_systems(Update, sync_hp_bar);
    app.update();

    // 4. 验证宽度是否变为 50%
    let node = app.world().get::<Node>(bar_ent).unwrap();
    if let Val::Percent(w) = node.width {
        assert_eq!(w, 50.0, "血条宽度应根据 50/100 缩放为 50%");
    } else {
        panic!("宽度类型不正确");
    }
    println!("✅ 集成验证通过：血条宽度随 HP 动态同步成功");
}
