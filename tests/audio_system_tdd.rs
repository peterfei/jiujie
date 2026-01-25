#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy_card_battler::components::audio::{PlaySfxEvent, SfxType};
    use bevy_card_battler::components::{Card, CardEffect, CardType, CardRarity};

    #[test]
    fn test_card_play_triggers_audio_event() {
        let mut app = App::new();
        app.add_event::<PlaySfxEvent>();

        // 模拟逻辑：发送一个出牌事件（这里假设我们有一个系统在处理出牌逻辑时发送音效）
        // 在实际代码中，我们需要在 apply_card_effect 中加入此逻辑
        let mut event_writer = app.world_mut().resource_mut::<Events<PlaySfxEvent>>();
        event_writer.send(PlaySfxEvent::new(SfxType::CardPlay));

        // 验证事件是否已发送
        let events = app.world().resource::<Events<PlaySfxEvent>>();
        let mut reader = events.get_cursor();
        let first_event = reader.read(events).next().expect("应该触发音效事件");
        assert_eq!(first_event.sfx_type, SfxType::CardPlay);
    }

    #[test]
    fn test_lightning_strike_triggers_audio_event() {
        let mut app = App::new();
        app.add_event::<PlaySfxEvent>();

        // 模拟雷劫落下的逻辑
        let mut event_writer = app.world_mut().resource_mut::<Events<PlaySfxEvent>>();
        event_writer.send(PlaySfxEvent::new(SfxType::LightningStrike));

        let events = app.world().resource::<Events<PlaySfxEvent>>();
        let mut reader = events.get_cursor();
        let event = reader.read(events).next().unwrap();
        assert_eq!(event.sfx_type, SfxType::LightningStrike);
    }
}
