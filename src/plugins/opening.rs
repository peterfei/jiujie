use bevy::prelude::*;
use crate::states::GameState;
use crate::components::background_music::{BgmType, PlayBgmEvent};

pub struct OpeningPlugin;

/// 用于启动预热的第一帧资源
#[derive(Resource)]
pub struct FirstFrameResource(pub Handle<Image>);

impl Plugin for OpeningPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::OpeningVideo), setup_opening_video);
        app.add_systems(Update, video_playback_system.run_if(in_state(GameState::OpeningVideo)));
        app.add_systems(OnExit(GameState::OpeningVideo), cleanup_opening_video);
    }
}

/// 全局单次运行锁
#[derive(Resource, Default)]
struct GlobalOpeningLock(bool);

#[derive(Resource)]
struct VideoController {
    frame_timer: Timer,
    current_index: usize,
    handles: Vec<Handle<Image>>,
    is_playing: bool,
}

#[derive(Component)]
struct OpeningVideoMarker;

#[derive(Component)]
struct VideoFrame;

#[derive(Resource, Default)]
struct SkipOpening(bool);

fn setup_opening_video(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    bgm_events: EventWriter<PlayBgmEvent>,
    first_frame_res: Res<FirstFrameResource>,
    mut global_lock: Local<bool>,
) {
    // 1. 物理拦截
    if *global_lock {
        return;
    }
    *global_lock = true;

    println!("【Opening】初始化播放器...");

    // 使用预热好的第一帧
    let first_frame_handle = first_frame_res.0.clone();

    // 2. 预加载其他帧
    let total_frames = 120;
    let mut handles = Vec::with_capacity(total_frames);
    for i in 1..=total_frames {
        handles.push(asset_server.load(format!("video/frames/frame_{:03}.jpg", i)));
    }

    commands.init_resource::<SkipOpening>();
    commands.insert_resource(VideoController {
        frame_timer: Timer::from_seconds(1.0 / 15.0, TimerMode::Repeating),
        current_index: 0,
        handles,
        is_playing: false,
    });

    // 3. 构建 UI (由于资源已就绪，这里会瞬间显示图像)
    commands.spawn((
        Node {
            width: Val::Vw(100.0),
            height: Val::Vh(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::BLACK),
        OpeningVideoMarker,
    )).with_children(|parent| {
        // 视频帧
        parent.spawn((
            ImageNode::new(first_frame_handle),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            VideoFrame,
        ));

        // 提示
        parent.spawn((
            Text::new("按 [ 空格 ] 跳过"),
            TextFont {
                font: asset_server.load("fonts/Arial Unicode.ttf"),
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.1)),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(30.0),
                ..default()
            },
        ));
    });
}

fn video_playback_system(
    mut commands: Commands,
    time: Res<Time>,
    controller: Option<ResMut<VideoController>>,
    mut query: Query<&mut ImageNode, With<VideoFrame>>,
    asset_server: Res<AssetServer>,
    mut bgm_events: EventWriter<PlayBgmEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    skip: Res<SkipOpening>,
) {
    let Some(mut ctl) = controller else { return; };

    // 跳过
    if keyboard.just_pressed(KeyCode::Space) || skip.0 {
        commands.remove_resource::<VideoController>();
        next_state.set(GameState::MainMenu);
        return;
    }

    // 1. 启动检查 (Loading -> Playing)
    if !ctl.is_playing {
        let mut ready_to_play = false;
        let mut first_frame_handle = Handle::default();

        // 阶段一：只读检查
        if let Some(handle) = ctl.handles.first() {
            if matches!(asset_server.get_load_state(handle.id()), Some(bevy::asset::LoadState::Loaded)) {
                ready_to_play = true;
                first_frame_handle = handle.clone();
            }
        }

        // 阶段二：可变修改 (此时阶段一的借用已结束)
        if ready_to_play {
            println!("【Opening】资源就绪 -> 开始播放");
            ctl.is_playing = true;
            bgm_events.send(PlayBgmEvent::new(BgmType::Opening).with_fade_in(0.1));
            // 立即显示第一帧
            if let Ok(mut node) = query.get_single_mut() {
                node.image = first_frame_handle;
            }
        }
        return; // 等待加载，不走下面的逻辑
    }

    // 播放逻辑
    ctl.frame_timer.tick(time.delta());
    
    if ctl.frame_timer.just_finished() {
        let next_idx = ctl.current_index + 1;

        if next_idx >= ctl.handles.len() {
            println!("【Opening】播放结束，进入主菜单");
            commands.remove_resource::<VideoController>();
            next_state.set(GameState::MainMenu);
            return;
        }

        let next_handle = &ctl.handles[next_idx];
        if matches!(asset_server.get_load_state(next_handle.id()), Some(bevy::asset::LoadState::Loaded)) {
            if let Ok(mut node) = query.get_single_mut() {
                node.image = next_handle.clone();
            }
            ctl.current_index = next_idx;
        }
    }
}

fn cleanup_opening_video(
    mut commands: Commands,
    query: Query<Entity, With<OpeningVideoMarker>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<VideoController>();
}