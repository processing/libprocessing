use bevy::prelude::*;
use bevy::render::render_resource::{BlendComponent, BlendFactor, BlendOperation, BlendState};
use processing_core::error::{self, ProcessingError};

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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum ArcMode {
    #[default]
    Open = 0,
    Chord = 1,
    Pie = 2,
}

impl From<u8> for ArcMode {
    fn from(v: u8) -> Self {
        match v {
            0 => Self::Open,
            1 => Self::Chord,
            2 => Self::Pie,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum ShapeMode {
    #[default]
    Corner = 0,
    Corners = 1,
    Center = 2,
    Radius = 3,
}

impl From<u8> for ShapeMode {
    fn from(v: u8) -> Self {
        match v {
            0 => Self::Corner,
            1 => Self::Corners,
            2 => Self::Center,
            3 => Self::Radius,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum ShapeKind {
    #[default]
    Polygon = 0,
    Points = 1,
    Lines = 2,
    Triangles = 3,
    TriangleFan = 4,
    TriangleStrip = 5,
    Quads = 6,
    QuadStrip = 7,
}

impl From<u8> for ShapeKind {
    fn from(v: u8) -> Self {
        match v {
            0 => Self::Polygon,
            1 => Self::Points,
            2 => Self::Lines,
            3 => Self::Triangles,
            4 => Self::TriangleFan,
            5 => Self::TriangleStrip,
            6 => Self::Quads,
            7 => Self::QuadStrip,
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

impl TryFrom<u8> for BlendMode {
    type Error = ProcessingError;

    fn try_from(v: u8) -> std::result::Result<Self, Self::Error> {
        match v {
            0 => Ok(Self::Blend),
            1 => Ok(Self::Add),
            2 => Ok(Self::Subtract),
            3 => Ok(Self::Darkest),
            4 => Ok(Self::Lightest),
            5 => Ok(Self::Difference),
            6 => Ok(Self::Exclusion),
            7 => Ok(Self::Multiply),
            8 => Ok(Self::Screen),
            9 => Ok(Self::Replace),
            _ => Err(ProcessingError::InvalidArgument(format!(
                "unknown blend mode: {v}"
            ))),
        }
    }
}

fn blend_factor_from_u8(v: u8) -> std::result::Result<BlendFactor, ProcessingError> {
    match v {
        0 => Ok(BlendFactor::Zero),
        1 => Ok(BlendFactor::One),
        2 => Ok(BlendFactor::Src),
        3 => Ok(BlendFactor::OneMinusSrc),
        4 => Ok(BlendFactor::SrcAlpha),
        5 => Ok(BlendFactor::OneMinusSrcAlpha),
        6 => Ok(BlendFactor::Dst),
        7 => Ok(BlendFactor::OneMinusDst),
        8 => Ok(BlendFactor::DstAlpha),
        9 => Ok(BlendFactor::OneMinusDstAlpha),
        10 => Ok(BlendFactor::SrcAlphaSaturated),
        _ => Err(ProcessingError::InvalidArgument(format!(
            "unknown blend factor: {v}"
        ))),
    }
}

fn blend_op_from_u8(v: u8) -> std::result::Result<BlendOperation, ProcessingError> {
    match v {
        0 => Ok(BlendOperation::Add),
        1 => Ok(BlendOperation::Subtract),
        2 => Ok(BlendOperation::ReverseSubtract),
        3 => Ok(BlendOperation::Min),
        4 => Ok(BlendOperation::Max),
        _ => Err(ProcessingError::InvalidArgument(format!(
            "unknown blend operation: {v}"
        ))),
    }
}

pub fn custom_blend_state(
    color_src: u8,
    color_dst: u8,
    color_op: u8,
    alpha_src: u8,
    alpha_dst: u8,
    alpha_op: u8,
) -> error::Result<BlendState> {
    Ok(BlendState {
        color: BlendComponent {
            src_factor: blend_factor_from_u8(color_src)?,
            dst_factor: blend_factor_from_u8(color_dst)?,
            operation: blend_op_from_u8(color_op)?,
        },
        alpha: BlendComponent {
            src_factor: blend_factor_from_u8(alpha_src)?,
            dst_factor: blend_factor_from_u8(alpha_dst)?,
            operation: blend_op_from_u8(alpha_op)?,
        },
    })
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
            Self::Darkest => Some(BlendState {
                color: color(One, One, Min),
                alpha: ALPHA_ADDITIVE,
            }),
            Self::Lightest => Some(BlendState {
                color: color(One, One, Max),
                alpha: ALPHA_ADDITIVE,
            }),
            // TODO: this is an approximation as we can't do abs difference in fixed function
            // blending. this should probs be a fullscreen post-process effect instead. if we
            // choose to add shader based blending, we should also consider adding more
            // blend modes
            //
            // alternatively, we could express these shader based blend modes via a generic
            // composite filter, which would more accurately reflect what is actually happening
            Self::Difference => Some(BlendState {
                color: color(One, One, ReverseSubtract),
                alpha: BlendComponent {
                    src_factor: One,
                    dst_factor: One,
                    operation: Max,
                },
            }),
            Self::Exclusion => Some(BlendState {
                color: color(OneMinusDst, OneMinusSrc, Add),
                alpha: BlendComponent {
                    src_factor: One,
                    dst_factor: OneMinusSrcAlpha,
                    operation: Add,
                },
            }),
            Self::Multiply => Some(BlendState {
                color: color(Dst, OneMinusSrcAlpha, Add),
                alpha: ALPHA_ADDITIVE,
            }),
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
    RectMode(ShapeMode),
    EllipseMode(ShapeMode),
    Rect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        radii: [f32; 4], // [tl, tr, br, bl]
    },
    Ellipse {
        cx: f32,
        cy: f32,
        w: f32,
        h: f32,
    },
    Line {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    },
    Triangle {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
    },
    Quad {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        x4: f32,
        y4: f32,
    },
    Point {
        x: f32,
        y: f32,
    },
    Arc {
        cx: f32,
        cy: f32,
        w: f32,
        h: f32,
        start: f32,
        stop: f32,
        mode: ArcMode,
    },
    Bezier {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        x4: f32,
        y4: f32,
    },
    Curve {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        x4: f32,
        y4: f32,
    },
    BeginShape {
        kind: ShapeKind,
    },
    EndShape {
        close: bool,
    },
    ShapeVertex {
        x: f32,
        y: f32,
    },
    ShapeBezierVertex {
        cx1: f32,
        cy1: f32,
        cx2: f32,
        cy2: f32,
        x: f32,
        y: f32,
    },
    ShapeQuadraticVertex {
        cx: f32,
        cy: f32,
        x: f32,
        y: f32,
    },
    ShapeCurveVertex {
        x: f32,
        y: f32,
    },
    BeginContour,
    EndContour,
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
    Cylinder {
        radius: f32,
        height: f32,
        detail: u32,
    },
    Cone {
        radius: f32,
        height: f32,
        detail: u32,
    },
    Torus {
        radius: f32,
        tube_radius: f32,
        major_segments: u32,
        minor_segments: u32,
    },
    Plane {
        width: f32,
        height: f32,
    },
    Capsule {
        radius: f32,
        length: f32,
        detail: u32,
    },
    ConicalFrustum {
        radius_top: f32,
        radius_bottom: f32,
        height: f32,
        detail: u32,
    },
    Tetrahedron {
        radius: f32,
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
