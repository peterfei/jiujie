//! 音效系统组件与事件
//!
//! # 文件配置
//! 所有音效文件应放置在 `assets/audio/sfx/` 目录下：
//! ```text
//! assets/audio/sfx/
//! ├── 卡牌相关/
//! ├── 战斗相关/
//! ├── 法术技能/
//! ├── 大招技能/
//! ├── UI交互/
//! ├── 系统事件/
//! └── 敌人相关/
//! ```
//!
//! # 音效资源获取指南
//! 详细的音效对照表和资源网站参见: `assets/audio/sfx/SOUND_EFFECTS_GUIDE.md`

use bevy::prelude::*;
use serde::{Serialize, Deserialize};

// ============================================================================
// 音效类型
// ============================================================================

/// 音效类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SfxType {
    // ==================== 卡牌相关 ====================
    /// 出牌音效
    CardPlay,
    /// 抽牌音效
    DrawCard,
    /// 洗牌音效
    ShuffleCard,
    /// 卡牌悬停
    CardHover,
    /// 卡牌选中
    CardSelect,

    // ==================== 战斗相关 ====================
    /// 玩家攻击
    PlayerAttack,
    /// 玩家受击
    PlayerHit,
    /// 敌人受击
    EnemyHit,
    /// 格挡音效
    Block,
    /// 暴击音效
    CriticalHit,
    /// 闪避音效
    Dodge,

    // ==================== 法术/技能 ====================
    /// 天雷落下
    LightningStrike,
    /// 火焰法术
    FireSpell,
    /// 冰霜法术
    IceSpell,
    /// 治疗音效
    Heal,
    /// 增益施加
    BuffApply,
    /// 减益施加
    DebuffApply,
    /// 护盾升起
    ShieldUp,

    // ==================== 大招/终极技能 ====================
    /// 大招起手
    UltimateStart,
    /// 大招释放
    UltimateRelease,
    /// 剑气斩击
    SwordStrike,
    /// 万剑归宗
    ThousandSwords,

    // ==================== UI交互 ====================
    /// UI点击
    UiClick,
    /// UI悬停
    UiHover,
    /// UI确认
    UiConfirm,
    /// UI取消
    UiCancel,
    /// UI错误
    UiError,

    // ==================== 系统/事件 ====================
    /// 突破开始
    BreakthroughStart,
    /// 境界突破成功
    BreakthroughSuccess,
    /// 升级音效
    LevelUp,
    /// 获得金币
    GoldGain,
    /// 获得遗物
    RelicObtain,
    /// 战斗胜利
    Victory,
    /// 战斗失败
    Defeat,

    // ==================== 敌人相关 ====================
    /// 敌人生成
    EnemySpawn,
    /// 敌人死亡
    EnemyDeath,
    /// Boss登场
    BossAppear,
    /// Boss死亡
    BossDeath,
}

impl SfxType {
    /// 获取音效文件路径
    pub fn file_path(&self) -> &'static str {
        match self {
            // 卡牌相关
            SfxType::CardPlay => "audio/sfx/card_play.ogg",
            SfxType::DrawCard => "audio/sfx/draw_card.ogg",
            SfxType::ShuffleCard => "audio/sfx/shuffle_card.ogg",
            SfxType::CardHover => "audio/sfx/card_hover.ogg",
            SfxType::CardSelect => "audio/sfx/ui_confirm.ogg", 

            // 战斗相关
            SfxType::PlayerAttack => "audio/sfx/player_attack.ogg",
            SfxType::PlayerHit => "audio/sfx/player_hit.ogg", 
            SfxType::EnemyHit => "audio/sfx/enemy_hit.ogg",
            SfxType::Block => "audio/sfx/block.ogg",
            SfxType::CriticalHit => "audio/sfx/critical_hit.ogg",
            SfxType::Dodge => "audio/sfx/ui_hover.ogg", 

            // 法术/技能
            SfxType::LightningStrike => "audio/sfx/lightning_strike.ogg",
            SfxType::FireSpell => "audio/sfx/fire_spell.ogg",
            SfxType::IceSpell => "audio/sfx/ice_spell.ogg",
            SfxType::Heal => "audio/sfx/heal.ogg",
            SfxType::BuffApply => "audio/sfx/buff_apply.ogg",
            SfxType::DebuffApply => "audio/sfx/debuff_apply.ogg",
            SfxType::ShieldUp => "audio/sfx/shield_up.ogg",

            // 大招/终极技能
            SfxType::UltimateStart => "audio/sfx/ultimate_start.ogg",
            SfxType::UltimateRelease => "audio/sfx/ultimate_release.ogg",
            SfxType::SwordStrike => "audio/sfx/sword_strike.ogg",
            SfxType::ThousandSwords => "audio/sfx/thousand_swords.ogg",

            // UI交互
            SfxType::UiClick => "audio/sfx/ui_click.ogg",
            SfxType::UiHover => "audio/sfx/ui_hover.ogg",
            SfxType::UiConfirm => "audio/sfx/ui_confirm.ogg",
            SfxType::UiCancel => "audio/sfx/ui_cancel.ogg",
            SfxType::UiError => "audio/sfx/ui_error.ogg",

            // 系统/事件
            SfxType::BreakthroughStart => "audio/sfx/breakthrough_start.ogg",
            SfxType::BreakthroughSuccess => "audio/sfx/breakthrough_success.ogg",
            SfxType::LevelUp => "audio/sfx/level_up.ogg",
            SfxType::GoldGain => "audio/sfx/gold_gain.ogg",
            SfxType::RelicObtain => "audio/sfx/relic_obtain.ogg",
            SfxType::Victory => "audio/sfx/victory.ogg",
            SfxType::Defeat => "audio/sfx/defeat.ogg",

            // 敌人相关
            SfxType::EnemySpawn => "audio/sfx/enemy_spawn.ogg",
            SfxType::EnemyDeath => "audio/sfx/enemy_death.ogg",
            SfxType::BossAppear => "audio/sfx/boss_appear.ogg",
            SfxType::BossDeath => "audio/sfx/boss_death.ogg",
        }
    }

    /// 获取音效文件名（不含扩展名）
    pub fn file_name(&self) -> &'static str {
        match self {
            SfxType::CardPlay => "card_play",
            SfxType::DrawCard => "draw_card",
            SfxType::ShuffleCard => "shuffle_card",
            SfxType::CardHover => "card_hover",
            SfxType::CardSelect => "card_select",
            SfxType::PlayerAttack => "player_attack",
            SfxType::PlayerHit => "player_hit",
            SfxType::EnemyHit => "enemy_hit",
            SfxType::Block => "block",
            SfxType::CriticalHit => "critical_hit",
            SfxType::Dodge => "dodge",
            SfxType::LightningStrike => "lightning_strike",
            SfxType::FireSpell => "fire_spell",
            SfxType::IceSpell => "ice_spell",
            SfxType::Heal => "heal",
            SfxType::BuffApply => "buff_apply",
            SfxType::DebuffApply => "debuff_apply",
            SfxType::ShieldUp => "shield_up",
            SfxType::UltimateStart => "ultimate_start",
            SfxType::UltimateRelease => "ultimate_release",
            SfxType::SwordStrike => "sword_strike",
            SfxType::ThousandSwords => "thousand_swords",
            SfxType::UiClick => "ui_click",
            SfxType::UiHover => "ui_hover",
            SfxType::UiConfirm => "ui_confirm",
            SfxType::UiCancel => "ui_cancel",
            SfxType::UiError => "ui_error",
            SfxType::BreakthroughStart => "breakthrough_start",
            SfxType::BreakthroughSuccess => "breakthrough_success",
            SfxType::LevelUp => "level_up",
            SfxType::GoldGain => "gold_gain",
            SfxType::RelicObtain => "relic_obtain",
            SfxType::Victory => "victory",
            SfxType::Defeat => "defeat",
            SfxType::EnemySpawn => "enemy_spawn",
            SfxType::EnemyDeath => "enemy_death",
            SfxType::BossAppear => "boss_appear",
            SfxType::BossDeath => "boss_death",
        }
    }

    /// 获取音效的中文名称
    pub fn chinese_name(&self) -> &'static str {
        match self {
            SfxType::CardPlay => "出牌",
            SfxType::DrawCard => "抽牌",
            SfxType::ShuffleCard => "洗牌",
            SfxType::CardHover => "卡牌悬停",
            SfxType::CardSelect => "卡牌选中",
            SfxType::PlayerAttack => "玩家攻击",
            SfxType::PlayerHit => "玩家受击",
            SfxType::EnemyHit => "敌人受击",
            SfxType::Block => "格挡",
            SfxType::CriticalHit => "暴击",
            SfxType::Dodge => "闪避",
            SfxType::LightningStrike => "天雷落下",
            SfxType::FireSpell => "火焰法术",
            SfxType::IceSpell => "冰霜法术",
            SfxType::Heal => "治疗",
            SfxType::BuffApply => "增益施加",
            SfxType::DebuffApply => "减益施加",
            SfxType::ShieldUp => "护盾升起",
            SfxType::UltimateStart => "大招起手",
            SfxType::UltimateRelease => "大招释放",
            SfxType::SwordStrike => "剑气斩击",
            SfxType::ThousandSwords => "万剑归宗",
            SfxType::UiClick => "UI点击",
            SfxType::UiHover => "UI悬停",
            SfxType::UiConfirm => "UI确认",
            SfxType::UiCancel => "UI取消",
            SfxType::UiError => "UI错误",
            SfxType::BreakthroughStart => "突破开始",
            SfxType::BreakthroughSuccess => "突破成功",
            SfxType::LevelUp => "升级",
            SfxType::GoldGain => "获得金币",
            SfxType::RelicObtain => "获得遗物",
            SfxType::Victory => "战斗胜利",
            SfxType::Defeat => "战斗失败",
            SfxType::EnemySpawn => "敌人生成",
            SfxType::EnemyDeath => "敌人死亡",
            SfxType::BossAppear => "Boss登场",
            SfxType::BossDeath => "Boss死亡",
        }
    }

    /// 获取音效分类
    pub fn category(&self) -> &'static str {
        match self {
            SfxType::CardPlay | SfxType::DrawCard | SfxType::ShuffleCard |
            SfxType::CardHover | SfxType::CardSelect => "卡牌相关",

            SfxType::PlayerAttack | SfxType::PlayerHit | SfxType::EnemyHit |
            SfxType::Block | SfxType::CriticalHit | SfxType::Dodge => "战斗相关",

            SfxType::LightningStrike | SfxType::FireSpell | SfxType::IceSpell |
            SfxType::Heal | SfxType::BuffApply | SfxType::DebuffApply |
            SfxType::ShieldUp => "法术技能",

            SfxType::UltimateStart | SfxType::UltimateRelease |
            SfxType::SwordStrike | SfxType::ThousandSwords => "大招技能",

            SfxType::UiClick | SfxType::UiHover | SfxType::UiConfirm |
            SfxType::UiCancel | SfxType::UiError => "UI交互",

            SfxType::BreakthroughStart | SfxType::BreakthroughSuccess |
            SfxType::LevelUp | SfxType::GoldGain | SfxType::RelicObtain |
            SfxType::Victory | SfxType::Defeat => "系统事件",

            SfxType::EnemySpawn | SfxType::EnemyDeath |
            SfxType::BossAppear | SfxType::BossDeath => "敌人相关",
        }
    }

    /// 获取音效默认音量
    pub fn default_volume(&self) -> f32 {
        match self {
            // UI音效通常较轻
            SfxType::UiHover | SfxType::UiClick | SfxType::CardHover => 0.3,

            // 大招音效较响
            SfxType::UltimateRelease | SfxType::ThousandSwords |
            SfxType::BossAppear | SfxType::BossDeath => 0.9,

            // 暴击和重要事件
            SfxType::CriticalHit | SfxType::Victory | SfxType::BreakthroughSuccess => 0.8,

            // 默认音量
            _ => 0.6,
        }
    }

    /// 获取推荐时长（秒）
    pub fn recommended_duration(&self) -> (f32, f32) {
        match self {
            SfxType::UiHover => (0.1, 0.15),
            SfxType::UiClick => (0.1, 0.2),
            SfxType::CardHover => (0.2, 0.3),
            SfxType::CardPlay => (0.5, 1.0),
            SfxType::DrawCard => (0.3, 0.5),
            SfxType::ThousandSwords => (3.0, 5.0),
            SfxType::BossDeath => (3.0, 5.0),
            _ => (0.5, 2.0),
        }
    }
}

// ============================================================================
// 音效事件
// ============================================================================

/// 播放音效事件
#[derive(Event, Debug, Clone)]
pub struct PlaySfxEvent {
    pub sfx_type: SfxType,
    /// 音量 (0.0 - 1.0)，None则使用默认
    pub volume: Option<f32>,
}

impl PlaySfxEvent {
    /// 创建默认播放事件（使用默认音量）
    pub fn new(sfx_type: SfxType) -> Self {
        Self {
            sfx_type,
            volume: None,
        }
    }

    /// 创建指定音量的播放事件
    pub fn with_volume(sfx_type: SfxType, volume: f32) -> Self {
        Self {
            sfx_type,
            volume: Some(volume),
        }
    }

    /// 获取实际音量
    pub fn get_volume(&self) -> f32 {
        self.volume.unwrap_or_else(|| self.sfx_type.default_volume())
    }
}

// ============================================================================
// 音效设置
// ============================================================================

/// 音效全局设置
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct SfxSettings {
    /// 主音量开关
    pub enabled: bool,
    /// 主音量 (0.0 - 1.0)
    pub master_volume: f32,
    /// UI音效开关
    pub ui_enabled: bool,
    /// 战斗音效开关
    pub combat_enabled: bool,
}

impl Default for SfxSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            master_volume: 0.7,
            ui_enabled: true,
            combat_enabled: true,
        }
    }
}
