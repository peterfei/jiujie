use bevy::prelude::*;
use rand::Rng;
use crate::components::combat::{Enemy, EnemyType, EnemyIntent, AiPattern};

/// 敌人生成器资源
#[derive(Resource)]
pub struct EnemyGenerator;

impl EnemyGenerator {
    /// 根据深度生成一个敌人实体 Bundle
    /// 
    /// # 参数
    /// * `depth` - 当前地图深度（层数），决定怪物强度
    /// * `id` - 敌人的唯一ID
    pub fn generate_enemy(depth: u32, id: u32) -> Enemy {
        let mut rng = rand::thread_rng();

        // 1. 根据深度选择原型
        let archetype = Self::pick_archetype(depth, &mut rng);

        // 2. 根据深度计算等级/境界
        // 简单模型：每层增加一点强度系数
        let scaling_factor = 1.0 + (depth as f32 * 0.1); 

        // 3. 生成基础数值
        let base_hp = (archetype.base_hp_range.0 as f32 * scaling_factor) as i32;
        // 确保HP在合理范围内波动
        let hp_variance = rng.gen_range(0.9..=1.1);
        let final_hp = (base_hp as f32 * hp_variance) as i32;

        // 4. 构建 Enemy 组件
        // 注意：目前 AI Pattern 是硬编码在 EnemyType 里的，后续可以解耦
        // 这里暂时复用 Enemy::with_type，但数值是计算出来的
        let mut enemy = Enemy::with_type(
            id,
            Self::generate_name(&archetype, depth),
            final_hp,
            archetype.enemy_type
        );
        
        // 5. 应用额外的数值修正（如果需要）
        // 例如：随深度增加攻击力 (strength)
        if depth > 5 {
            enemy.strength = ((depth - 5) as f32 * 0.5) as i32;
        }

        enemy
    }

    /// 生成指定深度的 Boss
    pub fn generate_boss(depth: u32, id: u32) -> Enemy {
        let mut rng = rand::thread_rng();
        // Boss 固定为筑基大妖（或者根据深度选择更强的 Boss）
        let archetype = EnemyArchetypeData::great_demon();
        
        let scaling_factor = 1.2 + (depth as f32 * 0.2); // Boss 成长系数更高
        let base_hp = (archetype.base_hp_range.0 as f32 * scaling_factor) as i32;
        let hp_variance = rng.gen_range(0.95..=1.05); // Boss 波动较小
        let final_hp = (base_hp as f32 * hp_variance) as i32;

        let mut enemy = Enemy::with_type(
            id,
            // Boss 名字更霸气
            format!("【镇守】{}", archetype.name),
            final_hp,
            archetype.enemy_type
        );

        // Boss 属性修正
        enemy.strength = (depth / 2) as i32 + 2;
        enemy.block = (depth / 2) as i32 + 5;
        
        // 修正 AI 数值范围
        enemy.ai_pattern.damage_range.0 += enemy.strength;
        enemy.ai_pattern.damage_range.1 += enemy.strength;

        enemy
    }

    fn pick_archetype(depth: u32, rng: &mut impl Rng) -> EnemyArchetypeData {
        // 简单的权重选择逻辑
        // 浅层多是狼和蜘蛛，深层出现怨灵和恶魔
        let roll: f32 = rng.gen();
        
        if depth < 3 {
            if roll < 0.6 {
                EnemyArchetypeData::demonic_wolf()
            } else {
                EnemyArchetypeData::poison_spider()
            }
        } else if depth < 7 {
            if roll < 0.4 {
                EnemyArchetypeData::demonic_wolf()
            } else if roll < 0.7 {
                EnemyArchetypeData::poison_spider()
            } else {
                EnemyArchetypeData::cursed_spirit()
            }
        } else {
            // 深层混合
            if roll < 0.3 {
                EnemyArchetypeData::demonic_wolf() // 即使是狼也是高等级的狼
            } else if roll < 0.5 {
                EnemyArchetypeData::poison_spider()
            } else if roll < 0.8 {
                EnemyArchetypeData::cursed_spirit()
            } else {
                EnemyArchetypeData::great_demon()
            }
        }
    }

    fn generate_name(archetype: &EnemyArchetypeData, depth: u32) -> String {
        // 简单的前缀生成
        let prefix = if depth <= 2 {
            "幼年"
        } else if depth <= 5 {
            "成年"
        } else if depth <= 8 {
            "狂暴"
        } else {
            "千年"
        };
        
        format!("{}{}", prefix, archetype.name)
    }
}

/// 内部使用的原型数据结构
struct EnemyArchetypeData {
    enemy_type: EnemyType,
    name: &'static str,
    base_hp_range: (i32, i32),
}

impl EnemyArchetypeData {
    fn demonic_wolf() -> Self {
        Self {
            enemy_type: EnemyType::DemonicWolf,
            name: "嗜血妖狼",
            base_hp_range: (25, 35),
        }
    }

    fn poison_spider() -> Self {
        Self {
            enemy_type: EnemyType::PoisonSpider,
            name: "剧毒魔蛛",
            base_hp_range: (40, 50),
        }
    }

    fn cursed_spirit() -> Self {
        Self {
            enemy_type: EnemyType::CursedSpirit,
            name: "怨灵",
            base_hp_range: (60, 80),
        }
    }

    fn great_demon() -> Self {
        Self {
            enemy_type: EnemyType::GreatDemon,
            name: "筑基大妖",
            base_hp_range: (150, 200),
        }
    }
}
