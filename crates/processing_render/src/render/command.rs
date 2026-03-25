use bevy::prelude::*;

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
