use bevy::prelude::*;
use crate::components::combat::{Player, Enemy, HandArea, CardDescriptionMarker};
use crate::components::cards::{Hand, Card, CardType, CardEffect, DrawPile, DiscardPile};
use crate::plugins::{HandCard, HandCountText, DrawPileText, DiscardPileText};

/// [大作级] 增强版手牌更新系统 (V3 - 绝对稳定版)
pub fn update_hand_ui_v2(
    hand_query: Query<&Hand>,
    player_query: Query<&Player>,
    enemy_query: Query<&Enemy>,
    draw_pile_query: Query<&DrawPile>,
    discard_pile_query: Query<&DiscardPile>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<DrawPileText>>,
        Query<&mut Text, With<DiscardPileText>>,
        Query<&mut Text, With<HandCountText>>,
        Query<(&CardDescriptionMarker, &mut Text, &mut TextColor)>,
    )>,
    hand_area_query: Query<(Entity, Option<&Children>), With<HandArea>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut last_hand_ids: Local<Vec<u32>>, // 核心保险：记录上次手牌 ID 列表
) {
    let hand = match hand_query.get_single() {
        Ok(h) => h,
        Err(_) => return,
    };

    // 1. 同步堆栈数字 (低开销)
    if let Ok(draw_pile) = draw_pile_query.get_single() {
        if let Ok(mut text) = text_queries.p0().get_single_mut() { text.0 = format!("剑冢: {}", draw_pile.count); }
    }
    if let Ok(discard_pile) = discard_pile_query.get_single() {
        if let Ok(mut text) = text_queries.p1().get_single_mut() { text.0 = format!("归墟: {}", discard_pile.count); }
    }

    // 2. 检测卡牌构成是否真的改变 (数量、ID、顺序)
    let current_ids: Vec<u32> = hand.cards.iter().map(|c| c.id).collect();
    let structure_changed = current_ids != *last_hand_ids;

    // 3. 动态数值预览同步 (即便结构没变，数值也要每帧刷新，但不重建实体)
    if let Ok(player) = player_query.get_single() {
        let default_enemy = enemy_query.iter().find(|e| e.hp > 0);
        for (marker, mut text, mut color) in text_queries.p3().iter_mut() {
            if let Some(card) = hand.cards.iter().find(|c| c.id == marker.card_id) {
                if card.card_type == CardType::Attack {
                    let base_damage = card.effect_amount();
                    let mut final_damage = player.calculate_outgoing_damage(base_damage);
                    if let Some(enemy) = default_enemy {
                        final_damage = enemy.calculate_incoming_damage(final_damage);
                    }
                    
                    let new_text = if final_damage != base_damage {
                        format!("造成{}点伤害", final_damage)
                    } else {
                        card.description.clone()
                    };
                    
                    let new_color = if final_damage > base_damage {
                        Color::srgb(1.0, 0.4, 0.4) 
                    } else if final_damage < base_damage {
                        Color::srgb(0.4, 1.0, 0.4) 
                    } else {
                        Color::srgb(0.9, 0.9, 0.9)
                    };

                    if text.0 != new_text { text.0 = new_text; }
                    if color.0 != new_color { color.0 = new_color; }
                }
            }
        }
    }

    // 4. 只有当结构真正变化时，才重建 UI 实体
    if structure_changed {
        if let Some((hand_area_entity, _)) = hand_area_query.iter().next() {
            info!("【稳定版】检测到手牌构成变化，重建实体。旧: {:?}, 新: {:?}", *last_hand_ids, current_ids);
            
            // 更新保险记录
            *last_hand_ids = current_ids;
            
            commands.entity(hand_area_entity).despawn_descendants();
            let chinese_font = asset_server.load("fonts/Arial Unicode.ttf");

            // A. 计数文本
            commands.entity(hand_area_entity).with_children(|parent| {
                parent.spawn((
                    Text::new(format!("手牌: {}/{}", hand.cards.len(), hand.max_size)),
                    TextFont { font: chinese_font.clone(), font_size: 18.0, ..default() },
                    TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
                    Node { position_type: PositionType::Absolute, top: Val::Px(10.0), ..default() },
                    HandCountText
                ));
            });

            // B. 卡牌实体生成
            let total_cards = hand.cards.len();
            let center_index = (total_cards as f32 - 1.0) / 2.0;

            for (i, card) in hand.cards.iter().enumerate() {
                let offset_from_center = i as f32 - center_index;
                let x_pos = offset_from_center * 95.0; 
                let base_bottom = 20.0 + (-3.0 * offset_from_center * offset_from_center);
                let rotation = -offset_from_center * 0.06;

                // 初始数值计算
                let mut display_desc = card.description.clone();
                let mut desc_color = Color::srgb(0.9, 0.9, 0.9);
                if let Ok(player) = player_query.get_single() {
                    if card.card_type == CardType::Attack {
                        let base_damage = card.effect_amount();
                        let mut final_damage = player.calculate_outgoing_damage(base_damage);
                        if let Some(enemy) = enemy_query.iter().find(|e| e.hp > 0) {
                            final_damage = enemy.calculate_incoming_damage(final_damage);
                        }
                        if final_damage != base_damage {
                            display_desc = format!("造成{}点伤害", final_damage);
                            desc_color = if final_damage > base_damage { Color::srgb(1.0, 0.4, 0.4) } else { Color::srgb(0.4, 1.0, 0.4) };
                        }
                    }
                }

                commands.entity(hand_area_entity).with_children(|parent| {
                    parent.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Px(105.0),
                            height: Val::Px(145.0),
                            left: Val::Px(x_pos + 450.0),
                            bottom: Val::Px(base_bottom),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            padding: UiRect::all(Val::Px(6.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        Transform::from_rotation(Quat::from_rotation_z(rotation)),
                        BackgroundColor(card.get_color()),
                        BorderColor(Color::BLACK),
                        HandCard { card_id: card.id, base_bottom, base_rotation: rotation, index: i },
                        Button,
                    )).with_children(|card_ui| {
                        card_ui.spawn((Text::new(format!("{}", card.cost)), TextFont { font: chinese_font.clone(), font_size: 18.0, ..default() }, TextColor(Color::WHITE)));
                        card_ui.spawn((Text::new(card.name.clone()), TextFont { font: chinese_font.clone(), font_size: 14.0, ..default() }, TextColor(Color::WHITE), TextLayout::new_with_justify(JustifyText::Center)));
                        let img_path = if card.image_path.is_empty() { "textures/cards/default.png".to_string() } else { card.image_path.clone() };
                        card_ui.spawn((ImageNode::new(asset_server.load(img_path)), Node { width: Val::Percent(95.0), height: Val::Percent(55.0), border: UiRect::all(Val::Px(1.0)), ..default() }, BorderColor(Color::srgba(1.0, 1.0, 1.0, 0.3))));
                        
                        card_ui.spawn((
                            Text::new(display_desc),
                            TextFont { font: chinese_font.clone(), font_size: 11.0, ..default() },
                            TextColor(desc_color),
                            TextLayout::new_with_justify(JustifyText::Center),
                            Node { max_width: Val::Px(90.0), ..default() },
                            CardDescriptionMarker { card_id: card.id },
                        ));
                    });
                });
            }
        }
    }
}