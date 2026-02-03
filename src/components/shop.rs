//! 商店组件和系统

use bevy::prelude::*;
use crate::components::{Card, Relic};

// ============================================================================
// 商店组件
// ============================================================================

/// 商店商品
#[derive(Debug, Clone)]
pub enum ShopItem {
    /// 功法（卡牌）
    Card(Card),
    /// 法宝（遗物）
    Relic(Relic),
    /// 灵丹（恢复类）
    Elixir {
        name: String,
        hp_restore: i32,
        price: i32,
        description: String,
    },
    /// 遗忘功法（移除卡牌服务）
    ForgetTechnique,
    /// 售罄/已获得
    SoldOut {
        original_name: String,
    },
}

#[derive(Event)]
pub struct ShopItemPurchased {
    pub item_index: usize,
}

impl ShopItem {
    /// 获取商品价格
    pub fn get_price(&self) -> i32 {
        match self {
            ShopItem::Card(card) => {
                // 特殊处理：大招售价 100
                if card.name.contains("天象·引雷术") {
                    return 100;
                }
                match card.rarity {
                    crate::components::CardRarity::Common => 30,
                    crate::components::CardRarity::Uncommon => 50,
                    crate::components::CardRarity::Rare => 80,
                    crate::components::CardRarity::Special => 100,
                }
            }
            ShopItem::Relic(relic) => {
                match relic.rarity {
                    crate::components::relic::RelicRarity::Common => 50,
                    crate::components::relic::RelicRarity::Uncommon => 75,
                    crate::components::relic::RelicRarity::Rare => 100,
                    crate::components::relic::RelicRarity::Special => 150,
                }
            }
            ShopItem::Elixir { price, .. } => *price,
            ShopItem::ForgetTechnique => 50,
            ShopItem::SoldOut { .. } => 0,
        }
    }

    /// 获取商品名称
    pub fn get_name(&self) -> String {
        match self {
            ShopItem::Card(card) => card.name.clone(),
            ShopItem::Relic(relic) => relic.name.clone(),
            ShopItem::Elixir { name, .. } => name.clone(),
            ShopItem::ForgetTechnique => "遗忘功法".to_string(),
            ShopItem::SoldOut { .. } => "已售罄".to_string(),
        }
    }

    /// 获取商品描述
    pub fn get_description(&self) -> String {
        match self {
            ShopItem::Card(card) => card.description.clone(),
            ShopItem::Relic(relic) => relic.description.clone(),
            ShopItem::Elixir { description, .. } => description.clone(),
            ShopItem::ForgetTechnique => "从识海中永久抹去一门功法，以免贪多嚼不烂".to_string(),
            ShopItem::SoldOut { original_name } => format!("{} 已被取走，因果已了。", original_name),
        }
    }
}

/// 商店UI标记
#[derive(Component)]
pub struct ShopUiRoot;

/// 商店卡牌按钮标记
#[derive(Component)]
pub struct ShopCardButton {
    pub item_index: usize,
}

/// 商店遗物按钮标记
#[derive(Component)]
pub struct ShopRelicButton {
    pub item_index: usize,
}

/// 移除卡牌按钮标记
#[derive(Component)]
pub struct ShopRemoveCardButton;

/// 返回地图按钮标记
#[derive(Component)]
pub struct ShopExitButton;

/// 商店金币文本标记
#[derive(Component)]
pub struct ShopGoldText;

/// 当前商店商品资源
#[derive(Resource, Default)]
pub struct CurrentShopItems {
    pub items: Vec<ShopItem>,
}

/// 选中的待移除卡牌（用于移除卡牌服务）
#[derive(Resource, Default)]
pub struct SelectedCardForRemoval {
    pub card_id: Option<u32>,
}
