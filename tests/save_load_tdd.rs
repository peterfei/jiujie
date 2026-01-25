#[cfg(test)]
mod tests {
    use bevy_card_battler::components::{Player, Cultivation, Realm};
    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct SaveData {
        player_hp: i32,
        player_gold: i32,
        realm: Realm,
    }

    #[test]
    fn test_save_load_serialization() {
        let save = SaveData {
            player_hp: 80,
            player_gold: 150,
            realm: Realm::FoundationEstablishment,
        };

        // 序列化
        let serialized = serde_json::to_string(&save).unwrap();
        
        // 反序列化
        let deserialized: SaveData = serde_json::from_str(&serialized).unwrap();

        assert_eq!(save, deserialized, "反序列化后的数据应与原始数据一致");
    }
}
