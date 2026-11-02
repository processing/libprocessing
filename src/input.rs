//! Keyboard and mouse identifiers.

use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseButton as BevyMouseButton;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u16),
}

impl From<BevyMouseButton> for MouseButton {
    fn from(b: BevyMouseButton) -> Self {
        match b {
            BevyMouseButton::Left => Self::Left,
            BevyMouseButton::Right => Self::Right,
            BevyMouseButton::Middle => Self::Middle,
            BevyMouseButton::Back => Self::Back,
            BevyMouseButton::Forward => Self::Forward,
            BevyMouseButton::Other(n) => Self::Other(n),
        }
    }
}

macro_rules! keys {
    ($($key:ident => $code:ident),* $(,)?) => {
        /// A physical key, named by its position on a US keyboard.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Key {
            $($key,)*
        }

        impl Key {
            pub(crate) fn code(self) -> KeyCode {
                match self {
                    $(Self::$key => KeyCode::$code,)*
                }
            }

            pub(crate) fn from_code(code: KeyCode) -> Option<Self> {
                match code {
                    $(KeyCode::$code => Some(Self::$key),)*
                    _ => None,
                }
            }
        }
    };
}

keys!(
    A => KeyA, B => KeyB, C => KeyC, D => KeyD, E => KeyE, F => KeyF, G => KeyG, H => KeyH,
    I => KeyI, J => KeyJ, K => KeyK, L => KeyL, M => KeyM, N => KeyN, O => KeyO, P => KeyP,
    Q => KeyQ, R => KeyR, S => KeyS, T => KeyT, U => KeyU, V => KeyV, W => KeyW, X => KeyX,
    Y => KeyY, Z => KeyZ, Num0 => Digit0, Num1 => Digit1, Num2 => Digit2, Num3 => Digit3,
    Num4 => Digit4, Num5 => Digit5, Num6 => Digit6, Num7 => Digit7, Num8 => Digit8, Num9 => Digit9,
    Up => ArrowUp, Down => ArrowDown, Left => ArrowLeft, Right => ArrowRight, Space => Space,
    Enter => Enter, Escape => Escape, Tab => Tab, Backspace => Backspace, Delete => Delete,
    Insert => Insert, Home => Home, End => End, PageUp => PageUp, PageDown => PageDown,
    ShiftLeft => ShiftLeft, ShiftRight => ShiftRight, ControlLeft => ControlLeft,
    ControlRight => ControlRight, AltLeft => AltLeft, AltRight => AltRight, SuperLeft => SuperLeft,
    SuperRight => SuperRight, Minus => Minus, Equal => Equal, BracketLeft => BracketLeft,
    BracketRight => BracketRight, Backslash => Backslash, Semicolon => Semicolon, Quote => Quote,
    Backquote => Backquote, Comma => Comma, Period => Period, Slash => Slash, F1 => F1, F2 => F2,
    F3 => F3, F4 => F4, F5 => F5, F6 => F6, F7 => F7, F8 => F8, F9 => F9, F10 => F10, F11 => F11,
    F12 => F12,
);
