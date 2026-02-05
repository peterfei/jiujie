//! 全局资源和状态管理

pub mod save;



use bevy::prelude::*;



/// 环境氛围配置

#[derive(Resource, Clone)]

pub struct EnvironmentConfig {

    pub wind_strength: f32,

    pub fog_color: Color,

    pub ambient_brightness: f32,

}



impl Default for EnvironmentConfig {

    fn default() -> Self {

        Self {

            wind_strength: 1.0,

            fog_color: Color::srgba(0.01, 0.005, 0.02, 1.0),

            ambient_brightness: 0.1,

        }

    }

}



/// 程序化地形生成器

#[derive(Resource)]

pub struct LandscapeGenerator {

    pub seed: u64,

}



impl LandscapeGenerator {

    pub fn new(seed: u64) -> Self {

        Self { seed }

    }

}

#[derive(Resource, Default)]
pub struct EnvironmentAssets {
    pub rock: Handle<Scene>,
    pub cloud: Handle<Scene>,
    pub bush: Handle<Scene>,
    pub shrub: Handle<Scene>,
    pub lantern: Handle<Scene>, // 新增：灯笼
    pub rune_stone: Handle<Scene>, // 新增：符文石
    pub berries: Handle<Scene>, // 新增：浆果丛
}
