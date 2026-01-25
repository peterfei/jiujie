#[cfg(test)]
mod tests {
    use bevy_card_battler::components::relic::{Relic, RelicId};

    #[test]
    fn test_relics_are_cultivation_themed() {
        // 1. 测试 "飞剑符" (原燃烧之血)
        let sword_talisman = Relic::burning_blood();
        assert_eq!(sword_talisman.name, "飞剑符", "BurningBlood 应重命名为 '飞剑符'");
        assert!(sword_talisman.description.contains("伤害"), "飞剑符描述应包含伤害");

        // 2. 测试 "乾坤袋" (原准备背包)
        let universe_bag = Relic::bag_of_preparation();
        assert_eq!(universe_bag.name, "乾坤袋", "BagOfPreparation 应重命名为 '乾坤袋'");
        assert!(universe_bag.description.contains("获得 1 张"), "乾坤袋描述应准确");

        // 3. 测试 "定风珠" (原锚)
        let wind_pearl = Relic::anchor();
        assert_eq!(wind_pearl.name, "定风珠", "Anchor 应重命名为 '定风珠'");
        assert!(wind_pearl.description.contains("保留"), "定风珠描述应包含保留手牌");

        // 4. 测试 "聚灵阵" (原奇怪勺子)
        let spirit_array = Relic::strange_spoon();
        assert_eq!(spirit_array.name, "聚灵阵", "StrangeSpoon 应重命名为 '聚灵阵'");
        assert!(spirit_array.description.contains("抽"), "聚灵阵描述应包含抽牌");
    }
}
