use bevy_card_battler::components::cards::{create_starting_deck};

mod tests {
    use super::*;

    #[test]
    fn test_starting_deck_is_cultivation_themed() {
        let deck = create_starting_deck();
        assert_eq!(deck.len(), 15, "初始牌组数量应为15张");
        
        let has_cultivation_flavor = deck.iter().any(|c| 
            c.name.contains("剑") || 
            c.name.contains("刺") || 
            c.name.contains("护") || 
            c.name.contains("盾") || 
            c.name.contains("步") ||
            c.name.contains("雷")
        );
        assert!(has_cultivation_flavor, "初始牌组应包含修仙元素的卡牌");
    }
}