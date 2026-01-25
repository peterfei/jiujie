#[cfg(test)]
mod tests {
    use bevy_card_battler::components::cards::{DeckConfig, CardType, CardPool};

    #[test]
    fn test_starting_deck_is_cultivation_themed() {
        // Arrange
        let deck_config = DeckConfig::default();
        let starting_deck = deck_config.starting_deck;

        // Assert
        assert_eq!(starting_deck.len(), 12, "初始牌组数量应为12张");

        let first_card = &starting_deck[0];
        assert_eq!(first_card.name, "御剑术", "初始攻击卡名称应为'御剑术'");
        
        let defense_card = &starting_deck[5];
        assert_eq!(defense_card.name, "金光咒", "初始防御卡名称应为'金光咒'");
    }

    #[test]
    fn test_card_pool_is_cultivation_themed() {
        let all_cards = CardPool::all_cards();
        
        // 验证卡池不为空
        assert!(!all_cards.is_empty());

        // 检查是否包含特定的修仙卡牌
        // 原"重击" -> "雷法·掌心雷"
        let heavy_attack = all_cards.iter().find(|c| c.id == 100).expect("ID 100 的卡牌应存在");
        assert_eq!(heavy_attack.name, "雷法·掌心雷", "ID 100 应为 '雷法·掌心雷'");

        // 原"铁壁" -> "不动明王"
        let heavy_defense = all_cards.iter().find(|c| c.id == 101).expect("ID 101 的卡牌应存在");
        assert_eq!(heavy_defense.name, "不动明王", "ID 101 应为 '不动明王'");

        // 原"旋风斩" -> "御剑·流云"
        let special_attack = all_cards.iter().find(|c| c.id == 200).expect("ID 200 的卡牌应存在");
        assert_eq!(special_attack.name, "御剑·流云", "ID 200 应为 '御剑·流云'");
    }
}

