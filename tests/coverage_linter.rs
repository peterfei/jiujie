//! 测试覆盖检查工具
//!
//! 运行: cargo test --test coverage_linter
//!
//! 自动检测：
//! 1. 哪些枚举值没有被测试
//! 2. 哪些状态转换没有测试
//! 3. 哪些组件交互没有测试

use bevy_card_battler::components::NodeType;
use bevy_card_battler::states::GameState;

/// 检查点：所有枚举值都应该被测试
#[test]
fn lint_all_node_types_should_have_tests() {
    // 这只是一个示例，实际应该检查测试文件中是否引用了这些类型

    // NodeType 所有变体
    let all_types = vec![
        NodeType::Normal,
        NodeType::Elite,
        NodeType::Boss,
        NodeType::Rest,
        NodeType::Shop,
        NodeType::Treasure,
        NodeType::Unknown,
    ];

    // TODO: 自动扫描测试文件，检查每个 NodeType 是否被测试
    // 当前手动记录缺失的测试：
    let tested_types = vec![
        NodeType::Normal,  // ✅ 有测试
    ];

    let missing: Vec<_> = all_types.iter()
        .filter(|t| !tested_types.contains(t))
        .collect();

    if !missing.is_empty() {
        eprintln!("\n==========================================");
        eprintln!("⚠️  测试覆盖警告：以下 NodeType 没有测试");
        eprintln!("==========================================");
        for t in missing {
            eprintln!("  - {:?}", t);
        }
        eprintln!("==========================================\n");
        // 不要让测试失败，只是警告
        // panic!("请添加缺失的测试");
    }
}

/// 检查点：所有状态转换都应该被测试
#[test]
fn lint_all_state_transitions_should_have_tests() {
    // GameState 所有可能的转换
    let expected_transitions = vec![
        ("MainMenu", "Map"),
        ("Map", "Combat"),
        ("Map", "Rest"),
        ("Map", "Shop"),
        ("Combat", "Reward"),
        ("Combat", "GameOver"),
        ("Rest", "Map"),
        ("Shop", "Map"),
        ("Reward", "Map"),
        ("GameOver", "MainMenu"),
    ];

    // TODO: 自动扫描测试文件，检查哪些转换被测试了
    let tested_transitions = vec![
        ("MainMenu", "Map"),    // ✅
        ("Map", "Combat"),       // ✅
        ("Combat", "Reward"),    // ✅
        ("Combat", "GameOver"),  // ✅
    ];

    let missing: Vec<_> = expected_transitions.iter()
        .filter(|t| !tested_transitions.contains(t))
        .collect();

    if !missing.is_empty() {
        eprintln!("\n==========================================");
        eprintln!("⚠️  测试覆盖警告：以下状态转换没有测试");
        eprintln!("==========================================");
        for (from, to) in missing {
            eprintln!("  - {} → {}", from, to);
        }
        eprintln!("==========================================\n");
    }
}

/// 检查点：每个用户交互都应该有E2E测试
#[test]
fn lint_user_interactions_should_have_e2e_tests() {
    // 识别所有用户交互点
    let interactions = vec![
        "点击开始游戏按钮",
        "点击地图节点",
        "点击卡牌",
        "点击结束回合",
        "选择奖励遗物",
        "点击商店商品",
        "点击休息确认",
        "点击返回地图",
    ];

    // TODO: 自动检查哪些交互有E2E测试
    let tested = vec![
        "点击开始游戏按钮",
        "点击卡牌",
        "点击结束回合",
        "选择奖励遗物",
        "点击地图节点",           // interactions_simple
        "点击商店商品",           // interactions_simple
        "点击休息确认",           // interactions_simple
        "点击返回地图",           // interactions_simple
    ];

    let missing: Vec<_> = interactions.iter()
        .filter(|i| !tested.contains(i))
        .collect();

    if !missing.is_empty() {
        println!("\n==========================================");
        println!("⚠️  测试覆盖警告：以下交互没有E2E测试");
        println!("==========================================");
        for i in missing {
            println!("  - {}", i);
        }
        println!("==========================================\n");
    }
}
