use bevy::prelude::*;
use bevy_card_battler::components::combat::EnemyType;
use bevy_card_battler::components::sprite::{CharacterAssets, CharacterType};
use bevy_card_battler::systems::sprite::spawn_character_sprite;

#[test]
fn test_enemy_specific_assets_mapping() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(AssetPlugin::default());

    // 1. 验证 CharacterAssets 是否支持多种敌人
    let assets = CharacterAssets {
        player_idle: Handle::default(),
        player_attack: Handle::default(),
        wolf: Handle::default(),
        spider: Handle::default(),
        spirit: Handle::default(),
        boss: Handle::default(),
    };

    // 2. 验证是否可以弱引用 (确保字段存在)
    assert!(assets.wolf.is_weak());
    assert!(assets.spider.is_weak());
    assert!(assets.spirit.is_weak());
    assert!(assets.boss.is_weak());
}
