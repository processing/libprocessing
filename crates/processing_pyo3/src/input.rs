use bevy::prelude::Entity;
use processing::prelude::*;
use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
};

pub fn mouse_x(surface: Entity) -> PyResult<f32> {
    processing::prelude::input_mouse_x(surface).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}

pub fn mouse_y(surface: Entity) -> PyResult<f32> {
    processing::prelude::input_mouse_y(surface).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}

pub fn pmouse_x(surface: Entity) -> PyResult<f32> {
    processing::prelude::input_pmouse_x(surface)
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}

pub fn pmouse_y(surface: Entity) -> PyResult<f32> {
    processing::prelude::input_pmouse_y(surface)
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}

pub fn mouse_is_pressed() -> PyResult<bool> {
    processing::prelude::input_mouse_is_pressed()
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}

pub fn mouse_button() -> PyResult<Option<String>> {
    processing::prelude::input_mouse_button()
        .map(|opt| {
            opt.map(|b| match b {
                MouseButton::Left => "LEFT".to_string(),
                MouseButton::Right => "RIGHT".to_string(),
                MouseButton::Middle => "CENTER".to_string(),
                _ => format!("{b:?}"),
            })
        })
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}

pub fn moved_x() -> PyResult<f32> {
    processing::prelude::input_moved_x().map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}

pub fn moved_y() -> PyResult<f32> {
    processing::prelude::input_moved_y().map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}

pub fn mouse_wheel() -> PyResult<f32> {
    processing::prelude::input_mouse_wheel().map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}

pub fn key_is_pressed() -> PyResult<bool> {
    processing::prelude::input_key_is_pressed().map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}

pub fn key_is_down(key_code: u32) -> PyResult<bool> {
    let kc = u32_to_key_code(key_code)?;
    processing::prelude::input_key_is_down(kc).map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}

pub fn key() -> PyResult<Option<String>> {
    processing::prelude::input_key()
        .map(|opt| opt.map(String::from))
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}

pub fn key_code() -> PyResult<Option<u32>> {
    processing::prelude::input_key_code()
        .map(|opt| opt.map(key_code_to_u32))
        .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
}

pub fn sync_globals(func: &Bound<'_, PyAny>, surface: Entity) -> PyResult<()> {
    let globals = func.getattr("__globals__")?;
    globals.set_item("mouse_x", mouse_x(surface)?)?;
    globals.set_item("mouse_y", mouse_y(surface)?)?;
    globals.set_item("pmouse_x", pmouse_x(surface)?)?;
    globals.set_item("pmouse_y", pmouse_y(surface)?)?;
    globals.set_item("mouse_is_pressed", mouse_is_pressed()?)?;
    globals.set_item("mouse_button", mouse_button()?)?;
    globals.set_item("moved_x", moved_x()?)?;
    globals.set_item("moved_y", moved_y()?)?;
    globals.set_item("mouse_wheel", mouse_wheel()?)?;
    globals.set_item("key", key()?)?;
    globals.set_item("key_code", key_code()?)?;
    globals.set_item("key_is_pressed", key_is_pressed()?)?;
    Ok(())
}

fn u32_to_key_code(val: u32) -> PyResult<bevy::input::keyboard::KeyCode> {
    use bevy::input::keyboard::KeyCode;
    match val {
        32 => Ok(KeyCode::Space),
        256 => Ok(KeyCode::Escape),
        257 => Ok(KeyCode::Enter),
        258 => Ok(KeyCode::Tab),
        259 => Ok(KeyCode::Backspace),
        261 => Ok(KeyCode::Delete),
        262 => Ok(KeyCode::ArrowRight),
        263 => Ok(KeyCode::ArrowLeft),
        264 => Ok(KeyCode::ArrowDown),
        265 => Ok(KeyCode::ArrowUp),
        340 => Ok(KeyCode::ShiftLeft),
        341 => Ok(KeyCode::ControlLeft),
        342 => Ok(KeyCode::AltLeft),
        343 => Ok(KeyCode::SuperLeft),
        65..=90 => Ok(match val {
            65 => KeyCode::KeyA,
            66 => KeyCode::KeyB,
            67 => KeyCode::KeyC,
            68 => KeyCode::KeyD,
            69 => KeyCode::KeyE,
            70 => KeyCode::KeyF,
            71 => KeyCode::KeyG,
            72 => KeyCode::KeyH,
            73 => KeyCode::KeyI,
            74 => KeyCode::KeyJ,
            75 => KeyCode::KeyK,
            76 => KeyCode::KeyL,
            77 => KeyCode::KeyM,
            78 => KeyCode::KeyN,
            79 => KeyCode::KeyO,
            80 => KeyCode::KeyP,
            81 => KeyCode::KeyQ,
            82 => KeyCode::KeyR,
            83 => KeyCode::KeyS,
            84 => KeyCode::KeyT,
            85 => KeyCode::KeyU,
            86 => KeyCode::KeyV,
            87 => KeyCode::KeyW,
            88 => KeyCode::KeyX,
            89 => KeyCode::KeyY,
            90 => KeyCode::KeyZ,
            _ => unreachable!(),
        }),
        48..=57 => Ok(match val {
            48 => KeyCode::Digit0,
            49 => KeyCode::Digit1,
            50 => KeyCode::Digit2,
            51 => KeyCode::Digit3,
            52 => KeyCode::Digit4,
            53 => KeyCode::Digit5,
            54 => KeyCode::Digit6,
            55 => KeyCode::Digit7,
            56 => KeyCode::Digit8,
            57 => KeyCode::Digit9,
            _ => unreachable!(),
        }),
        _ => Err(PyValueError::new_err(format!(
            "unknown key code: {val}"
        ))),
    }
}

fn key_code_to_u32(kc: bevy::input::keyboard::KeyCode) -> u32 {
    use bevy::input::keyboard::KeyCode;
    match kc {
        KeyCode::Space => 32,
        KeyCode::Escape => 256,
        KeyCode::Enter => 257,
        KeyCode::Tab => 258,
        KeyCode::Backspace => 259,
        KeyCode::Delete => 261,
        KeyCode::ArrowRight => 262,
        KeyCode::ArrowLeft => 263,
        KeyCode::ArrowDown => 264,
        KeyCode::ArrowUp => 265,
        KeyCode::ShiftLeft => 340,
        KeyCode::ControlLeft => 341,
        KeyCode::AltLeft => 342,
        KeyCode::SuperLeft => 343,
        KeyCode::KeyA => 65,
        KeyCode::KeyB => 66,
        KeyCode::KeyC => 67,
        KeyCode::KeyD => 68,
        KeyCode::KeyE => 69,
        KeyCode::KeyF => 70,
        KeyCode::KeyG => 71,
        KeyCode::KeyH => 72,
        KeyCode::KeyI => 73,
        KeyCode::KeyJ => 74,
        KeyCode::KeyK => 75,
        KeyCode::KeyL => 76,
        KeyCode::KeyM => 77,
        KeyCode::KeyN => 78,
        KeyCode::KeyO => 79,
        KeyCode::KeyP => 80,
        KeyCode::KeyQ => 81,
        KeyCode::KeyR => 82,
        KeyCode::KeyS => 83,
        KeyCode::KeyT => 84,
        KeyCode::KeyU => 85,
        KeyCode::KeyV => 86,
        KeyCode::KeyW => 87,
        KeyCode::KeyX => 88,
        KeyCode::KeyY => 89,
        KeyCode::KeyZ => 90,
        KeyCode::Digit0 => 48,
        KeyCode::Digit1 => 49,
        KeyCode::Digit2 => 50,
        KeyCode::Digit3 => 51,
        KeyCode::Digit4 => 52,
        KeyCode::Digit5 => 53,
        KeyCode::Digit6 => 54,
        KeyCode::Digit7 => 55,
        KeyCode::Digit8 => 56,
        KeyCode::Digit9 => 57,
        _ => 0,
    }
}
