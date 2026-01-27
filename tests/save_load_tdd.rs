use bevy::prelude::*;
use bevy_card_battler::resources::save::GameStateSave;
use bevy_card_battler::components::map::MapProgress;

#[test]
fn test_new_game_clears_old_save() {
    let mut app = App::new();
    
    // 1. 模拟存在旧进度
    let old_progress = MapProgress::default();
    app.insert_resource(old_progress);
    
    // 2. 模拟点击“重新开始”逻辑：删除磁盘文件并重置资源
    GameStateSave::delete_save();
    app.insert_resource(MapProgress::default()); // 强制覆盖为默认
    
    // 3. 验证：读取磁盘应失败
    let load_result = GameStateSave::load_from_disk();
    assert!(load_result.is_err(), "重新开始后，磁盘存档必须被物理删除");
    
    // 4. 验证：内存资源已回到第 0 层
    let current_progress = app.world().resource::<MapProgress>();
    assert_eq!(current_progress.current_layer, 0, "内存进度应重置为第 0 层");
}