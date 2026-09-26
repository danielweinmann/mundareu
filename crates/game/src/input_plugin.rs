use bevy::input::InputSystems;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

const JOYSTICK_RADIUS_IN_PIXELS: f32 = 70.0;
const TOUCH_LOOK_RADIANS_PER_PIXEL: f32 = 0.006;
const MOUSE_LOOK_RADIANS_PER_PIXEL: f32 = 0.004;
const CLICK_TRAVEL_LIMIT_IN_PIXELS: f32 = 6.0;

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct PlayerIntent {
    pub movement: Vec2,
    pub look: Vec2,
    pub jump: bool,
    pub break_block: bool,
    pub place_block: bool,
    pub select_palette_index: Option<usize>,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum HudButton {
    Jump,
    Place,
    Break,
    Palette(usize),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Joystick {
    pub touch_id: u64,
    pub anchor: Vec2,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct LookFinger {
    touch_id: u64,
    last_position: Vec2,
}

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct TouchState {
    pub joystick: Option<Joystick>,
    look_finger: Option<LookFinger>,
}

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
struct RightButtonDrag {
    travel_in_pixels: f32,
}

type HudButtonQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static HudButton,
        &'static ComputedNode,
        &'static UiGlobalTransform,
    ),
>;

pub fn plugin(app: &mut App) {
    app.init_resource::<PlayerIntent>()
        .init_resource::<TouchState>()
        .init_resource::<RightButtonDrag>()
        .add_systems(
            PreUpdate,
            (read_touch_input, read_desktop_input)
                .chain()
                .after(InputSystems),
        );
}

fn read_touch_input(
    touches: Res<Touches>,
    window: Single<&Window, With<PrimaryWindow>>,
    buttons: HudButtonQuery,
    mut touch_state: ResMut<TouchState>,
    mut intent: ResMut<PlayerIntent>,
) {
    for touch in touches.iter_just_pressed() {
        if let Some(button) = button_at(&window, touch.position(), &buttons) {
            press(button, &mut intent);
        } else if touch.position().x < window.width() / 2.0 {
            if touch_state.joystick.is_none() {
                touch_state.joystick = Some(Joystick {
                    touch_id: touch.id(),
                    anchor: touch.position(),
                });
            }
        } else if touch_state.look_finger.is_none() {
            touch_state.look_finger = Some(LookFinger {
                touch_id: touch.id(),
                last_position: touch.position(),
            });
        }
    }
    intent.movement = Vec2::ZERO;
    if let Some(joystick) = touch_state.joystick {
        match touches.get_pressed(joystick.touch_id) {
            Some(touch) => intent.movement = joystick_movement(joystick.anchor, touch.position()),
            None => touch_state.joystick = None,
        }
    }
    if let Some(look_finger) = &mut touch_state.look_finger {
        match touches.get_pressed(look_finger.touch_id) {
            Some(touch) => {
                intent.look +=
                    (touch.position() - look_finger.last_position) * TOUCH_LOOK_RADIANS_PER_PIXEL;
                look_finger.last_position = touch.position();
            }
            None => touch_state.look_finger = None,
        }
    }
}

fn read_desktop_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    window: Single<&Window, With<PrimaryWindow>>,
    buttons: HudButtonQuery,
    mut right_button_drag: ResMut<RightButtonDrag>,
    mut intent: ResMut<PlayerIntent>,
) {
    intent.movement = (intent.movement + keyboard_movement(&keys)).clamp_length_max(1.0);
    if keys.just_pressed(KeyCode::Space) {
        intent.jump = true;
    }
    let palette_keys = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
    ];
    if let Some(index) = palette_keys.iter().position(|key| keys.just_pressed(*key)) {
        intent.select_palette_index = Some(index);
    }

    let button_under_cursor = window
        .cursor_position()
        .and_then(|cursor| button_at(&window, cursor, &buttons));
    if mouse.just_pressed(MouseButton::Left) {
        match button_under_cursor {
            Some(button) => press(button, &mut intent),
            None => intent.break_block = true,
        }
    }
    if mouse.just_pressed(MouseButton::Right) {
        right_button_drag.travel_in_pixels = 0.0;
    }
    if mouse.pressed(MouseButton::Right) {
        intent.look += mouse_motion.delta * MOUSE_LOOK_RADIANS_PER_PIXEL;
        right_button_drag.travel_in_pixels += mouse_motion.delta.length();
    }
    if mouse.just_released(MouseButton::Right)
        && right_button_drag.travel_in_pixels < CLICK_TRAVEL_LIMIT_IN_PIXELS
        && button_under_cursor.is_none()
    {
        intent.place_block = true;
    }
}

fn press(button: HudButton, intent: &mut PlayerIntent) {
    match button {
        HudButton::Jump => intent.jump = true,
        HudButton::Place => intent.place_block = true,
        HudButton::Break => intent.break_block = true,
        HudButton::Palette(index) => intent.select_palette_index = Some(index),
    }
}

fn button_at(
    window: &Window,
    logical_position: Vec2,
    buttons: &HudButtonQuery,
) -> Option<HudButton> {
    let physical_position = logical_position * window.scale_factor();
    buttons
        .iter()
        .find(|(_, node, transform)| node.contains_point(**transform, physical_position))
        .map(|(button, _, _)| *button)
}

fn keyboard_movement(keys: &ButtonInput<KeyCode>) -> Vec2 {
    let axis = |negative: [KeyCode; 2], positive: [KeyCode; 2]| {
        let pressed = |keys_for_direction: [KeyCode; 2]| {
            keys_for_direction.iter().any(|key| keys.pressed(*key))
        };
        f32::from(pressed(positive)) - f32::from(pressed(negative))
    };
    Vec2::new(
        axis(
            [KeyCode::KeyA, KeyCode::ArrowLeft],
            [KeyCode::KeyD, KeyCode::ArrowRight],
        ),
        axis(
            [KeyCode::KeyS, KeyCode::ArrowDown],
            [KeyCode::KeyW, KeyCode::ArrowUp],
        ),
    )
}

pub fn joystick_movement(anchor: Vec2, position: Vec2) -> Vec2 {
    let offset = (position - anchor) / JOYSTICK_RADIUS_IN_PIXELS;
    Vec2::new(offset.x, -offset.y).clamp_length_max(1.0)
}

#[cfg(test)]
mod tests {
    use bevy::input::InputPlugin;
    use bevy::input::touch::TouchPhase;

    use super::*;

    fn touch_input(phase: TouchPhase, position: Vec2, window: Entity) -> TouchInput {
        TouchInput {
            phase,
            position,
            window,
            force: None,
            id: 7,
        }
    }

    #[test]
    fn a_look_finger_resting_still_stops_turning_the_camera() {
        let mut app = App::new();
        app.add_plugins((InputPlugin, plugin));
        let window = app
            .world_mut()
            .spawn((Window::default(), PrimaryWindow))
            .id();
        app.world_mut().write_message(touch_input(
            TouchPhase::Started,
            Vec2::new(1000.0, 300.0),
            window,
        ));
        app.update();
        app.world_mut().write_message(touch_input(
            TouchPhase::Moved,
            Vec2::new(1040.0, 300.0),
            window,
        ));
        app.update();
        let look_after_dragging = app.world().resource::<PlayerIntent>().look;
        app.update();
        app.update();
        assert!(look_after_dragging.x > 0.0);
        assert_eq!(
            app.world().resource::<PlayerIntent>().look,
            look_after_dragging
        );
    }

    #[test]
    fn dragging_the_joystick_up_the_screen_moves_forward() {
        let anchor = Vec2::new(100.0, 500.0);
        let movement = joystick_movement(anchor, anchor + Vec2::new(0.0, -35.0));
        assert_eq!(movement, Vec2::new(0.0, 0.5));
    }

    #[test]
    fn dragging_beyond_the_joystick_radius_saturates_at_full_speed() {
        let anchor = Vec2::new(100.0, 500.0);
        let movement = joystick_movement(anchor, anchor + Vec2::new(300.0, 0.0));
        assert_eq!(movement, Vec2::new(1.0, 0.0));
    }

    #[test]
    fn a_resting_joystick_produces_no_movement() {
        let anchor = Vec2::new(100.0, 500.0);
        assert_eq!(joystick_movement(anchor, anchor), Vec2::ZERO);
    }

    #[test]
    fn pressing_a_hud_button_sets_the_matching_intent() {
        let mut intent = PlayerIntent::default();
        press(HudButton::Jump, &mut intent);
        press(HudButton::Palette(3), &mut intent);
        assert!(intent.jump);
        assert!(!intent.break_block);
        assert_eq!(intent.select_palette_index, Some(3));
    }
}
