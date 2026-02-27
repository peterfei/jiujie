use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;
use crate::components::*;
use crate::components::cards::Card;
use crate::components::relic::Relic;
use crate::components::map::MapNode;
use crate::components::cultivation::Cultivation;

/// 完整的游戏存档数据
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameStateSave {
    pub player: Player,
    pub cultivation: Cultivation,
    pub deck: Vec<Card>,
    pub relics: Vec<Relic>,
    pub map_nodes: Vec<MapNode>,
    pub current_map_node_id: Option<u32>,
    pub current_map_layer: u32,
}

impl GameStateSave {
    /// 存档文件路径
    pub fn get_save_path() -> String {
        "savegame.json".to_string()
    }

    /// 检查是否存在存档
    pub fn exists() -> bool {
        Path::new(&Self::get_save_path()).exists()
    }

    /// 执行存档
    pub fn save_to_disk(&self) -> Result<(), String> {
        let serialized = serde_json::to_string_pretty(self)
            .map_err(|e| format!("序列化失败: {}", e))?;
        fs::write(Self::get_save_path(), serialized)
            .map_err(|e| format!("写入文件失败: {}", e))?;
        info!("【存档系统】修仙进度已保存至磁盘");
        Ok(())
    }

    /// 执行读档
    pub fn load_from_disk() -> Result<Self, String> {
        let data = fs::read_to_string(Self::get_save_path())
            .map_err(|e| format!("读取文件失败: {}", e))?;
        let deserialized: Self = serde_json::from_str(&data)
            .map_err(|e| format!("反序列化失败: {}", e))?;
        info!("【存档系统】修仙进度已从磁盘恢复");
        Ok(deserialized)
    }

    /// 删除存档（道消身殒时调用）
    pub fn delete_save() {
        if Self::exists() {
            let _ = fs::remove_file(Self::get_save_path());
            info!("【存档系统】存档已删除");
        }
    }
}

/// 存档插件
pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, _app: &mut App) {
        // 后续可添加自动存档系统
    }
}
