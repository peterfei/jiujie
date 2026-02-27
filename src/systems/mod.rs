//! 游戏系统实现

pub mod animation;
pub mod sprite;
pub mod vfx_orchestrator;
pub mod gpu_particle;
pub mod screen_effect;
pub mod relic;
pub mod shop;
pub mod rest;
pub mod audio;
pub mod background_music;
pub mod ui;
pub mod map;
pub mod event;
pub mod enemy_gen;

pub use animation::AnimationPlugin;
pub use sprite::SpritePlugin;
pub use vfx_orchestrator::VfxOrchestratorPlugin;
pub use gpu_particle::GpuParticlePlugin;
pub use screen_effect::ScreenEffectPlugin;
pub use relic::{
    RelicPlugin, RelicUiPlugin, CombatStartProcessed,
    trigger_relics_on_combat_start, trigger_relics_on_phase_change, trigger_relics_on_card_played
};
pub use shop::{ShopPlugin, update_gold_display};
pub use rest::RestPlugin;
pub use event::EventPlugin;
pub use audio::SfxPlugin;
pub use background_music::BackgroundMusicPlugin;
pub use ui::UiPlugin;
pub use map::MapPlugin;