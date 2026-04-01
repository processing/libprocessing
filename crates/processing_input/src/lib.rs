pub mod state;

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyCode, KeyboardInput, NativeKey};
use bevy::input::mouse::{
    AccumulatedMouseMotion, AccumulatedMouseScroll, MouseButton, MouseButtonInput, MouseMotion,
    MouseScrollUnit, MouseWheel,
};
use bevy::prelude::*;
use bevy::window::CursorMoved;

use processing_core::app_mut;
use processing_core::error;

pub use state::{CursorPosition, LastKey, LastMouseButton};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LastKey>()
            .init_resource::<LastMouseButton>()
            .add_systems(
                PreUpdate,
                (
                    state::snapshot_cursor,
                    (
                        state::track_cursor_position,
                        state::track_last_key,
                        state::track_last_mouse_button,
                    )
                        .after(state::snapshot_cursor),
                ),
            );
    }
}

pub fn input_set_mouse_move(surface: Entity, x: f32, y: f32) -> error::Result<()> {
    app_mut(|app| {
        let world = app.world_mut();
        let new_pos = Vec2::new(x, y);

        let delta = if let Some(cursor) = world.get::<CursorPosition>(surface) {
            let d = new_pos - cursor.current();
            Some(d)
        } else {
            world.entity_mut(surface).insert(CursorPosition::default());
            None
        };

        world.write_message(CursorMoved {
            window: surface,
            position: new_pos,
            delta,
        });

        if let Some(d) = delta {
            world.write_message(MouseMotion { delta: d });
        }

        Ok(())
    })
}

pub fn input_set_mouse_button(
    surface: Entity,
    button: MouseButton,
    pressed: bool,
) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut().write_message(MouseButtonInput {
            button,
            state: if pressed {
                ButtonState::Pressed
            } else {
                ButtonState::Released
            },
            window: surface,
        });
        Ok(())
    })
}

pub fn input_set_scroll(surface: Entity, x: f32, y: f32) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut().write_message(MouseWheel {
            unit: MouseScrollUnit::Pixel,
            x,
            y,
            window: surface,
        });
        Ok(())
    })
}

pub fn input_set_key(surface: Entity, key_code: KeyCode, pressed: bool) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut().write_message(KeyboardInput {
            key_code,
            logical_key: Key::Unidentified(NativeKey::Unidentified),
            state: if pressed {
                ButtonState::Pressed
            } else {
                ButtonState::Released
            },
            text: None,
            repeat: false,
            window: surface,
        });
        Ok(())
    })
}

pub fn input_set_char(surface: Entity, key_code: KeyCode, character: char) -> error::Result<()> {
    app_mut(|app| {
        let text = String::from(character);
        app.world_mut().write_message(KeyboardInput {
            key_code,
            logical_key: Key::Character(text.clone().into()),
            state: ButtonState::Pressed,
            text: Some(text.into()),
            repeat: false,
            window: surface,
        });
        Ok(())
    })
}

pub fn input_set_cursor_enter(surface: Entity) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .write_message(bevy::window::CursorEntered { window: surface });
        Ok(())
    })
}

pub fn input_set_cursor_leave(surface: Entity) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut()
            .write_message(bevy::window::CursorLeft { window: surface });
        Ok(())
    })
}

pub fn input_set_focus(surface: Entity, focused: bool) -> error::Result<()> {
    app_mut(|app| {
        app.world_mut().write_message(bevy::window::WindowFocused {
            window: surface,
            focused,
        });
        Ok(())
    })
}

pub fn input_cursor_grab_mode(surface: Entity) -> error::Result<bevy::window::CursorGrabMode> {
    app_mut(|app| {
        let cursor = app
            .world()
            .get::<bevy::window::CursorOptions>(surface)
            .map(|c| c.grab_mode)
            .unwrap_or(bevy::window::CursorGrabMode::None);
        Ok(cursor)
    })
}

pub fn input_cursor_visible(surface: Entity) -> error::Result<bool> {
    app_mut(|app| {
        let visible = app
            .world()
            .get::<bevy::window::CursorOptions>(surface)
            .map(|c| c.visible)
            .unwrap_or(true);
        Ok(visible)
    })
}

/// Flushes the input state by running the relevant schedules. This is required to ensure that
/// Bevy's bookkeeping of input state is up to date after manually sending input events.
/// It should be called at the end of each frame
pub fn input_flush() -> error::Result<()> {
    app_mut(|app| {
        let world = app.world_mut();
        world.run_schedule(First);
        world.run_schedule(PreUpdate);
        world.run_schedule(RunFixedMainLoop);
        Ok(())
    })
}

pub fn input_mouse_x(surface: Entity) -> error::Result<f32> {
    app_mut(|app| {
        let pos = app
            .world()
            .get::<CursorPosition>(surface)
            .map(|c| c.current())
            .unwrap_or(Vec2::ZERO);
        Ok(pos.x)
    })
}

pub fn input_mouse_y(surface: Entity) -> error::Result<f32> {
    app_mut(|app| {
        let pos = app
            .world()
            .get::<CursorPosition>(surface)
            .map(|c| c.current())
            .unwrap_or(Vec2::ZERO);
        Ok(pos.y)
    })
}

pub fn input_pmouse_x(surface: Entity) -> error::Result<f32> {
    app_mut(|app| {
        let pos = app
            .world()
            .get::<CursorPosition>(surface)
            .map(|c| c.previous())
            .unwrap_or(Vec2::ZERO);
        Ok(pos.x)
    })
}

pub fn input_pmouse_y(surface: Entity) -> error::Result<f32> {
    app_mut(|app| {
        let pos = app
            .world()
            .get::<CursorPosition>(surface)
            .map(|c| c.previous())
            .unwrap_or(Vec2::ZERO);
        Ok(pos.y)
    })
}

pub fn input_mouse_is_pressed() -> error::Result<bool> {
    app_mut(|app| {
        Ok(app
            .world()
            .resource::<ButtonInput<MouseButton>>()
            .get_pressed()
            .next()
            .is_some())
    })
}

pub fn input_mouse_button() -> error::Result<Option<MouseButton>> {
    app_mut(|app| Ok(app.world().resource::<LastMouseButton>().button))
}

pub fn input_moved_x() -> error::Result<f32> {
    app_mut(|app| Ok(app.world().resource::<AccumulatedMouseMotion>().delta.x))
}

pub fn input_moved_y() -> error::Result<f32> {
    app_mut(|app| Ok(app.world().resource::<AccumulatedMouseMotion>().delta.y))
}

pub fn input_mouse_wheel() -> error::Result<f32> {
    app_mut(|app| Ok(app.world().resource::<AccumulatedMouseScroll>().delta.y))
}

pub fn input_key_is_pressed() -> error::Result<bool> {
    app_mut(|app| {
        Ok(app
            .world()
            .resource::<ButtonInput<KeyCode>>()
            .get_pressed()
            .next()
            .is_some())
    })
}

pub fn input_key_just_pressed(key_code: KeyCode) -> error::Result<bool> {
    app_mut(|app| {
        Ok(app
            .world()
            .resource::<ButtonInput<KeyCode>>()
            .just_pressed(key_code))
    })
}

pub fn input_key_is_down(key_code: KeyCode) -> error::Result<bool> {
    app_mut(|app| {
        Ok(app
            .world()
            .resource::<ButtonInput<KeyCode>>()
            .pressed(key_code))
    })
}

pub fn input_key() -> error::Result<Option<char>> {
    app_mut(|app| Ok(app.world().resource::<LastKey>().character))
}

pub fn input_key_code() -> error::Result<Option<KeyCode>> {
    app_mut(|app| Ok(app.world().resource::<LastKey>().code))
}
