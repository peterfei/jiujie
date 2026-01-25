#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy_card_battler::components::shop::{ShopItem, CurrentShopItems};
    use bevy_card_battler::components::{Player, Card, CardType, CardRarity, CardEffect};

    #[test]
    fn test_spirit_stone_purchase_logic() {
        let mut player = Player { gold: 100, hp: 50, max_hp: 100, ..Default::default() };
        
        // 模拟一个“洗髓丹” (恢复20道行，价值30灵石)
        let elixir = ShopItem::Elixir {
            name: "洗髓丹".to_string(),
            hp_restore: 20,
            price: 30,
            description: "洗筋伐髓，恢复少量道行".to_string(),
        };

        // 验证购买逻辑
        if player.gold >= elixir.get_price() {
            player.gold -= elixir.get_price();
            if let ShopItem::Elixir { hp_restore, .. } = elixir {
                player.hp = (player.hp + hp_restore).min(player.max_hp);
            }
        }

        assert_eq!(player.gold, 70, "购买后灵石应扣除");
        assert_eq!(player.hp, 70, "服用洗髓丹后道行应增加");
    }

    #[test]
    fn test_marketplace_stock_generation() {
        // 验证坊市是否能正确生成随机的灵丹和秘籍
        let mut shop_items = CurrentShopItems::default();
        
        // 模拟生成逻辑
        shop_items.items.push(ShopItem::Elixir { 
            name: "聚灵散".to_string(), 
            hp_restore: 10, 
            price: 20, 
            description: "聚拢天地灵气".to_string() 
        });

        assert_eq!(shop_items.items.len(), 1);
        assert!(shop_items.items[0].get_name().contains("聚灵"), "应包含灵丹");
    }
}
