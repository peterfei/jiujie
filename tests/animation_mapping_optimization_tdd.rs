use bevy::prelude::*;
use bevy_card_battler::components::sprite::{CharacterSprite, AnimationState, PlayerSpriteMarker, PlayerAnimationConfig, PhysicalImpact, ActionType};
use bevy_card_battler::systems::sprite::{sync_player_skeletal_animation};

#[test]
fn test_animation_mapping_optimization() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<AnimationClip>();
    app.init_asset::<AnimationGraph>();
    app.add_event::<bevy_card_battler::components::particle::SpawnEffectEvent>();
    app.add_event::<bevy_card_battler::components::ScreenEffectEvent>();
    app.insert_resource(bevy_card_battler::components::sprite::CharacterAssets::default());

    // 1. 设置资产索引
    let idle_node = AnimationNodeIndex::new(0);
    let kick_node = AnimationNodeIndex::new(1);
    let run_node = AnimationNodeIndex::new(2);
    let strike_node = AnimationNodeIndex::new(3);

    let graph_handle = app.world_mut().resource_mut::<Assets<AnimationGraph>>().add(AnimationGraph::new());

    // 2. 创建玩家实体
    let player_entity = app.world_mut().spawn((
        PlayerSpriteMarker,
        PhysicalImpact::default(),
        CharacterSprite::new(Handle::default(), Vec2::ONE),
        PlayerAnimationConfig {
            graph: graph_handle.clone(),
            idle_node,
            kick_node,
            run_node,
            strike_node,
        },
    )).id();

    // 创建子实体层级包含 AnimationPlayer
    let mut anim_player_entity = Entity::PLACEHOLDER;
    app.world_mut().entity_mut(player_entity).with_children(|parent| {
        anim_player_entity = parent.spawn(AnimationPlayer::default()).id();
    });

    app.add_systems(Update, sync_player_skeletal_animation);

    // --- 优化项 1: 引雷 (HeavenCast) 默认应为 Idle ---
    {
        let mut sprite = app.world_mut().get_mut::<CharacterSprite>(player_entity).unwrap();
        sprite.state = AnimationState::HeavenCast;
        
        // 我们需要模拟 HeavenCast 有主动动作，否则会被静止拦截器强制重置为 Idle
        let mut impact = app.world_mut().get_mut::<PhysicalImpact>(player_entity).unwrap();
        impact.action_type = ActionType::Ascend;
    }
    app.update();
    
    let player = app.world().get::<AnimationPlayer>(anim_player_entity).unwrap();
    assert!(player.is_playing_animation(idle_node), "引雷状态应当播放 Idle 动画节点");

    // --- 优化项 2: 万剑归宗 (ImperialSword) 应为 Kick ---
    {
        let mut sprite = app.world_mut().get_mut::<CharacterSprite>(player_entity).unwrap();
        sprite.state = AnimationState::ImperialSword;
    }
    app.update();
    
    let player = app.world().get::<AnimationPlayer>(anim_player_entity).unwrap();
    assert!(player.is_playing_animation(kick_node), "万剑归宗状态应当播放 Kick 动画节点");
}
