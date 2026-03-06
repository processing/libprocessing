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
    Translate {
        x: f32,
        y: f32,
    },
    Rotate {
        angle: f32,
    },
    Scale {
        x: f32,
        y: f32,
    },
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
