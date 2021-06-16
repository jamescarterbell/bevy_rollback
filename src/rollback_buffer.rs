use std::collections::HashMap;
use crate::err::RollbackError;
use crate::util::clone_world;
use std::collections::VecDeque;
use bevy::prelude::*;
use bevy::scene::{
    DynamicScene,
    serde::*,
};
use bevy::reflect::*;
use ron::de::*;

pub struct RollbackBuffer{
    buffer: Vec<Option<World>>,
    overrides: HashMap<usize, SystemStage>,
    current_frame: usize,
    rollback_needed: usize,
}

impl RollbackBuffer{
    pub fn with_capacity(capacity: usize) -> RollbackBuffer{
        let mut buf = RollbackBuffer{
            buffer: Vec::with_capacity(capacity),
            overrides: HashMap::default(),
            current_frame: 0,
            rollback_needed: 0,
        };
        for _ in 0..capacity{
            buf.buffer.push(None);
        }
        buf
    }

    /// Pushes a new world to the buffer, returns the serialized world that needs to be popped
    /// from the buffer.
    pub fn push_world(&mut self, world: &World, registry: &TypeRegistry) -> std::result::Result<Option<World>, RollbackError>{
        let len = self.buffer.len();
        let old_world = self
            .buffer
            .get_mut(self.current_frame % len)
            .unwrap()
            .replace(clone_world(world, &registry)?);
        self.current_frame += 1;

        Ok(old_world)
    }

    /// Gets the world at the given index without changing the internal buffer.
    pub fn get_world(&self, index: usize) -> Option<&World>{
        if self.current_frame - index > self.buffer.len(){
            return None;
        }
        self
            .buffer
            .get(index % self.buffer.len())
            .unwrap()
            .as_ref()
    }

    pub fn get_world_mut(&mut self, index: usize) -> Option<&mut World>{
        let len = self.buffer.len();
        if self.current_frame - index > len{
            return None;
        }
        self
            .buffer
            .get_mut(index % len)
            .unwrap()
            .as_mut()
    }

    pub fn add_overrides(&mut self, index: &usize, overrides: impl System<In = (), Out = ()>){
        self
            .overrides
            .entry(*index)
            .or_insert(SystemStage::parallel())
            .add_system(overrides);
        let relative_frame = self.current_frame - index;
        if relative_frame < self.rollback_needed{
            self.rollback_needed = relative_frame;
        }
    }

    pub fn get_override(&mut self, index: &usize) -> Option<&mut SystemStage>{
        self.overrides.get_mut(&index)
    }
}