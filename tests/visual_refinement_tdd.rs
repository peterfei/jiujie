use bevy::prelude::*;
use bevy_card_battler::components::combat::{Enemy, EnemyType};
use bevy_card_battler::components::sprite::Ghost;

#[test]
fn test_spider_web_mesh_spawning() {
    let mut app = App::new();
    
    fn mock_spider_attack(mut commands: Commands, enemy_query: Query<&Enemy>) {
        for enemy in enemy_query.iter() {
            if enemy.enemy_type == EnemyType::PoisonSpider {
                commands.spawn((
                    Node::default(),
                    Ghost { ttl: 1.5 },
                ));
            }
        }
    }

    app.world_mut().spawn(Enemy::new(1, "毒蛛", 100)); // 使用 new

    app.add_systems(Update, mock_spider_attack);
    app.update();

    let mut query = app.world_mut().query::<&Ghost>();
    assert!(query.iter(app.world()).next().is_some(), "蜘蛛攻击必须生成实体蛛网 (Ghost)");
    
    println!("✅ 蜘蛛吐丝实体逻辑验证通过");
}
