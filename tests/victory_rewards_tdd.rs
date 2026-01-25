#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy_card_battler::components::{CardPool, CardRarity};

    #[test]
    fn test_heavenly_reward_generation() {
        // 验证天道机缘是否能生成合理稀有度的奖励
        let mut rng = rand::thread_rng();
        
        // 模拟机缘抽取：应包含 3 张功法残页
        let rewards = CardPool::random_rewards(3);
        
        assert_eq!(rewards.len(), 3, "每次天道机缘应赐予3份功法选择");
        
        // 验证是否有概率获得“特殊/稀有”功法（虽然是随机的，但我们可以检查结构）
        for card in rewards {
            assert!(!card.name.is_empty(), "机缘名不能为空");
        }
    }
}
