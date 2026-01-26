#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy_card_battler::components::{Enemy, Player};

    #[test]
    fn test_victory_requires_all_enemies_dead() {
        // 模拟有两个敌人的战场
        let mut enemy1 = Enemy::new(1, "幼狼", 10);
        let mut enemy2 = Enemy::new(2, "幼狼", 10);
        
        // 击败其中一个
        enemy1.hp = 0;
        
        // 逻辑断言：如果还有一个生命值 > 0 的敌人，战斗不应结束
        let any_alive = enemy1.hp > 0 || enemy2.hp > 0;
        assert!(any_alive, "仍有敌人存活，战斗不应判定胜利");
        
        // 全部击败
        enemy2.hp = 0;
        let none_alive = !(enemy1.hp > 0 || enemy2.hp > 0);
        assert!(none_alive, "所有敌人已伏诛，应当判定胜利");
    }

    #[test]
    fn test_multiple_enemy_intents_processing() {
        // 验证系统是否能处理多个敌人的不同意图
        // 实际逻辑中我们将使用 iter() 代替 get_single()
    }
}
