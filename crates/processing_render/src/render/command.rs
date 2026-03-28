use bevy::prelude::*;
use bevy::render::render_resource::{BlendComponent, BlendFactor, BlendOperation, BlendState};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum StrokeCapMode {
    #[default]
    Round = 0,
    Square = 1,
    Project = 2,
}

impl From<u8> for StrokeCapMode {
    fn from(v: u8) -> Self {
        match v {
            0 => Self::Round,
            1 => Self::Square,
            2 => Self::Project,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum StrokeJoinMode {
    #[default]
    Round = 0,
    Miter = 1,
    Bevel = 2,
}

impl From<u8> for StrokeJoinMode {
    fn from(v: u8) -> Self {
        match v {
            0 => Self::Round,
            1 => Self::Miter,
            2 => Self::Bevel,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum BlendMode {
    #[default]
    Blend = 0,
    Add = 1,
    Subtract = 2,
    Darkest = 3,
    Lightest = 4,
    Difference = 5,
    Exclusion = 6,
    Multiply = 7,
    Screen = 8,
    Replace = 9,
}

impl From<u8> for BlendMode {
    fn from(v: u8) -> Self {
        match v {
            0 => Self::Blend,
            1 => Self::Add,
            2 => Self::Subtract,
            3 => Self::Darkest,
            4 => Self::Lightest,
            5 => Self::Difference,
            6 => Self::Exclusion,
            7 => Self::Multiply,
            8 => Self::Screen,
            9 => Self::Replace,
            _ => Self::default(),
        }
    }
}

pub fn blend_factor_from_u8(v: u8) -> BlendFactor {
    match v {
        0 => BlendFactor::Zero,
        1 => BlendFactor::One,
        2 => BlendFactor::Src,
        3 => BlendFactor::OneMinusSrc,
        4 => BlendFactor::SrcAlpha,
        5 => BlendFactor::OneMinusSrcAlpha,
        6 => BlendFactor::Dst,
        7 => BlendFactor::OneMinusDst,
        8 => BlendFactor::DstAlpha,
        9 => BlendFactor::OneMinusDstAlpha,
        10 => BlendFactor::SrcAlphaSaturated,
        _ => BlendFactor::One,
    }
}

pub fn blend_op_from_u8(v: u8) -> BlendOperation {
    match v {
        0 => BlendOperation::Add,
        1 => BlendOperation::Subtract,
        2 => BlendOperation::ReverseSubtract,
        3 => BlendOperation::Min,
        4 => BlendOperation::Max,
        _ => BlendOperation::Add,
    }
}

pub fn custom_blend_state(
    color_src: u8,
    color_dst: u8,
    color_op: u8,
    alpha_src: u8,
    alpha_dst: u8,
    alpha_op: u8,
) -> BlendState {
    BlendState {
        color: BlendComponent {
            src_factor: blend_factor_from_u8(color_src),
            dst_factor: blend_factor_from_u8(color_dst),
            operation: blend_op_from_u8(color_op),
        },
        alpha: BlendComponent {
            src_factor: blend_factor_from_u8(alpha_src),
            dst_factor: blend_factor_from_u8(alpha_dst),
            operation: blend_op_from_u8(alpha_op),
        },
    }
}

const ALPHA_ADDITIVE: BlendComponent = BlendComponent {
    src_factor: BlendFactor::One,
    dst_factor: BlendFactor::One,
    operation: BlendOperation::Add,
};

impl BlendMode {
    pub fn name(self) -> &'static str {
        match self {
            Self::Blend => "BLEND",
            Self::Add => "ADD",
            Self::Subtract => "SUBTRACT",
            Self::Darkest => "DARKEST",
            Self::Lightest => "LIGHTEST",
            Self::Difference => "DIFFERENCE",
            Self::Exclusion => "EXCLUSION",
            Self::Multiply => "MULTIPLY",
            Self::Screen => "SCREEN",
            Self::Replace => "REPLACE",
        }
    }

    /// Returns None for the default Blend mode, letting AlphaMode handle blending.
    pub fn to_blend_state(self) -> Option<BlendState> {
        use BlendFactor::*;
        use BlendOperation::*;

        let color = |src_factor, dst_factor, operation| BlendComponent {
            src_factor,
            dst_factor,
            operation,
        };

        match self {
            Self::Blend => None,
            Self::Add => Some(BlendState {
                color: color(SrcAlpha, One, Add),
                alpha: ALPHA_ADDITIVE,
            }),
            Self::Subtract => Some(BlendState {
                color: color(SrcAlpha, One, ReverseSubtract),
                alpha: ALPHA_ADDITIVE,
            }),
            // Blend factors are ignored by Min/Max operations
            Self::Darkest => Some(BlendState {
                color: color(One, One, Min),
                alpha: ALPHA_ADDITIVE,
            }),
            Self::Lightest => Some(BlendState {
                color: color(One, One, Max),
                alpha: ALPHA_ADDITIVE,
            }),
            // |src - dst| — not representable with fixed-function blending;
            // reverse subtract is the same approximation Processing's OpenGL renderer uses.
            Self::Difference => Some(BlendState {
                color: color(One, One, ReverseSubtract),
                alpha: BlendComponent {
                    src_factor: One,
                    dst_factor: One,
                    operation: Max,
                },
            }),
            // src + dst - 2*src*dst = (1-dst)*src + (1-src)*dst
            Self::Exclusion => Some(BlendState {
                color: color(OneMinusDst, OneMinusSrc, Add),
                alpha: BlendComponent {
                    src_factor: One,
                    dst_factor: OneMinusSrcAlpha,
                    operation: Add,
                },
            }),
            // src * dst (alpha-aware: falls back to dst when src is transparent)
            Self::Multiply => Some(BlendState {
                color: color(Dst, OneMinusSrcAlpha, Add),
                alpha: ALPHA_ADDITIVE,
            }),
            // src + dst - src*dst = (1-dst)*src + dst
            Self::Screen => Some(BlendState {
                color: color(OneMinusDst, One, Add),
                alpha: ALPHA_ADDITIVE,
            }),
            Self::Replace => Some(BlendState::REPLACE),
        }
    }
}

#[derive(Debug, Clone)]
pub enum DrawCommand {
    BackgroundColor(Color),
    BackgroundImage(Entity),
    Fill(Color),
    NoFill,
    StrokeColor(Color),
    NoStroke,
    StrokeWeight(f32),
    StrokeCap(StrokeCapMode),
    StrokeJoin(StrokeJoinMode),
    Roughness(f32),
    Metallic(f32),
    Emissive(Color),
    Unlit,
    Rect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        radii: [f32; 4], // [tl, tr, br, bl]
    },
    PushMatrix,
    PopMatrix,
    ResetMatrix,
    Translate(Vec2),
    Rotate {
        angle: f32,
    },
    Scale(Vec2),
    ShearX {
        angle: f32,
    },
    ShearY {
        angle: f32,
    },
    Geometry(Entity),
    BlendMode(Option<BlendState>),
    Material(Entity),
    Box {
        width: f32,
        height: f32,
        depth: f32,
    },
    Sphere {
        radius: f32,
        sectors: u32,
        stacks: u32,
    },
}

#[derive(Debug, Default, Component)]
pub struct CommandBuffer {
    pub commands: Vec<DrawCommand>,
}

impl CommandBuffer {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn push(&mut self, cmd: DrawCommand) {
        self.commands.push(cmd);
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }
}
