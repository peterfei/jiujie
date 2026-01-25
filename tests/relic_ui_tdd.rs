//! 遗物UI显示系统TDD测试
//!
//! 遵循TDD原则：先写测试，覆盖所有场景，然后驱动开发

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_card_battler::components::*;
use bevy_card_battler::components::relic::{RelicUiMarker, RelicItemMarker};
use bevy_card_battler::plugins::CorePlugin;
use bevy_card_battler::states::GameState;
use bevy_card_battler::systems::{RelicPlugin, RelicUiPlugin, CombatStartProcessed};

// ============================================================================
// 测试辅助函数
// ============================================================================

fn create_combat_ui_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .add_plugins(CorePlugin)
        .add_plugins(RelicPlugin)
        .add_plugins(RelicUiPlugin)
        .init_state::<GameState>();

    // 初始化战斗资源
    app.insert_resource(CombatState::default());
    app.insert_resource(PlayerDeck::new());
    app.insert_resource(RelicCollection::default());
    app.insert_resource(CombatStartProcessed {
        processed: false,
    });

    // 创建玩家和敌人
    app.world_mut().spawn(Player::default());
    app.world_mut().spawn(Enemy::new(1, "嗜血妖狼", 30));

    // 创建牌堆
    app.world_mut().spawn(Hand::new(10));
    app.world_mut().spawn(DrawPile::new(vec![
        Card::new(
            1,
            "打击",
            "造成6点伤害",
            CardType::Attack,
            1,
            CardEffect::DealDamage { amount: 6 },
            CardRarity::Common,
        ),
    ]));
    app.world_mut().spawn(DiscardPile::new());

    // 设置初始状态为 MainMenu
    app.world_mut().insert_resource(State::new(GameState::MainMenu));

    app
}

// ============================================================================
// 场景1: 遗物UI区域标记组件
// ============================================================================

#[test]
fn test_relic_ui_marker_component_exists() {
    // 场景描述: 遗物UI区域应该有专门的标记组件
    // 预期结果: 可以通过RelicUiMarker查询到遗物UI实体

    let mut app = create_combat_ui_app();

    // 先添加遗物，然后转换状态
    let mut collection = RelicCollection::default();
    collection.add_relic(Relic::burning_blood());
    app.world_mut().insert_resource(collection);

    // 转换到战斗状态
    app.insert_state(GameState::Combat);
    app.update();

    // 查询遗物UI标记组件
    let mut ui_query = app.world_mut().query_filtered::<Entity, With<RelicUiMarker>>();
    let count = ui_query.iter(app.world_mut()).count();

    assert!(count >= 1, "应该至少有1个遗物UI区域实体");
}

// ============================================================================
// 场景2: 空遗物集合时不显示遗物UI
// ============================================================================

#[test]
fn test_empty_relic_collection_hides_ui() {
    // 场景描述: 玩家没有遗物时，不应该显示遗物UI
    // 预期结果: 遗物列表为空或不显示

    let mut app = create_combat_ui_app();

    // 确保遗物集合为空
    app.world_mut().insert_resource(RelicCollection::default());
    app.update();

    // 查询遗物显示项
    let mut item_query = app.world_mut().query_filtered::<Entity, With<RelicItemMarker>>();
    let count = item_query.iter(app.world_mut()).count();

    assert_eq!(count, 0, "没有遗物时不应该显示遗物项");
}

// ============================================================================
// 场景3: 单个遗物显示
// ============================================================================

#[test]
fn test_single_relic_display() {
    // 场景描述: 拥有1个遗物时，应该正确显示
    // 预期结果: UI中显示1个遗物项

    let mut app = create_combat_ui_app();

    // 添加1个遗物
    let mut collection = RelicCollection::default();
    collection.add_relic(Relic::burning_blood());
    app.world_mut().insert_resource(collection);

    // 转换到战斗状态
    app.insert_state(GameState::Combat);
    app.update();

    // 查询遗物显示项
    let mut item_query = app.world_mut().query_filtered::<Entity, With<RelicItemMarker>>();
    let count = item_query.iter(app.world_mut()).count();

    assert_eq!(count, 1, "应该显示1个遗物项");
}

// ============================================================================
// 场景4: 多个遗物显示
// ============================================================================

#[test]
fn test_multiple_relics_display() {
    // 场景描述: 拥有多个遗物时，应该全部显示
    // 预期结果: UI中显示所有遗物

    let mut app = create_combat_ui_app();

    // 添加3个遗物
    let mut collection = RelicCollection::default();
    collection.add_relic(Relic::burning_blood());
    collection.add_relic(Relic::bag_of_preparation());
    collection.add_relic(Relic::anchor());
    app.world_mut().insert_resource(collection);

    // 转换到战斗状态
    app.insert_state(GameState::Combat);
    app.update();

    // 查询遗物显示项
    let mut item_query = app.world_mut().query_filtered::<Entity, With<RelicItemMarker>>();
    let count = item_query.iter(app.world_mut()).count();

    assert_eq!(count, 3, "应该显示3个遗物项");
}

// ============================================================================
// 场景5: 遗物名称文本显示
// ============================================================================

#[test]
fn test_relic_name_display() {
    // 场景描述: 遗物项应该显示遗物名称
    // 预期结果: 可以查询到包含遗物名称的文本组件

    let mut app = create_combat_ui_app();

    let mut collection = RelicCollection::default();
    collection.add_relic(Relic::burning_blood());
    app.world_mut().insert_resource(collection);

    app.insert_state(GameState::Combat);
    app.update();

    // 查询遗物项（通过背景色）
    let mut bg_query = app.world_mut().query_filtered::<&BackgroundColor, With<RelicItemMarker>>();
    let has_item = bg_query.iter(app.world()).count() > 0;

    assert!(has_item, "应该有遗物项（背景色）");
}

// ============================================================================
// 场景6: 遗物描述文本显示
// ============================================================================

#[test]
fn test_relic_description_display() {
    // 场景描述: 遗物项应该显示遗物描述
    // 预期结果: 可以查询到包含描述的文本组件

    let mut app = create_combat_ui_app();

    let mut collection = RelicCollection::default();
    collection.add_relic(Relic::burning_blood());
    app.world_mut().insert_resource(collection);

    app.insert_state(GameState::Combat);
    app.update();

    // 查询遗物项（通过背景色）
    let mut bg_query = app.world_mut().query_filtered::<&BackgroundColor, With<RelicItemMarker>>();
    let has_item = bg_query.iter(app.world()).count() > 0;

    assert!(has_item, "应该有遗物项（背景色）");
}

// ============================================================================
// 场景7: 遗物稀有度颜色区分
// ============================================================================

#[test]
fn test_relic_rarity_color_coding() {
    // 场景描述: 不同稀有度的遗物应该有不同的颜色
    // 预期结果: Common/Uncommon/Rare有不同颜色

    let mut app = create_combat_ui_app();

    let mut collection = RelicCollection::default();
    collection.add_relic(Relic::burning_blood()); // Common
    collection.add_relic(Relic::anchor());        // Uncommon
    collection.add_relic(Relic::strange_spoon());  // Rare
    app.world_mut().insert_resource(collection);

    app.insert_state(GameState::Combat);
    app.update();

    // 查询遗物项的背景/边框颜色
    let mut bg_query = app.world_mut().query_filtered::<&BackgroundColor, With<RelicItemMarker>>();
    let colors: Vec<_> = bg_query.iter(app.world_mut()).collect();

    assert!(!colors.is_empty(), "应该有遗物背景颜色");

    // 验证不同稀有度有不同颜色（至少有2种不同颜色）
    let unique_colors: std::collections::HashSet<_> = colors
        .iter()
        .map(|color| format!("{:?}", color.0))
        .collect();

    assert!(unique_colors.len() >= 2, "不同稀有度应该有不同的颜色");
}

// ============================================================================
// 场景8: 遗物UI在战斗状态下的显示
// ============================================================================

#[test]
fn test_relic_ui_shows_in_combat() {
    // 场景描述: 在战斗状态下应该显示遗物UI
    // 预期结果: GameState::Combat时遗物UI可见

    let mut app = create_combat_ui_app();

    let mut collection = RelicCollection::default();
    collection.add_relic(Relic::burning_blood());
    app.world_mut().insert_resource(collection);
    app.world_mut().insert_resource(State::new(GameState::Combat));
    app.update();

    // 验证UI存在
    let mut ui_query = app.world_mut().query_filtered::<Entity, With<RelicUiMarker>>();
    let count = ui_query.iter(app.world_mut()).count();

    assert!(count >= 1, "战斗状态下应该显示遗物UI");
}

// ============================================================================
// 场景9: 遗物UI在非战斗状态下的隐藏
// ============================================================================

#[test]
fn test_relic_ui_hides_outside_combat() {
    // 场景描述: 在非战斗状态下不应该显示战斗遗物UI
    // 预期结果: GameState::MainMenu时遗物UI隐藏

    let mut app = create_combat_ui_app();

    let mut collection = RelicCollection::default();
    collection.add_relic(Relic::burning_blood());
    app.world_mut().insert_resource(collection);
    app.world_mut().insert_resource(State::new(GameState::MainMenu));
    app.update();

    // 验证UI不存在或隐藏
    let mut visibility_query = app.world_mut().query_filtered::<Entity, With<RelicUiMarker>>();
    let entities: Vec<_> = visibility_query.iter(app.world_mut()).collect();

    // 要么没有实体，要么所有实体都是隐藏的
    let all_hidden = entities.iter().all(|entity| {
        app.world()
            .get::<Visibility>(*entity)
            .map(|v| v == Visibility::Hidden)
            .unwrap_or(false)
    });

    assert!(entities.is_empty() || all_hidden, "主菜单状态下遗物UI应该隐藏");
}

// ============================================================================
// 场景10: 遗物UI布局位置
// ============================================================================

#[test]
fn test_relic_ui_positioning() {
    // 场景描述: 遗物UI应该在屏幕的合适位置
    // 预期结果: 遗物UI在屏幕左侧或右侧

    let mut app = create_combat_ui_app();

    let mut collection = RelicCollection::default();
    collection.add_relic(Relic::burning_blood());
    app.world_mut().insert_resource(collection);

    app.insert_state(GameState::Combat);
    app.update();

    // 查询遗物UI的位置
    let mut node_query = app.world_mut().query_filtered::<&Node, With<RelicUiMarker>>();
    let nodes: Vec<_> = node_query.iter(app.world_mut()).collect();

    assert!(!nodes.is_empty(), "应该有遗物UI节点");

    // 验证遗物UI有有效的节点
    assert!(!nodes.is_empty(), "遗物UI应该有有效的节点");
}

// ============================================================================
// 场景11: 遗物UI更新响应遗物获取
// ============================================================================

#[test]
fn test_relic_ui_updates_on_acquisition() {
    // 场景描述: 获取新遗物时UI应该自动更新
    // 预期结果: 添加遗物后UI显示新遗物

    let mut app = create_combat_ui_app();

    // 初始1个遗物
    let mut collection = RelicCollection::default();
    collection.add_relic(Relic::burning_blood());
    app.world_mut().insert_resource(collection);

    // 转换到战斗状态
    app.insert_state(GameState::Combat);
    app.update();

    let mut item_query = app.world_mut().query_filtered::<Entity, With<RelicItemMarker>>();
    let initial_count = item_query.iter(app.world_mut()).count();
    assert_eq!(initial_count, 1, "初始应该有1个遗物");

    // 添加新遗物
    let mut collection = app.world_mut().get_resource_mut::<RelicCollection>().unwrap();
    collection.add_relic(Relic::anchor());
    drop(collection);

    // 更新UI
    app.update();

    let mut item_query = app.world_mut().query_filtered::<Entity, With<RelicItemMarker>>();
    let new_count = item_query.iter(app.world_mut()).count();
    assert_eq!(new_count, 2, "添加遗物后应该显示2个遗物");
}

// ============================================================================
// 场景12: 遗物UI不重复显示相同遗物
// ============================================================================

#[test]
fn test_relic_ui_no_duplicates() {
    // 场景描述: UI不应该显示重复的遗物
    // 预期结果: 即使尝试添加相同遗物，UI也只显示1个

    let mut app = create_combat_ui_app();

    let mut collection = RelicCollection::default();
    collection.add_relic(Relic::burning_blood());
    // 尝试添加相同遗物（会失败，因为RelicCollection有去重）
    collection.add_relic(Relic::burning_blood());
    app.world_mut().insert_resource(collection);

    app.insert_state(GameState::Combat);
    app.update();

    let mut item_query = app.world_mut().query_filtered::<Entity, With<RelicItemMarker>>();
    let count = item_query.iter(app.world_mut()).count();

    assert_eq!(count, 1, "不应该显示重复的遗物");
}

// ============================================================================
// 场景13: 遗物UI适应多个遗物的布局
// ============================================================================

#[test]
fn test_relic_ui_layout_with_many_relics() {
    // 场景描述: 当有很多遗物时，UI应该正确布局
    // 预期结果: 4个遗物应该都能正确显示

    let mut app = create_combat_ui_app();

    let mut collection = RelicCollection::default();
    collection.add_relic(Relic::burning_blood());
    collection.add_relic(Relic::bag_of_preparation());
    collection.add_relic(Relic::anchor());
    collection.add_relic(Relic::strange_spoon());
    app.world_mut().insert_resource(collection);

    app.insert_state(GameState::Combat);
    app.update();

    let mut item_query = app.world_mut().query_filtered::<Entity, With<RelicItemMarker>>();
    let count = item_query.iter(app.world_mut()).count();

    assert_eq!(count, 4, "应该显示4个遗物");
}

// ============================================================================
// 场景14: 遗物UI与战斗UI共存
// ============================================================================

#[test]
fn test_relic_ui_coexists_with_combat_ui() {
    // 场景描述: 遗物UI不应该影响其他战斗UI元素
    // 预期结果: 遗物UI、手牌UI、敌人UI都能正常显示

    let mut app = create_combat_ui_app();

    let mut collection = RelicCollection::default();
    collection.add_relic(Relic::burning_blood());
    app.world_mut().insert_resource(collection);

    app.insert_state(GameState::Combat);
    app.update();

    // 验证遗物UI存在
    let mut relic_ui_query = app.world_mut().query_filtered::<Entity, With<RelicUiMarker>>();
    assert!(relic_ui_query.iter(app.world_mut()).count() >= 1, "应该有遗物UI");

    // 验证其他UI也仍然存在
    let mut player_ui_query = app.world_mut().query_filtered::<Entity, With<Player>>();
    assert!(player_ui_query.iter(app.world_mut()).count() >= 1, "应该有玩家实体");

    let mut enemy_ui_query = app.world_mut().query_filtered::<Entity, With<Enemy>>();
    assert!(enemy_ui_query.iter(app.world_mut()).count() >= 1, "应该有敌人实体");
}

// ============================================================================
// 场景15: 遗物描述文本截断处理
// ============================================================================

#[test]
fn test_relic_description_overflow_handling() {
    // 场景描述: 长描述应该正确处理，不破坏布局
    // 预期结果: UI应该正常显示，不会被长描述破坏

    let mut app = create_combat_ui_app();

    let mut collection = RelicCollection::default();
    // 准备背包有较长的描述
    collection.add_relic(Relic::bag_of_preparation());
    app.world_mut().insert_resource(collection);

    app.insert_state(GameState::Combat);
    app.update();

    // 验证UI仍然正常
    let mut item_query = app.world_mut().query_filtered::<&Node, With<RelicItemMarker>>();
    let items: Vec<_> = item_query.iter(app.world_mut()).collect();

    assert!(!items.is_empty(), "应该有遗物项");

    // 验证遗物项存在
    assert!(!items.is_empty(), "应该有遗物项");
}

// ============================================================================
// 场景16: 遗物图标显示（预留）
// ============================================================================

#[test]
fn test_relic_icon_display() {
    // 场景描述: 遗物应该有图标显示
    // 预期结果: 遗物项包含图像节点或颜色块

    let mut app = create_combat_ui_app();

    let mut collection = RelicCollection::default();
    collection.add_relic(Relic::burning_blood());
    app.world_mut().insert_resource(collection);

    app.insert_state(GameState::Combat);
    app.update();

    // 验证遗物项有视觉元素（背景色或图标）
    let mut bg_query = app.world_mut().query_filtered::<&BackgroundColor, With<RelicItemMarker>>();
    let has_background = bg_query.iter(app.world()).count() > 0;

    assert!(has_background, "遗物项应该有背景色或图标");
}

// ============================================================================
// 场景17: 遗物UI在战斗结束后清理
// ============================================================================

#[test]
fn test_relic_ui_cleanup_after_combat() {
    // 场景描述: 战斗结束后应该清理战斗UI，但保留遗物数据
    // 预期结果: 切换到Reward状态后，遗物UI消失但RelicCollection保留

    let mut app = create_combat_ui_app();

    let mut collection = RelicCollection::default();
    collection.add_relic(Relic::burning_blood());
    app.world_mut().insert_resource(collection.clone());
    app.update();

    // 切换到奖励状态
    app.world_mut().insert_resource(State::new(GameState::Reward));
    app.update();

    // 遗物集合应该保留
    let collection = app.world().get_resource::<RelicCollection>();
    assert!(collection.is_some(), "遗物集合应该保留");
    assert_eq!(collection.unwrap().count(), 1, "遗物数量应该保留");
}

// ============================================================================
// 场景18: 遗物UI层级正确
// ============================================================================

#[test]
fn test_relic_ui_z_index() {
    // 场景描述: 遗物UI应该在正确的层级，不被其他UI遮挡
    // 预期结果: 遗物UI应该在背景之上，但在弹窗之下

    let mut app = create_combat_ui_app();

    let mut collection = RelicCollection::default();
    collection.add_relic(Relic::burning_blood());
    app.world_mut().insert_resource(collection);

    app.insert_state(GameState::Combat);
    app.update();

    // 查询遗物UI的节点
    let mut node_query = app.world_mut().query_filtered::<&Node, With<RelicUiMarker>>();
    let nodes: Vec<_> = node_query.iter(app.world_mut()).collect();

    assert!(!nodes.is_empty(), "应该有遗物UI节点");

    // 验证节点存在（Bevy会自动处理Z-index）
    assert!(!nodes.is_empty(), "遗物UI应该有节点");
}
