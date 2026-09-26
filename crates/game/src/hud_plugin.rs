use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

use crate::GameSystems;
use crate::building_plugin::{PALETTE, SelectedPaletteIndex};
use crate::input_plugin::{HudButton, TouchState};
use crate::locale;

const TOUCH_BUTTON_WIDTH: f32 = 112.0;
const TOUCH_BUTTON_HEIGHT: f32 = 64.0;
const TOUCH_BUTTON_GAP: f32 = 12.0;
const SCREEN_MARGIN: f32 = 24.0;
const SWATCH_SIZE: f32 = 56.0;
const SWATCH_BORDER: f32 = 4.0;
const JOYSTICK_INDICATOR_SIZE: f32 = 140.0;
const FRAME_RATE_REFRESHES_PER_SECOND: f32 = 4.0;
const BUTTON_COLOR: Color = Color::srgba(0.12, 0.16, 0.24, 0.72);
const SELECTED_SWATCH_BORDER: Color = Color::WHITE;
const UNSELECTED_SWATCH_BORDER: Color = Color::srgba(0.0, 0.0, 0.0, 0.25);

#[derive(Component)]
struct FrameRateText;

#[derive(Component)]
struct JoystickIndicator;

#[derive(Resource)]
struct FrameRateRefresh(Timer);

pub fn plugin(app: &mut App) {
    app.add_plugins(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(FrameRateRefresh(Timer::from_seconds(
            1.0 / FRAME_RATE_REFRESHES_PER_SECOND,
            TimerMode::Repeating,
        )))
        .add_systems(
            Startup,
            (
                spawn_frame_rate_text,
                spawn_palette,
                spawn_touch_buttons,
                spawn_joystick_indicator,
            ),
        )
        .add_systems(
            Update,
            (
                refresh_frame_rate_text,
                highlight_selected_swatch,
                follow_joystick,
            )
                .in_set(GameSystems::Hud),
        );
}

fn label(text: &str, font_size: f32) -> impl Bundle {
    (
        Text::new(text),
        TextFont {
            font_size: FontSize::Px(font_size),
            ..default()
        },
        TextColor(Color::WHITE),
    )
}

fn spawn_frame_rate_text(mut commands: Commands) {
    commands.spawn((
        FrameRateText,
        label("", 18.0),
        Node {
            position_type: PositionType::Absolute,
            left: px(SCREEN_MARGIN / 2.0),
            top: px(SCREEN_MARGIN / 2.0),
            ..default()
        },
    ));
}

fn spawn_palette(mut commands: Commands) {
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: px(SCREEN_MARGIN / 2.0),
            left: percent(50),
            width: px(0),
            justify_content: JustifyContent::Center,
            column_gap: px(TOUCH_BUTTON_GAP),
            ..default()
        })
        .with_children(|palette| {
            for (index, block) in PALETTE.iter().enumerate() {
                let [red, green, blue, alpha] = block.color();
                palette
                    .spawn((
                        HudButton::Palette(index),
                        Node {
                            width: px(SWATCH_SIZE + 2.0 * TOUCH_BUTTON_GAP),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: px(4),
                            ..default()
                        },
                    ))
                    .with_children(|swatch| {
                        swatch.spawn((
                            Node {
                                width: px(SWATCH_SIZE),
                                height: px(SWATCH_SIZE),
                                border: UiRect::all(px(SWATCH_BORDER)),
                                border_radius: BorderRadius::all(px(12)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(red, green, blue, alpha)),
                            BorderColor::all(UNSELECTED_SWATCH_BORDER),
                        ));
                        swatch.spawn(label(locale::block_name(*block), 14.0));
                    });
            }
        });
}

fn spawn_touch_buttons(mut commands: Commands) {
    let buttons = [
        (HudButton::Break, locale::BREAK),
        (HudButton::Place, locale::PLACE),
        (HudButton::Jump, locale::JUMP),
    ];
    for (column, (button, text)) in buttons.into_iter().enumerate() {
        let right = SCREEN_MARGIN
            + (buttons.len() - 1 - column) as f32 * (TOUCH_BUTTON_WIDTH + TOUCH_BUTTON_GAP);
        commands.spawn((
            button,
            Node {
                position_type: PositionType::Absolute,
                right: px(right),
                bottom: px(SCREEN_MARGIN),
                width: px(TOUCH_BUTTON_WIDTH),
                height: px(TOUCH_BUTTON_HEIGHT),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(px(18)),
                ..default()
            },
            BackgroundColor(BUTTON_COLOR),
            children![label(text, 22.0)],
        ));
    }
}

fn spawn_joystick_indicator(mut commands: Commands) {
    commands.spawn((
        JoystickIndicator,
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            width: px(JOYSTICK_INDICATOR_SIZE),
            height: px(JOYSTICK_INDICATOR_SIZE),
            border: UiRect::all(px(3)),
            border_radius: BorderRadius::MAX,
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.12)),
        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.6)),
    ));
}

fn refresh_frame_rate_text(
    time: Res<Time>,
    mut refresh: ResMut<FrameRateRefresh>,
    diagnostics: Res<DiagnosticsStore>,
    mut text: Single<&mut Text, With<FrameRateText>>,
) {
    if !refresh.0.tick(time.delta()).just_finished() {
        return;
    }
    if let Some(frames_per_second) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|diagnostic| diagnostic.smoothed())
    {
        text.0 = format!("{}: {frames_per_second:.0}", locale::FRAMES_PER_SECOND);
    }
}

fn highlight_selected_swatch(
    selected: Res<SelectedPaletteIndex>,
    swatches: Query<(&HudButton, &Children)>,
    mut borders: Query<&mut BorderColor>,
) {
    if !selected.is_changed() {
        return;
    }
    for (button, children) in &swatches {
        let HudButton::Palette(index) = button else {
            continue;
        };
        let color = if *index == selected.0 {
            SELECTED_SWATCH_BORDER
        } else {
            UNSELECTED_SWATCH_BORDER
        };
        for child in children {
            if let Ok(mut border) = borders.get_mut(*child) {
                *border = BorderColor::all(color);
            }
        }
    }
}

fn follow_joystick(
    touch_state: Res<TouchState>,
    mut indicator: Single<&mut Node, With<JoystickIndicator>>,
) {
    match touch_state.joystick {
        Some(joystick) => {
            indicator.display = Display::Flex;
            indicator.left = px(joystick.anchor.x - JOYSTICK_INDICATOR_SIZE / 2.0);
            indicator.top = px(joystick.anchor.y - JOYSTICK_INDICATOR_SIZE / 2.0);
        }
        None => indicator.display = Display::None,
    }
}
