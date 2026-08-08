use processing::prelude::BlendMode;
use processing::prelude::constants as c;
use pyo3::prelude::*;
use pyo3::types::PyModule;

use crate::PyBlendMode;

macro_rules! add {
    ($m:expr, $($name:ident),+ $(,)?) => {
        $( $m.add(stringify!($name), c::$name)?; )+
    };
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    add!(m, ROUND, SQUARE, PROJECT, MITER, BEVEL);
    add!(
        m,
        POLYGON,
        POINTS,
        LINES,
        TRIANGLES,
        TRIANGLE_FAN,
        TRIANGLE_STRIP,
        QUADS,
        QUAD_STRIP,
        LINE_STRIP
    );
    add!(m, CORNER, CORNERS, CENTER, RADIUS);
    add!(m, OPEN, CHORD, PIE, CLOSE);

    m.add("INVERT", crate::filter::INVERT_U8)?;
    m.add("GRAY", crate::filter::GRAY_U8)?;
    m.add("THRESHOLD", crate::filter::THRESHOLD_U8)?;
    m.add("POSTERIZE", crate::filter::POSTERIZE_U8)?;
    m.add("BLUR", crate::filter::BLUR_U8)?;
    m.add("OPAQUE", crate::filter::OPAQUE_U8)?;
    m.add("ERODE", crate::filter::ERODE_U8)?;
    m.add("DILATE", crate::filter::DILATE_U8)?;

    add!(m, LEFT, RIGHT);
    // text align (LEFT/RIGHT/CENTER above), style, and wrap constants
    add!(m, TOP, BOTTOM, BASELINE);
    add!(m, NORMAL, ITALIC, BOLD, BOLD_ITALIC);
    add!(m, WORD, CHAR);
    add!(m, NEAREST, CLAMP, REPEAT, MIRROR);
    add!(m, SRGB, LINEAR, HSL, HSV, HWB, OKLAB, OKLCH, LAB, LCH, XYZ);
    add!(
        m, PI, TWO_PI, HALF_PI, QUARTER_PI, TAU, DEG_TO_RAD, RAD_TO_DEG
    );

    add!(
        m, KEY_A, KEY_B, KEY_C, KEY_D, KEY_E, KEY_F, KEY_G, KEY_H, KEY_I, KEY_J, KEY_K, KEY_L,
        KEY_M, KEY_N, KEY_O, KEY_P, KEY_Q, KEY_R, KEY_S, KEY_T, KEY_U, KEY_V, KEY_W, KEY_X, KEY_Y,
        KEY_Z
    );
    add!(
        m, KEY_0, KEY_1, KEY_2, KEY_3, KEY_4, KEY_5, KEY_6, KEY_7, KEY_8, KEY_9
    );
    add!(
        m,
        SPACE,
        QUOTE,
        COMMA,
        MINUS,
        PERIOD,
        SLASH,
        SEMICOLON,
        EQUAL,
        BRACKET_LEFT,
        BACKSLASH,
        BRACKET_RIGHT,
        BACKQUOTE
    );
    add!(
        m,
        ESCAPE,
        ENTER,
        TAB,
        BACKSPACE,
        INSERT,
        DELETE,
        UP,
        DOWN,
        LEFT_ARROW,
        RIGHT_ARROW,
        PAGE_UP,
        PAGE_DOWN,
        HOME,
        END
    );
    add!(m, SHIFT, CONTROL, ALT, SUPER);
    add!(m, F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12);

    // Objects rather than strings, passed straight to blend_mode().
    m.add("BLEND", PyBlendMode::from_preset(BlendMode::Blend))?;
    m.add("ADD", PyBlendMode::from_preset(BlendMode::Add))?;
    m.add("SUBTRACT", PyBlendMode::from_preset(BlendMode::Subtract))?;
    m.add("DARKEST", PyBlendMode::from_preset(BlendMode::Darkest))?;
    m.add("LIGHTEST", PyBlendMode::from_preset(BlendMode::Lightest))?;
    m.add(
        "DIFFERENCE",
        PyBlendMode::from_preset(BlendMode::Difference),
    )?;
    m.add("EXCLUSION", PyBlendMode::from_preset(BlendMode::Exclusion))?;
    m.add("MULTIPLY", PyBlendMode::from_preset(BlendMode::Multiply))?;
    m.add("SCREEN", PyBlendMode::from_preset(BlendMode::Screen))?;
    m.add("REPLACE", PyBlendMode::from_preset(BlendMode::Replace))?;

    Ok(())
}
