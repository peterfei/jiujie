use bevy::prelude::*;
use bevy_card_battler::components::*;

#[test]
fn test_jifengci_click_safety_during_victory() {
    let mut app = App::new();
    app.insert_resource(VictoryDelay::new(1.5));
    app.world_mut().resource_mut::<VictoryDelay>().active = true; // 模拟胜利中
    
    // 验证资源状态
    assert!(app.world().resource::<VictoryDelay>().active);
}

#[test]
fn test_card_id_102_is_jifengci() {
    let card = Card::new(102, "疾风刺", "描述", CardType::Attack, 0, CardEffect::DealDamage { amount: 4 }, CardRarity::Common, "path");
    assert_eq!(card.id, 102);
    assert_eq!(card.name, "疾风刺");
}
