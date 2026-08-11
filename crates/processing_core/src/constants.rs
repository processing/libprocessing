pub const CORNER: &str = "corner";
pub const CORNERS: &str = "corners";
pub const RADIUS: &str = "radius";
pub const CENTER: &str = "center"; // also the middle mouse button

pub const ROUND: &str = "round"; // caps and joins
pub const SQUARE: &str = "square";
pub const PROJECT: &str = "project";
pub const MITER: &str = "miter";
pub const BEVEL: &str = "bevel";

pub const POLYGON: &str = "polygon";
pub const POINTS: &str = "points";
pub const LINES: &str = "lines";
pub const TRIANGLES: &str = "triangles";
pub const TRIANGLE_FAN: &str = "triangle_fan";
pub const TRIANGLE_STRIP: &str = "triangle_strip";
pub const QUADS: &str = "quads";
pub const QUAD_STRIP: &str = "quad_strip";
pub const LINE_STRIP: &str = "line_strip"; // geometry topology; no begin_shape equivalent

pub const OPEN: &str = "open";
pub const CHORD: &str = "chord";
pub const PIE: &str = "pie";
pub const CLOSE: bool = true;

pub const LEFT: &str = "left";
pub const RIGHT: &str = "right";

pub const TOP: &str = "top"; // vertical text align
pub const BOTTOM: &str = "bottom";
pub const BASELINE: &str = "baseline";

pub const NORMAL: &str = "normal"; // text style
pub const ITALIC: &str = "italic";
pub const BOLD: &str = "bold";
pub const BOLD_ITALIC: &str = "bold_italic";

pub const WORD: &str = "word"; // text wrap mode
pub const CHAR: &str = "char";

pub const NEAREST: &str = "nearest";
pub const CLAMP: &str = "clamp";
pub const REPEAT: &str = "repeat";
pub const MIRROR: &str = "mirror";

pub const SRGB: &str = "srgb";
pub const LINEAR: &str = "linear"; // also the Sampler filter
pub const HSL: &str = "hsl";
pub const HSV: &str = "hsv";
pub const HWB: &str = "hwb";
pub const OKLAB: &str = "oklab";
pub const OKLCH: &str = "oklch";
pub const LAB: &str = "lab";
pub const LCH: &str = "lch";
pub const XYZ: &str = "xyz";

pub const MAP: &str = "map";
pub const COMBINE: &str = "combine";
pub const MIX: &str = "mix";
pub const LOOKUP: &str = "lookup";
pub const REDUCE: &str = "reduce";
pub const EXTRACT: &str = "extract";
pub const PACK: &str = "pack";
pub const GENERATE: &str = "generate";
pub const NEIGHBOR: &str = "neighbor";

pub const COUNT: &str = "count";
pub const DENSITY: &str = "density";

pub const CONSTANT: &str = "constant";
pub const SMOOTHSTEP: &str = "smoothstep";
pub const QUADRATIC: &str = "quadratic";
pub const CUBIC: &str = "cubic";
pub const INVERSE: &str = "inverse";

pub const AFFINE: &str = "affine";
pub const ABS: &str = "abs";
pub const NEGATE: &str = "negate";
pub const FLOOR: &str = "floor";
pub const SQRT: &str = "sqrt";
pub const GREATER: &str = "greater";
pub const LESS: &str = "less";
pub const GEQ: &str = "geq";
pub const LEQ: &str = "leq";
pub const EQ: &str = "eq";
pub const NEQ: &str = "neq";

pub const ADD: &str = "add";
pub const SUB: &str = "sub";
pub const MUL: &str = "mul";
pub const DIV: &str = "div";
pub const POW: &str = "pow";

pub const LENGTH: &str = "length";
pub const SUM: &str = "sum";
pub const SUMSQ: &str = "sumsq";
pub const MEAN: &str = "mean";
pub const MIN: &str = "min";
pub const MAX: &str = "max";

pub const UNIFORM: &str = "uniform";
pub const SIGNED: &str = "signed";
pub const GAUSSIAN: &str = "gaussian";

pub const NOISE: &str = "noise";
pub const TRANSFORM: &str = "transform";
pub const ATTRACT: &str = "attract";
pub const DRAG: &str = "drag";
pub const VORTEX: &str = "vortex";
pub const FORCE: &str = "force";
pub const INTEGRATE: &str = "integrate";
pub const AGE: &str = "age";
pub const IMPULSE: &str = "impulse";
pub const ORIENT: &str = "orient";
pub const FIELD: &str = "field";
pub const BOUNDS_SPHERE: &str = "bounds_sphere";
pub const BOUNDS_BOX: &str = "bounds_box";

pub const PI: f32 = std::f32::consts::PI;
pub const TWO_PI: f32 = std::f32::consts::TAU;
pub const HALF_PI: f32 = std::f32::consts::FRAC_PI_2;
pub const QUARTER_PI: f32 = std::f32::consts::FRAC_PI_4;
pub const TAU: f32 = std::f32::consts::TAU;
pub const DEG_TO_RAD: f32 = std::f32::consts::PI / 180.0;
pub const RAD_TO_DEG: f32 = 180.0 / std::f32::consts::PI;

// Key codes stay numeric (compared against `key_code`); values must match `key_code_to_u32`.

pub const KEY_A: u32 = 65;
pub const KEY_B: u32 = 66;
pub const KEY_C: u32 = 67;
pub const KEY_D: u32 = 68;
pub const KEY_E: u32 = 69;
pub const KEY_F: u32 = 70;
pub const KEY_G: u32 = 71;
pub const KEY_H: u32 = 72;
pub const KEY_I: u32 = 73;
pub const KEY_J: u32 = 74;
pub const KEY_K: u32 = 75;
pub const KEY_L: u32 = 76;
pub const KEY_M: u32 = 77;
pub const KEY_N: u32 = 78;
pub const KEY_O: u32 = 79;
pub const KEY_P: u32 = 80;
pub const KEY_Q: u32 = 81;
pub const KEY_R: u32 = 82;
pub const KEY_S: u32 = 83;
pub const KEY_T: u32 = 84;
pub const KEY_U: u32 = 85;
pub const KEY_V: u32 = 86;
pub const KEY_W: u32 = 87;
pub const KEY_X: u32 = 88;
pub const KEY_Y: u32 = 89;
pub const KEY_Z: u32 = 90;

pub const KEY_0: u32 = 48;
pub const KEY_1: u32 = 49;
pub const KEY_2: u32 = 50;
pub const KEY_3: u32 = 51;
pub const KEY_4: u32 = 52;
pub const KEY_5: u32 = 53;
pub const KEY_6: u32 = 54;
pub const KEY_7: u32 = 55;
pub const KEY_8: u32 = 56;
pub const KEY_9: u32 = 57;

pub const SPACE: u32 = 32;
pub const QUOTE: u32 = 39;
pub const COMMA: u32 = 44;
pub const MINUS: u32 = 45;
pub const PERIOD: u32 = 46;
pub const SLASH: u32 = 47;
pub const SEMICOLON: u32 = 59;
pub const EQUAL: u32 = 61;
pub const BRACKET_LEFT: u32 = 91;
pub const BACKSLASH: u32 = 92;
pub const BRACKET_RIGHT: u32 = 93;
pub const BACKQUOTE: u32 = 96;

pub const ESCAPE: u32 = 256;
pub const ENTER: u32 = 257;
pub const TAB: u32 = 258;
pub const BACKSPACE: u32 = 259;
pub const INSERT: u32 = 260;
pub const DELETE: u32 = 261;
pub const UP: u32 = 265;
pub const DOWN: u32 = 264;
pub const LEFT_ARROW: u32 = 263;
pub const RIGHT_ARROW: u32 = 262;
pub const PAGE_UP: u32 = 266;
pub const PAGE_DOWN: u32 = 267;
pub const HOME: u32 = 268;
pub const END: u32 = 269;

pub const SHIFT: u32 = 340;
pub const CONTROL: u32 = 341;
pub const ALT: u32 = 342;
pub const SUPER: u32 = 343;

pub const F1: u32 = 290;
pub const F2: u32 = 291;
pub const F3: u32 = 292;
pub const F4: u32 = 293;
pub const F5: u32 = 294;
pub const F6: u32 = 295;
pub const F7: u32 = 296;
pub const F8: u32 = 297;
pub const F9: u32 = 298;
pub const F10: u32 = 299;
pub const F11: u32 = 300;
pub const F12: u32 = 301;
