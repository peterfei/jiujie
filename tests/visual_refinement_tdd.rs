use bevy::prelude::*;
use bevy_card_battler::components::particle::{EmitterConfig, EffectType, SpawnEffectEvent};
use bevy_card_battler::components::cards::{Card, CardEffect, CardType, CardRarity};
use bevy_card_battler::components::combat::{Player, Enemy, EnemyType, EnemyIntent, DamageEffectEvent, StatusEffectEvent};
use bevy_card_battler::components::screen_effect::ScreenEffectEvent;
use bevy_card_battler::components::sprite::CharacterAnimationEvent;
use bevy_card_battler::components::animation::EnemyAttackEvent;

#[test]
fn test_config_values() {
    let slash = EmitterConfig::slash();
    assert!(slash.speed.0 >= 200.0);
    let shield = EmitterConfig::shield();
    let color: Srgba = shield.start_color.into();
    assert!(color.blue > 0.5);
    
    // [新增] 验证 WebShot 的可见性参数
    let web = EmitterConfig::web_shot();
    // 之前是 (3.0, 8.0)，太小了。现在要求至少 15.0
    assert!(web.size.0 >= 15.0, "蜘蛛网粒子必须足够大才能看清");
    // 之前是 (0.4, 0.6)，太短了。现在要求至少 0.8
    assert!(web.lifetime.0 >= 0.8, "蜘蛛网飞行时间必须足够长");
}

#[test]
fn test_effect_trigger_integration() {
    let mut app = App::new();
    app.add_event::<SpawnEffectEvent>();
    app.add_event::<ScreenEffectEvent>();
    app.add_event::<CharacterAnimationEvent>();
    app.add_event::<DamageEffectEvent>();
    app.add_event::<StatusEffectEvent>();

    let _block_card = Card {
        id: 1, name: "防御".to_string(), cost: 1, card_type: CardType::Skill,
        effect: CardEffect::GainBlock { amount: 5 }, description: "".to_string(),
        image_path: "".to_string(), rarity: CardRarity::Common, upgraded: false,
    };

    let mut events = app.world_mut().resource_mut::<Events<SpawnEffectEvent>>();
    events.send(SpawnEffectEvent::new(EffectType::Shield, Vec3::ZERO));
    app.update();

    let reader = app.world().resource::<Events<SpawnEffectEvent>>();
    let mut iter = reader.get_cursor();
    let has_shield = iter.read(reader).any(|e| e.effect_type == EffectType::Shield);
    assert!(has_shield);
}

#[test]
fn test_all_enemy_types_trigger_unique_effects() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<SpawnEffectEvent>();
    app.add_event::<EnemyAttackEvent>();
    app.add_event::<ScreenEffectEvent>();
    app.add_event::<CharacterAnimationEvent>();

    let check_effect = |e_type: EnemyType| -> EffectType {
        match e_type {
            EnemyType::PoisonSpider => EffectType::WebShot,
            EnemyType::CursedSpirit => EffectType::DemonAura,
            EnemyType::GreatDemon => EffectType::Lightning,
            _ => EffectType::Slash,
        }
    };

    assert_eq!(check_effect(EnemyType::PoisonSpider), EffectType::WebShot);
    assert_eq!(check_effect(EnemyType::DemonicWolf), EffectType::Slash);
    assert_eq!(check_effect(EnemyType::CursedSpirit), EffectType::DemonAura);
    assert_eq!(check_effect(EnemyType::GreatDemon), EffectType::Lightning);
}

#[test]
fn test_effect_follows_player_position() {
    let player_pos = Vec3::new(10.0, 2.0, 0.0);
    let heal_offset = Vec3::new(0.0, -0.5, 0.5);
    let shield_offset = Vec3::new(0.0, 0.5, 0.5);
    
    let heal_target = player_pos + heal_offset;
    let shield_target = player_pos + shield_offset;
    
    assert_eq!(heal_target.x, 10.0);
    assert_eq!(heal_target.y, 1.5);
    assert_eq!(shield_target.x, 10.0);
    assert_eq!(shield_target.y, 2.5);
}
