#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy_card_battler::components::dialogue::{Dialogue, DialogueLine};

    #[test]
    fn test_dialogue_progression() {
        // 1. 初始化包含 3 行台词的对话
        let mut dialogue = Dialogue::new(vec![
            DialogueLine::new("神秘声音", "九界崩塌，寰宇将灭..."),
            DialogueLine::new("天道", "唯有渡劫，方可一线生机。"),
            DialogueLine::new("凡人", "我，定要逆天而行！"),
        ]);

        // 验证初始行
        assert_eq!(dialogue.current_line().unwrap().text, "九界崩塌，寰宇将灭...");
        assert_eq!(dialogue.index, 0);

        // 2. 推进到下一行
        dialogue.next();
        assert_eq!(dialogue.current_line().unwrap().speaker, "天道");
        assert_eq!(dialogue.current_line().unwrap().text, "唯有渡劫，方可一线生机。");
        assert_eq!(dialogue.index, 1);

        // 3. 推进到最后一行
        dialogue.next();
        assert_eq!(dialogue.index, 2);
        assert!(!dialogue.is_finished());

        // 4. 再次推进，标记为结束
        dialogue.next();
        assert!(dialogue.is_finished());
    }
}
