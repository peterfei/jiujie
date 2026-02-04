use bevy::prelude::*;
use rand::Rng;
use crate::components::combat::{Enemy, EnemyType, EnemyIntent, AiPattern, EnemyAffix};

/// 生成的敌人结果，包含组件数据和视觉配置
pub struct GeneratedEnemy {
    /// 敌人核心组件
    pub enemy: Enemy,
    /// 视觉缩放比例 (相对于默认尺寸)
    pub visual_scale: Vec2,
    /// 视觉染色
    pub visual_color: Color,
}

/// 敌人生成器资源
#[derive(Resource)]
pub struct EnemyGenerator;

impl EnemyGenerator {
    /// 根据深度生成一个敌人
    pub fn generate_enemy(depth: u32, id: u32) -> GeneratedEnemy {
        let mut rng = rand::thread_rng();

        // 1. 选择原型
        let archetype = Self::pick_archetype(depth, &mut rng);

        // 2. 基础数值计算
        let scaling_factor = 1.0 + (depth as f32 * 0.2); 
        let base_hp = (archetype.base_hp_range.0 as f32 * scaling_factor) as i32;
        let hp_variance = rng.gen_range(0.9..=1.1);
        let mut final_hp = (base_hp as f32 * hp_variance) as i32;

        // 3. 构建 Enemy 组件
        let mut enemy = Enemy::with_type(
            id,
            "".to_string(), // 稍后生成名字
            final_hp,
            archetype.enemy_type
        );
        
        // 随深度增加基础攻击力
        if depth > 5 {
            enemy.strength = ((depth - 5) as f32 * 0.5) as i32;
        }

        // 4. 随机生成词缀
        let affixes = Self::roll_affixes(depth, &mut rng);
        enemy.affixes = affixes.clone();

        // 5. 应用词缀数值修正
        let mut visual_scale = Vec2::ONE;
        let mut visual_color = Color::WHITE;

        for affix in &affixes {
            match affix {
                EnemyAffix::Elite => {
                    final_hp = (final_hp as f32 * 1.5) as i32;
                    enemy.strength += 2;
                    enemy.block += 5;
                    visual_scale *= 1.3;
                    visual_color = Color::srgba(1.0, 0.9, 0.6, 1.0); // 金色
                }
                EnemyAffix::Weak => {
                    final_hp = (final_hp as f32 * 0.7) as i32;
                    enemy.strength = (enemy.strength - 1).max(0);
                    visual_scale *= 0.8;
                    visual_color = Color::srgba(0.8, 0.8, 0.8, 1.0); // 灰色
                }
                EnemyAffix::Berserk => {
                    enemy.strength += (depth as i32 / 3).max(3); // 攻击大幅提升
                    final_hp = (final_hp as f32 * 0.8) as i32;   // 血量稍减
                    visual_color = Color::srgba(1.0, 0.4, 0.4, 1.0); // 红色
                    // 修改 AI: 极高攻击概率
                    enemy.ai_pattern.attack_chance = 0.9;
                    enemy.ai_pattern.defend_chance = 0.0;
                }
                EnemyAffix::Tank => {
                    final_hp = (final_hp as f32 * 1.3) as i32;
                    enemy.block += 10;
                    visual_scale *= 1.1;
                    visual_color = Color::srgba(0.4, 0.6, 1.0, 1.0); // 蓝色
                    // 修改 AI: 高防御概率
                    enemy.ai_pattern.defend_chance += 0.3;
                    enemy.ai_pattern.attack_chance -= 0.2;
                }
                EnemyAffix::Swift => {
                    visual_color = Color::srgba(0.4, 1.0, 0.8, 1.0); // 青色
                }
                EnemyAffix::Fire => {
                    visual_color = Color::srgba(1.0, 0.3, 0.0, 1.0); // 橘红
                    enemy.strength += 1;
                }
                EnemyAffix::Poison => {
                    visual_color = Color::srgba(0.2, 0.8, 0.2, 1.0); // 毒绿
                }
                EnemyAffix::Ice => {
                    visual_color = Color::srgba(0.4, 0.8, 1.0, 1.0); // 冰蓝
                    enemy.block += 3;
                }
            }
        }
        
        enemy.hp = final_hp;
        enemy.max_hp = final_hp;

        // 6. 生成最终名称
        enemy.name = Self::generate_name(&archetype, depth, &affixes);

        GeneratedEnemy {
            enemy,
            visual_scale,
            visual_color,
        }
    }

    /// 生成指定深度的 Boss
    pub fn generate_boss(depth: u32, id: u32) -> GeneratedEnemy {
        let mut rng = rand::thread_rng();
        let archetype = EnemyArchetypeData::great_demon();
        
        let scaling_factor = 1.2 + (depth as f32 * 0.2); 
        let base_hp = (archetype.base_hp_range.0 as f32 * scaling_factor) as i32;
        let hp_variance = rng.gen_range(0.95..=1.05); 
        let final_hp = (base_hp as f32 * hp_variance) as i32;

        let mut enemy = Enemy::with_type(
            id,
            format!("【镇守】{}", archetype.name),
            final_hp,
            archetype.enemy_type
        );

        enemy.strength = (depth / 2) as i32 + 2;
        enemy.block = (depth / 2) as i32 + 5;
        
        enemy.ai_pattern.damage_range.0 += enemy.strength;
        enemy.ai_pattern.damage_range.1 += enemy.strength;

        // Boss 默认自带精英属性的视觉效果，但不加 Elite 词缀以免数值膨胀过度
        // 或者我们可以给 Boss 加一些特殊词缀
        
        GeneratedEnemy {
            enemy,
            visual_scale: Vec2::splat(1.5), // Boss 默认巨大
            visual_color: Color::WHITE,
        }
    }

    fn roll_affixes(depth: u32, rng: &mut impl Rng) -> Vec<EnemyAffix> {
        let mut affixes = Vec::new();
        let roll: f32 = rng.gen();

        // 词缀生成概率随深度增加
        let affix_chance = (0.1 + depth as f32 * 0.05).min(0.8);

        if roll < affix_chance {
            // 随机选择一个词缀
            let affix_pool = [
                EnemyAffix::Elite,
                EnemyAffix::Berserk,
                EnemyAffix::Tank,
                EnemyAffix::Swift,
            ];
            // 深度较低时也有可能出现 Weak
            if depth < 3 && rng.gen_bool(0.2) {
                affixes.push(EnemyAffix::Weak);
            } else {
                let choice = affix_pool[rng.gen_range(0..affix_pool.len())];
                affixes.push(choice);
            }
        }

        affixes
    }

    fn pick_archetype(depth: u32, rng: &mut impl Rng) -> EnemyArchetypeData {
        let roll: f32 = rng.gen();
        if depth < 3 {
            if roll < 0.6 { EnemyArchetypeData::demonic_wolf() } else { EnemyArchetypeData::poison_spider() }
        } else if depth < 7 {
            if roll < 0.4 { EnemyArchetypeData::demonic_wolf() } else if roll < 0.7 { EnemyArchetypeData::poison_spider() } else { EnemyArchetypeData::cursed_spirit() }
        } else {
            if roll < 0.3 { EnemyArchetypeData::demonic_wolf() } else if roll < 0.5 { EnemyArchetypeData::poison_spider() } else if roll < 0.8 { EnemyArchetypeData::cursed_spirit() } else { EnemyArchetypeData::great_demon() }
        }
    }

    fn generate_name(archetype: &EnemyArchetypeData, depth: u32, affixes: &[EnemyAffix]) -> String {
        let age_prefix = if depth <= 2 { "幼年" } else if depth <= 5 { "成年" } else if depth <= 8 { "狂暴" } else { "千年" };
        
        let affix_prefix = if let Some(affix) = affixes.first() {
            match affix {
                EnemyAffix::Elite => "精英",
                EnemyAffix::Weak => "虚弱的",
                EnemyAffix::Berserk => "嗜血",
                EnemyAffix::Tank => "铁甲",
                EnemyAffix::Swift => "疾风",
                EnemyAffix::Fire => "烈焰",
                EnemyAffix::Poison => "剧毒",
                EnemyAffix::Ice => "寒冰",
            }
        } else {
            ""
        };

        if affix_prefix.is_empty() {
            format!("{}{}", age_prefix, archetype.name)
        } else {
            format!("{} {}{}", affix_prefix, age_prefix, archetype.name)
        }
    }
}

struct EnemyArchetypeData {
    enemy_type: EnemyType,
    name: &'static str,
    base_hp_range: (i32, i32),
}

impl EnemyArchetypeData {
    fn demonic_wolf() -> Self { Self { enemy_type: EnemyType::DemonicWolf, name: "妖狼", base_hp_range: (25, 35) } }
    fn poison_spider() -> Self { Self { enemy_type: EnemyType::PoisonSpider, name: "毒蛛", base_hp_range: (40, 50) } }
    fn cursed_spirit() -> Self { Self { enemy_type: EnemyType::CursedSpirit, name: "怨灵", base_hp_range: (60, 80) } }
    fn great_demon() -> Self { Self { enemy_type: EnemyType::GreatDemon, name: "大妖", base_hp_range: (150, 200) } }
}