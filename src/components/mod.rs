//! ECS组件定义

pub mod map;
pub mod combat;
pub mod cards;
pub mod animation;
pub mod sprite;
pub mod particle;
pub mod screen_effect;
pub mod relic;
pub mod shop;
pub mod cultivation;
pub mod dialogue;
pub mod audio;

pub use map::{MapNode, NodeType, MapConfig, generate_map_nodes, MapProgress};
pub use combat::{
    Player, Enemy, EnemyIntent, EnemyType, AiPattern, CombatConfig, CombatState, TurnPhase, 
    PlayerDeck, VictoryDelay, CardHoverPanelMarker, RelicHoverPanelMarker, EnemyActionQueue,
    DamageNumber, DamageEffectEvent, BlockIconMarker, BlockText, StatusIndicator,
    EnemyHpText, EnemyIntentText, EnemyStatusUi, PlayerHpText, PlayerEnergyText, PlayerBlockText,
    TopBar, TopBarHpText, TopBarGoldText, EnergyOrb, EndTurnButton, HandArea, CombatUiRoot,
    StatusEffectEvent, CardDescriptionMarker, PlayerHpBarMarker, EnemyHpBarMarker, IntentIconMarker,
    PlayerHpBufferMarker, EnemyHpBufferMarker, Environment,
};
pub use cards::{
    Card, CardType, CardEffect, CardRarity,
    DrawPile, DiscardPile, Hand,
    DeckConfig, CardPool, RewardCard,
};
pub use cultivation::{Cultivation, Realm};
pub use dialogue::{Dialogue, DialogueLine};
pub use audio::{PlaySfxEvent, SfxType};
pub use animation::{
    MovementAnimation, ShakeAnimation, FloatingDamageText,
    EnemyUiMarker, PlayerUiMarker, EasingFunction, EnemyAttackEvent
};
pub use sprite::{
    CharacterSprite, AnimationState, CharacterType, CharacterAssets,
    CharacterAnimationEvent, SpriteMarker, PlayerSpriteMarker, EnemySpriteMarker
};
pub use particle::{
    Particle, ParticleEmitter, EmitterConfig, EffectType,
    SpawnEffectEvent, ParticleMarker, EmitterMarker,
    EnemyDeathAnimation, VictoryEvent
};
pub use screen_effect::{
    CameraShake, ScreenFlash, ScreenEffectEvent, ScreenEffectMarker, ScreenWarning
};
pub use relic::{
    Relic, RelicId, RelicRarity, RelicEffect,
    RelicCollection, RelicObtainedEvent, RelicTriggeredEvent
};
pub use shop::{
    ShopItem, ShopUiRoot, ShopCardButton, ShopRelicButton,
    ShopRemoveCardButton, ShopExitButton, ShopGoldText, CurrentShopItems,
    SelectedCardForRemoval
};
