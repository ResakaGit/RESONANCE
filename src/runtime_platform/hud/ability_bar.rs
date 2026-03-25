//! Barra QWER: fill = `cooldown_fraction` (buffer L5 / cost_qe). Sin timers en slots.

use bevy::prelude::*;
use bevy::text::{TextColor, TextFont};
use bevy::ui::{
    AlignItems, BackgroundColor, FlexDirection, JustifyContent, Node, PositionType, Val,
};

use crate::blueprint::equations;
use crate::layers::{AlchemicalEngine, Grimoire};
use crate::simulation::PlayerControlled;
use crate::simulation::ability_targeting::TargetingState;
use crate::simulation::states::{GameState, PlayState};

pub struct AbilityHudPlugin;

impl Plugin for AbilityHudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                ensure_ability_hud_system,
                sync_ability_bar_system,
                sync_targeting_hint_system,
            )
                .chain()
                .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active))),
        );
    }
}

#[derive(Component)]
struct AbilityHudRoot;

#[derive(Component)]
struct AbilityBarFill {
    slot: usize,
}

#[derive(Component)]
struct TargetingHintText;

fn ensure_ability_hud_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_root: Query<Entity, With<AbilityHudRoot>>,
) {
    if q_root.iter().next().is_some() {
        return;
    }

    let font: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");
    let keys = ["Q", "W", "E", "R"];

    commands
        .spawn((
            AbilityHudRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexEnd,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                padding: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },
        ))
        .with_children(|root| {
            root.spawn((
                TargetingHintText,
                Text::new(""),
                TextFont {
                    font: font.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 0.85, 0.35, 1.0)),
                Node {
                    padding: UiRect::horizontal(Val::Px(8.0)),
                    ..default()
                },
            ));
            root.spawn((Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(10.0),
                ..default()
            },))
                .with_children(|row| {
                    for i in 0..4 {
                        row.spawn((
                            Node {
                                width: Val::Px(58.0),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(4.0),
                                padding: UiRect::all(Val::Px(5.0)),
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.06, 0.07, 0.09, 0.88)),
                        ))
                        .with_children(|col| {
                            col.spawn((
                                Text::new(keys[i]),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 11.0,
                                    ..default()
                                },
                                TextColor(Color::srgba(0.92, 0.94, 1.0, 1.0)),
                            ));
                            col.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(10.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.14, 0.14, 0.16, 1.0)),
                            ))
                            .with_children(|track| {
                                track.spawn((
                                    AbilityBarFill { slot: i },
                                    Node {
                                        width: Val::Percent(0.0),
                                        height: Val::Percent(100.0),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.18, 0.62, 0.24, 1.0)),
                                ));
                            });
                        });
                    }
                });
        });
}

fn sync_ability_bar_system(
    q_player: Query<(&Grimoire, &AlchemicalEngine), With<PlayerControlled>>,
    mut q_fill: Query<(&AbilityBarFill, &mut BackgroundColor, &mut Node)>,
) {
    let Some((grim, eng)) = q_player.iter().next() else {
        return;
    };

    for (fill, mut bg, mut node) in &mut q_fill {
        let Some(slot) = grim.abilities().get(fill.slot) else {
            node.width = Val::Percent(0.0);
            continue;
        };
        let frac = equations::cooldown_fraction(eng, slot);
        let can = equations::can_cast(eng, slot);
        node.width = Val::Percent((frac * 100.0).clamp(0.0, 100.0));
        *bg = if can {
            BackgroundColor(Color::srgba(0.15, 0.65, 0.22, 1.0))
        } else {
            BackgroundColor(Color::srgba(0.62, 0.14, 0.14, 1.0))
        };
    }
}

fn sync_targeting_hint_system(
    targeting: Res<TargetingState>,
    mut q_text: Query<&mut Text, With<TargetingHintText>>,
) {
    const HINT: &str = "Targeting: click suelo para confirmar";
    for mut text in &mut q_text {
        let next = if targeting.active.is_some() { HINT } else { "" };
        if text.0 != next {
            text.0 = next.to_string();
        }
    }
}
