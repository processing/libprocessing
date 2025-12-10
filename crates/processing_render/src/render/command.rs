use bevy::prelude::*;

#[derive(Debug, Clone)]
pub enum DrawCommand {
    BackgroundColor(Color),
    BackgroundImage(Entity),
    Fill(Color),
    NoFill,
    StrokeColor(Color),
    NoStroke,
    StrokeWeight(f32),
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
