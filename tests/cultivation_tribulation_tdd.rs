#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy_card_battler::components::cultivation::{Cultivation, Realm};
    use bevy_card_battler::components::Player;

    #[test]
    fn test_tribulation_survival_logic() {
        let mut cultivation = Cultivation::new();
        let mut player = Player { hp: 100, max_hp: 100, ..Default::default() };
        
        // 准备渡劫
        cultivation.gain_insight(100);
        assert!(cultivation.can_breakthrough());
        
        // 模拟雷劫伤害：5次，每次15点
        for _ in 0..5 {
            player.hp -= 15;
        }
        
        // 检查存活
        assert!(player.hp > 0, "玩家应在雷劫中存活");
        
        // 存活后正式突破
        if cultivation.breakthrough() {
            player.max_hp += cultivation.get_hp_bonus();
            player.hp += cultivation.get_hp_bonus();
        }
        
        assert_eq!(cultivation.realm, Realm::FoundationEstablishment);
        assert_eq!(player.max_hp, 120);
    }
}
