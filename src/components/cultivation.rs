//! 修仙境界系统组件

use bevy::prelude::*;

/// 修仙境界枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Realm {
    /// 炼气期 (Qi Refining)
    #[default]
    QiRefining,
    /// 筑基期 (Foundation Establishment)
    FoundationEstablishment,
    /// 金丹期 (Golden Core)
    GoldenCore,
    /// 元婴期 (Nascent Soul)
    NascentSoul,
}

/// 玩家修炼进度组件
#[derive(Component, Debug, Clone)]
pub struct Cultivation {
    /// 当前境界
    pub realm: Realm,
    /// 当前感悟值 (Insight)
    pub insight: u32,
}

impl Cultivation {
    /// 创建初始修炼状态
    pub fn new() -> Self {
        Self {
            realm: Realm::QiRefining,
            insight: 0,
        }
    }

    /// 获得感悟
    pub fn gain_insight(&mut self, amount: u32) {
        self.insight += amount;
    }

    /// 获取当前境界突破所需的感悟阈值
    pub fn get_threshold(&self) -> u32 {
        match self.realm {
            Realm::QiRefining => 40, // 调试用：降低门槛（原为100）
            Realm::FoundationEstablishment => 250,
            Realm::GoldenCore => 600,
            Realm::NascentSoul => 1500,
        }
    }

    /// 检查是否满足突破条件
    pub fn can_breakthrough(&self) -> bool {
        self.insight >= self.get_threshold()
    }

    /// 执行突破
    /// 如果成功提升境界返回 true，否则返回 false
    pub fn breakthrough(&mut self) -> bool {
        if !self.can_breakthrough() {
            return false;
        }

        // 提升境界
        let next_realm = match self.realm {
            Realm::QiRefining => Some(Realm::FoundationEstablishment),
            Realm::FoundationEstablishment => Some(Realm::GoldenCore),
            Realm::GoldenCore => Some(Realm::NascentSoul),
            Realm::NascentSoul => None, // 已达巅峰
        };

        if let Some(new_realm) = next_realm {
            // 扣除感悟值
            self.insight -= self.get_threshold();
            self.realm = new_realm;
            true
        } else {
            false
        }
    }

    /// 获取境界带来的最大道行（HP）加成
    pub fn get_hp_bonus(&self) -> i32 {
        match self.realm {
            Realm::QiRefining => 0,
            Realm::FoundationEstablishment => 20,
            Realm::GoldenCore => 50,
            Realm::NascentSoul => 120,
        }
    }
}
