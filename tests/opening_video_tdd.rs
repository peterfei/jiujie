use bevy::prelude::*;
use bevy_card_battler::states::GameState;
use bevy_card_battler::plugins::opening::{OpeningPlugin, OpeningPlayedLock};

/// 计数器：统计进入状态的次数
#[derive(Resource, Default)]
struct EntryCounter(u32);

#[test]
fn reproduce_double_play_bug() {
    let mut app = App::new();
    
    // 1. 环境准备
    app.add_plugins((
        bevy::state::app::StatesPlugin,
        bevy::time::TimePlugin,
        bevy::input::InputPlugin,
        OpeningPlugin,
    ));
    app.init_state::<GameState>();
    app.init_resource::<EntryCounter>();
    
    // 模拟缺失的 BGM 事件防止崩溃
    app.add_event::<bevy_card_battler::components::background_music::PlayBgmEvent>();
    // 模拟 AssetServer 环境 (虽然我们不跑真实加载系统，但需要它存在)
    app.init_resource::<bevy::asset::Assets<Image>>();

    // 2. 注册计数系统：每当进入 OpeningVideo，计数器 +1
    app.add_systems(OnEnter(GameState::OpeningVideo), |mut count: ResMut<EntryCounter>| {
        count.0 += 1;
        println!("【Test】检测到进入 OpeningVideo 状态，当前总次数: {}", count.0);
    });

    // 3. 模拟应用启动逻辑（手动设置初始重定向）
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::OpeningVideo);
    
    // 运行几帧模拟初始化
    app.update(); // 应用 NextState
    app.update(); // 触发 OnEnter(OpeningVideo) -> 计数应为 1

    // 4. 模拟视频播放结束后的正常跳转
    println!("【Test】模拟播放结束，请求跳转到 MainMenu");
    app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::MainMenu);
    app.update(); // 应用转换
    
    assert_eq!(
        *app.world().resource::<State<GameState>>().get(), 
        GameState::MainMenu, 
        "正常跳转后应处于 MainMenu"
    );

    // 5. 关键步骤：在 MainMenu 运行几帧，检查是否有“回跳”
    for i in 0..10 {
        app.update();
        let current_count = app.world().resource::<EntryCounter>().0;
        if current_count > 1 {
            panic!("🔥 [BUG复现] 检测到状态回跳！视频播放了 {} 次，发生在第 {} 帧测试", current_count, i);
        }
    }
    
    println!("【Test】集成测试验证通过：视频仅播放了 1 次。");
}