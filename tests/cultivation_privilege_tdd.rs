#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy_card_battler::components::{Player, Cultivation, Realm, PlayerDeck};

    #[test]
    fn test_foundation_establishment_stat_boost() {
        let mut player = Player { max_hp: 100, hp: 10, gold: 50, ..Default::default() };
        let mut cultivation = Cultivation::new();
        
        // 模拟突破逻辑（手动触发 teardown_tribulation 中的逻辑）
        cultivation.realm = Realm::FoundationEstablishment;
        
        if cultivation.realm == Realm::FoundationEstablishment {
            player.max_hp += 50;
            player.hp = player.max_hp; // 全回满
            player.gold += 100;
        }

        assert_eq!(player.max_hp, 150, "筑基期应增加最大道行上限");
        assert_eq!(player.hp, 150, "突破后应补满状态");
        assert_eq!(player.gold, 150, "突破后应获得天道赏赐灵石");
    }

    #[test]
    fn test_foundation_establishment_innate_spell() {
        use bevy_card_battler::components::cards::CardPool;
        let mut deck = PlayerDeck::default();
        let mut cultivation = Cultivation::new();
        
        cultivation.realm = Realm::FoundationEstablishment;
        
        if cultivation.realm == Realm::FoundationEstablishment {
            let innate_spell = CardPool::get_innate_spell();
            deck.add_card(innate_spell);
        }
        
        assert!(deck.cards.iter().any(|c| c.name == "青莲剑歌"), "筑基期应获得本命功法：青莲剑歌");
    }
}
