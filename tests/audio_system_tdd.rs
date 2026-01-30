use bevy::prelude::*;
use bevy_card_battler::components::audio::{SfxType};
use std::path::Path;

#[test]
fn test_sfx_file_paths_existence() {
    // 检查所有在 SfxType 中定义的文件是否在 assets 目录中真实存在
    let types = [
        SfxType::CardPlay,
        SfxType::DrawCard,
        SfxType::ShuffleCard,
        SfxType::CardHover,
        SfxType::CardSelect,
        SfxType::PlayerAttack,
        SfxType::PlayerHit,
        SfxType::EnemyHit,
        SfxType::Block,
        SfxType::CriticalHit,
        SfxType::Dodge,
        SfxType::LightningStrike,
        SfxType::FireSpell,
        SfxType::IceSpell,
        SfxType::Heal,
        SfxType::BuffApply,
        SfxType::DebuffApply,
        SfxType::ShieldUp,
        SfxType::UltimateStart,
        SfxType::UltimateRelease,
        SfxType::SwordStrike,
        SfxType::ThousandSwords,
        SfxType::UiClick,
        SfxType::UiHover,
        SfxType::UiConfirm,
        SfxType::UiCancel,
        SfxType::UiError,
        SfxType::BreakthroughStart,
        SfxType::BreakthroughSuccess,
        SfxType::LevelUp,
        SfxType::GoldGain,
        SfxType::RelicObtain,
        SfxType::Victory,
        SfxType::Defeat,
        SfxType::EnemySpawn,
        SfxType::EnemyDeath,
        SfxType::BossAppear,
        SfxType::BossDeath,
    ];
    
    let mut missing_files = Vec::new();
    
    for sfx in types {
        let rel_path = sfx.file_path();
        let full_path = format!("assets/{}", rel_path);
        if !Path::new(&full_path).exists() {
            missing_files.push(full_path);
        }
    }
    
    if !missing_files.is_empty() {
        panic!("以下音效文件缺失: \n{}", missing_files.join("\n"));
    }
}
