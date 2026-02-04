#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::ecs::system::RunSystemOnce; // 新增导入
    use bevy_card_battler::components::sprite::{
        CharacterAnimationEvent, AnimationState, PhysicalImpact, SpriteMarker, CharacterSprite, 
        SpiritClone, Combatant3d, EnemySpriteMarker, CharacterAssets
    };
    use bevy_card_battler::systems::sprite::{
        update_physical_impacts, handle_animation_events, update_spirit_clones, 
        trigger_hit_feedback
    };
    use bevy_card_battler::components::particle::SpawnEffectEvent;
    
    #[derive(Resource)]
    struct VictoryDelay(f32);

    #[test]
    fn test_spirit_attack_4_clones_and_explosion() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        
        app.add_event::<CharacterAnimationEvent>();
        app.add_event::<SpawnEffectEvent>();
        app.add_event::<bevy_card_battler::components::ScreenEffectEvent>();
        
        app.init_resource::<Time>();
        app.init_resource::<Assets<Image>>();
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.insert_resource(CharacterAssets::default());

        // 1. 生成实体
        let spirit_entity = app.world_mut().spawn((
            Transform::from_translation(Vec3::new(3.5, 0.8, 0.0)),
            PhysicalImpact::default(),
            SpriteMarker,
            CharacterSprite::new(Handle::default(), Vec2::new(100.0, 100.0)),
            EnemySpriteMarker { id: 1 },
            Combatant3d { facing_right: false }, 
        )).id();

        // 2. 发送事件
        app.world_mut().send_event(CharacterAnimationEvent {
            target: spirit_entity,
            animation: AnimationState::SpiritAttack,
        });

        // 3. 手动运行系统逻辑
        let mut explosion_detected = false;
        
        // 运行足够长的循环
        for i in 0..200 {
            // 推进时间
            app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(16));
            
            // 只有第一帧运行反馈系统
            if i == 0 {
                let _ = app.world_mut().run_system_once(trigger_hit_feedback);
            }
            // 每一帧运行更新系统
            let _ = app.world_mut().run_system_once(update_spirit_clones);
            
            // 运行 app.update() 处理内部清理和事件缓冲
            app.update();

            // 验证分身是否生成且分散
            if i == 5 {
                let mut current_clones = 0;
                let mut positions = Vec::new();
                let mut clone_query = app.world_mut().query::<(&SpiritClone, &Transform)>();
                for (_, transform) in clone_query.iter(app.world()) {
                    current_clones += 1;
                    positions.push(transform.translation);
                }
                assert!(current_clones >= 4 && current_clones <= 6, "应当随机生成 4 到 6 个分身");
                let dist = positions[0].distance(positions[1]);
                assert!(dist > 50.0, "分身位置应当分散");
            }

            let events = app.world().resource::<Events<SpawnEffectEvent>>();
            let mut reader = events.get_cursor();
            for event in reader.read(events) {
                if event.count >= 10 {
                    explosion_detected = true;
                }
            }
            if explosion_detected { break; }
        }

        assert!(explosion_detected, "手动驱动系统后应当检测到粒子爆发事件");
    }
}