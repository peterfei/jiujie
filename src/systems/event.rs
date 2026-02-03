use bevy::prelude::*;
use crate::states::GameState;
use crate::components::{Player, MapProgress, RelicCollection, PlayerDeck};

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Event), setup_event_ui)
           .add_systems(Update, handle_event_choices.run_if(in_state(GameState::Event)))
           .add_systems(OnExit(GameState::Event), cleanup_event_ui);
    }
}

#[derive(Component)]
pub struct EventUiRoot;

#[derive(Component, Debug)]
pub enum EventChoiceButton {
    GainGold(i32),
    Heal(i32),
    Leave,
}

/// 设置机缘事件界面
fn setup_event_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    info!("【机缘】进入奇遇事件");
    let font = asset_server.load("fonts/Arial Unicode.ttf");

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(30.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.05, 0.1, 0.9)),
        EventUiRoot,
    )).with_children(|parent| {
        // 标题
        parent.spawn((
            Text::new("古 修 遗 迹"),
            TextFont { font: font.clone(), font_size: 48.0, ..default() },
            TextColor(Color::srgb(0.4, 0.8, 1.0)),
        ));

        // 描述
        parent.spawn((
            Text::new("你在一处断崖下发现了一尊古老的石像，石像手中握着一颗微弱发光的灵石，而基座上似乎刻着某种愈合咒文。"),
            Node { max_width: Val::Px(600.0), ..default() },
            TextFont { font: font.clone(), font_size: 20.0, ..default() },
            TextColor(Color::WHITE),
            TextLayout::new_with_justify(JustifyText::Center),
        ));

        // 选项区
        parent.spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(15.0),
            ..default()
        }).with_children(|choices| {
            // 选项 1: 拿走灵石
            create_event_button(choices, "取走灵石 (+50 灵石)", EventChoiceButton::GainGold(50), font.clone());
            // 选项 2: 虔诚祈祷
            create_event_button(choices, "虔诚祈祷 (回复 20 HP)", EventChoiceButton::Heal(20), font.clone());
            // 选项 3: 离去
            create_event_button(choices, "因果莫测，径直离去", EventChoiceButton::Leave, font.clone());
        });
    });
}

fn create_event_button(parent: &mut ChildBuilder, label: &str, choice: EventChoiceButton, font: Handle<Font>) {
    parent.spawn((
        Button,
        Node {
            width: Val::Px(400.0),
            height: Val::Px(50.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.2, 0.2, 0.3, 0.8)),
        BorderRadius::all(Val::Px(8.0)),
        choice,
    )).with_children(|p| {
        p.spawn((
            Text::new(label),
            TextFont { font, font_size: 18.0, ..default() },
            TextColor(Color::WHITE),
        ));
    });
}

/// 处理机缘事件选择
fn handle_event_choices(
    mut commands: Commands,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut player_query: Query<(&mut Player, &crate::components::Cultivation)>,
    mut map_progress: ResMut<MapProgress>,
    _deck: Res<PlayerDeck>,
    _relics: Res<RelicCollection>,
    button_query: Query<(&Interaction, &EventChoiceButton), Changed<Interaction>>,
) {
    if *current_state.get() != GameState::Event {
        return;
    }

    for (interaction, choice) in button_query.iter() {
        if *interaction == Interaction::Pressed {
            // 防止单帧内多次设置状态
            if next_state.is_changed() { continue; }

            if let Ok((mut player, _)) = player_query.get_single_mut() {
                match choice {
                    EventChoiceButton::GainGold(amt) => {
                        player.gold += *amt;
                        info!("【机缘】获得灵石 {}", amt);
                    }
                    EventChoiceButton::Heal(amt) => {
                        player.hp = (player.hp + *amt).min(player.max_hp);
                        info!("【机缘】回复生命 {}", amt);
                    }
                    EventChoiceButton::Leave => {
                        info!("【机缘】悄然离去");
                    }
                }
            }

            // 完成当前节点
            map_progress.complete_current_node();
            info!("【机缘】事件节点已完成，下一层已解锁");

            // 清理逻辑现在由 OnExit 处理
            next_state.set(GameState::Map);
        }
    }
}

fn cleanup_event_ui(
    mut commands: Commands,
    query: Query<Entity, With<EventUiRoot>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    info!("【机缘】UI 已清理");
}
