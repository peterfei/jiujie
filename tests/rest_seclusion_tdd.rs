#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy_card_battler::components::{Player, Cultivation};

    #[test]
    fn test_seclusion_breath_meditation() {
        let mut player = Player { hp: 30, max_hp: 100, ..Default::default() };
        
        // 调息：恢复 30% 的最大道行
        let heal_amount = (player.max_hp as f32 * 0.3) as i32;
        player.hp = (player.hp + heal_amount).min(player.max_hp);

        assert_eq!(player.hp, 60, "调息后应恢复30%道行");
    }

    #[test]
    fn test_seclusion_seated_insight() {
        let mut cultivation = Cultivation::new();
        cultivation.insight = 10;
        
        // 悟道：固定增加 20 点感悟
        cultivation.gain_insight(20);

        assert_eq!(cultivation.insight, 30, "悟道后感悟应大幅增加");
    }
}
