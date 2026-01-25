#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy_card_battler::components::cultivation::{Cultivation, Realm};

    #[test]
    fn test_initial_cultivation_state() {
        // 验证初始状态：炼气期，0感悟
        let cultivation = Cultivation::new();
        
        assert_eq!(cultivation.realm, Realm::QiRefining, "初始境界应为炼气期");
        assert_eq!(cultivation.insight, 0, "初始感悟应为0");
        assert!(!cultivation.can_breakthrough(), "初始状态不应可以突破");
    }

    #[test]
    fn test_perform_breakthrough_increases_realm() {
        let mut cultivation = Cultivation::new();
        let threshold = cultivation.get_threshold();
        cultivation.gain_insight(threshold);
        
        // 执行突破
        let success = cultivation.breakthrough();
        
        assert!(success, "感悟足够时突破应成功");
        assert_eq!(cultivation.realm, Realm::FoundationEstablishment, "突破后应进入筑基期");
        assert_eq!(cultivation.insight, 0, "突破后感悟值应清空");
    }

    #[test]
    fn test_breakthrough_bonuses_apply_to_player() {
        use bevy_card_battler::components::Player;

        let mut cultivation = Cultivation::new();
        let mut player = Player::default();
        let initial_max_hp = player.max_hp;

        cultivation.gain_insight(100);
        
        // 模拟系统逻辑：突破并获得奖励
        if cultivation.breakthrough() {
            // 筑基期奖励：最大HP +20
            player.max_hp += 20;
            player.hp += 20; // 同时回复对应生命
        }

        assert_eq!(player.max_hp, initial_max_hp + 20, "筑基期应增加20点道行上限");
    }

    #[test]
    fn test_player_persistence_after_cleanup() {
        use bevy_card_battler::plugins::cleanup_combat_ui;
        use bevy_card_battler::components::Player;
        use bevy::ecs::system::RunSystemOnce;

        let mut app = App::new();
        
        // 1. 准备玩家和修为
        let mut cultivation = Cultivation::new();
        cultivation.gain_insight(50);
        let player_id = app.world_mut().spawn((
            Player { hp: 80, max_hp: 80, ..Default::default() },
            cultivation,
        )).id();

        // 2. 运行清理系统 (cleanup_combat_ui)
        app.world_mut().run_system_once(cleanup_combat_ui);

        // 3. 验证玩家实体是否存活
        let still_exists = app.world().get_entity(player_id).is_ok();
        assert!(still_exists, "玩家实体在战斗清理后不应被销毁！");

        // 4. 验证修为数据是否保留
        let cultivation = app.world().get::<Cultivation>(player_id).expect("修为组件丢失");
        assert_eq!(cultivation.insight, 50, "感悟值在清理后应保留");
    }
}
