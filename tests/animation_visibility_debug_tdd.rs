use bevy::prelude::*;
use bevy_card_battler::components::sprite::{CharacterSprite, AnimationState, CharacterAnimationEvent, PlayerSpriteMarker, PlayerAnimationConfig, PlayerWeapon};
use bevy_card_battler::systems::sprite::{sync_player_skeletal_animation};

#[test]
fn test_visibility_and_animation_lock() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<AnimationClip>();
    app.init_asset::<AnimationGraph>();

    // 1. 设置资产
    let graph_handle = app.world_mut().resource_mut::<Assets<AnimationGraph>>().add(AnimationGraph::new());
    let idle_node = AnimationNodeIndex::new(0);
    let hit_node = AnimationNodeIndex::new(1);
    let attack_node = AnimationNodeIndex::new(2);

    // 2. 创建玩家实体
    let player_entity = app.world_mut().spawn((
        PlayerSpriteMarker,
        CharacterSprite::new(Handle::default(), Vec2::ONE),
        PlayerAnimationConfig {
            graph: graph_handle.clone(),
            idle_node,
            hit_node,
            attack_node,
        },
    )).id();

    // 3. 创建复杂的 GLB 层级 (模拟延迟加载出的模型结构)
    let mut weapon_entity = Entity::PLACEHOLDER;
    let mut anim_player_entity = Entity::PLACEHOLDER;

    app.world_mut().entity_mut(player_entity).with_children(|parent| {
        // 模拟骨骼根节点
        parent.spawn(SpatialBundle::default()).with_children(|bone_root| {
            // 动画播放器通常在模型深处
            anim_player_entity = bone_root.spawn(AnimationPlayer::default()).id();
            
            // 武器实体
            weapon_entity = bone_root.spawn((
                PlayerWeapon,
                VisibilityBundle::default(),
            )).id();
        });
    });

    app.update(); // 同步层级

    // 4. 模拟打出“万剑归宗”
    {
        let mut sprite = app.world_mut().get_mut::<CharacterSprite>(player_entity).unwrap();
        sprite.state = AnimationState::ImperialSword;
    }

    // 5. 运行同步系统
    app.add_systems(Update, sync_player_skeletal_animation);
    app.update();

    // --- 🔴 核心验证：显隐锁死 ---
    let vis = app.world().get::<Visibility>(weapon_entity).expect("武器应有 Visibility");
    assert_eq!(*vis, Visibility::Hidden, "大招期间，深层嵌套的武器实体必须隐藏");

    // --- 🟢 核心验证：动画驱动 ---
    let anim_player = app.world().get::<AnimationPlayer>(anim_player_entity).expect("应有播放器");
    assert!(anim_player.is_playing_animation(attack_node), "应正在播放攻击动画节点");

    // 6. 模拟“受击” (优先级测试)
    {
        let mut sprite = app.world_mut().get_mut::<CharacterSprite>(player_entity).unwrap();
        sprite.state = AnimationState::Hit;
    }
    app.update();
    
    let anim_player_hit = app.world().get::<AnimationPlayer>(anim_player_entity).unwrap();
    assert!(anim_player_hit.is_playing_animation(hit_node), "受击时应切换到受击动画");
}
