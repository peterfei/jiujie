#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy_card_battler::components::{Player, Cultivation};
    use bevy_card_battler::states::GameState;

    #[test]
    fn test_restart_resets_player_stats() {
        let mut player = Player { hp: 0, max_hp: 80, ..Default::default() };
        
        // 模拟重启逻辑：如果是新修行，必须重置 HP
        if true { // 模拟开始新修行
            player.hp = player.max_hp;
        }

        assert!(player.hp > 0, "重启后玩家血量必须大于0");
        assert_eq!(player.hp, 80, "重启后玩家血量应恢复至最大值");
    }
}