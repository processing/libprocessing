use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::{MouseButton, MouseButtonInput};
use bevy::prelude::*;
use bevy::window::CursorMoved;

use bevy::input::keyboard::KeyCode;

#[derive(Component, Default)]
pub struct CursorPosition {
    current: Vec2,
    previous: Vec2,
}

impl CursorPosition {
    pub fn current(&self) -> Vec2 {
        self.current
    }

    pub fn previous(&self) -> Vec2 {
        self.previous
    }
}

#[derive(Resource, Default)]
pub struct LastKey {
    pub code: Option<KeyCode>,
    pub character: Option<char>,
}

#[derive(Resource, Default)]
pub struct LastMouseButton {
    pub button: Option<MouseButton>,
}

pub fn snapshot_cursor(mut query: Query<&mut CursorPosition>) {
    for mut cursor in &mut query {
        cursor.previous = cursor.current;
    }
}

pub fn track_cursor_position(
    mut reader: MessageReader<CursorMoved>,
    mut query: Query<&mut CursorPosition>,
) {
    for event in reader.read() {
        if let Ok(mut cursor) = query.get_mut(event.window) {
            cursor.current = event.position;
        }
    }
}

pub fn track_last_key(mut reader: MessageReader<KeyboardInput>, mut last: ResMut<LastKey>) {
    if let Some(event) = reader
        .read()
        .filter(|e| e.state == ButtonState::Pressed)
        .last()
    {
        last.code = Some(event.key_code);
        last.character = event.text.as_ref().and_then(|t| t.chars().next());
    }
}

pub fn track_last_mouse_button(
    mut reader: MessageReader<MouseButtonInput>,
    mut last: ResMut<LastMouseButton>,
) {
    if let Some(event) = reader
        .read()
        .filter(|e| e.state == ButtonState::Pressed)
        .last()
    {
        last.button = Some(event.button);
    }
}
