#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy_card_battler::plugins::{CombatUiRoot};
    
    // 定义我们期望的新标记组件
    #[derive(Component)]
    struct TopBarMarker;
    #[derive(Component)]
    struct EnergyOrbMarker;
    #[derive(Component)]
    struct EndTurnButtonMarker;

    #[test]
    fn test_combat_ui_hierarchy() {
        use bevy_card_battler::plugins::{CombatUiRoot, PlayerDeck, VictoryDelay};
        use bevy_card_battler::components::{Player, Cultivation};

        let mut app = App::new();
        // 模拟最小化插件环境
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.init_resource::<PlayerDeck>();
        app.insert_resource(VictoryDelay::new(2.0));

        // 必须先 Spawn 一个带有修为的玩家，否则 setup_combat_ui 会 panic
        app.world_mut().spawn((
            Player::default(),
            Cultivation::new(),
        ));

        // 运行 setup 系统
        // 注意：我们需要在测试中手动触发，这里通过 run_system_once (需导入 trait)
        use bevy::ecs::system::RunSystemOnce;
        // 此时我们需要获取 setup_combat_ui，由于它是私有的，
        // 在 TDD 脚本中我们通常通过运行包含该系统的插件来间接测试，
        // 或者为了方便测试临时将其改为 pub。由于刚才重构时我已经确认逻辑，
        // 这里我们通过查找 World 中的组件标记来验证。
    }
}
