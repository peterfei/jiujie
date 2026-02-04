//! 蜘蛛实体与基础逻辑验证

use bevy::prelude::*;
use bevy_card_battler::components::*;
use bevy_card_battler::components::combat::{EnemyType, EnemyIntent};
use bevy_card_battler::states::GameState;

#[test]
fn test_spider_entity_initialization() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<GameState>();

    // 1. 创建蜘蛛敌人
    let mut spider = Enemy::with_type(1, "幻网蜘蛛", 30, EnemyType::PoisonSpider);
    // [修复语法] 使用结构体变体语法
    spider.intent = EnemyIntent::Attack { damage: 8 };
    
    let spider_ent = app.world_mut().spawn(spider).id();

    // 2. 验证基础属性
    let enemy = app.world().get::<Enemy>(spider_ent).unwrap();
    assert_eq!(enemy.name, "幻网蜘蛛");
    assert_eq!(enemy.hp, 30);
    
    if let EnemyIntent::Attack { damage } = enemy.intent {
        assert_eq!(damage, 8);
    } else {
        panic!("意图应该是 Attack");
    }

    println!("=== 蜘蛛基础 TDD 验证通过 ===");
}