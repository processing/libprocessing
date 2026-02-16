use bevy::prelude::*;

#[derive(Debug, Default, Component)]
pub struct CommandBuffer<T> {
    pub commands: Vec<T>,
}

impl<T> CommandBuffer<T> {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn push(&mut self, cmd: T) {
        self.commands.push(cmd);
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }
}
