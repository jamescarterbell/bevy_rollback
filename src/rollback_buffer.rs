use crate::rollback_registry::RollbackRegistry;
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
    overrides: HashMap<isize, SystemStage>,
    current_frame: usize,
    rollback_needed: isize,
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
    pub fn push_world(&mut self, index: &usize, world: &World, registry: &RollbackRegistry) -> std::result::Result<Option<World>, RollbackError>{
        let len = self.buffer.len();
        let old_world = self
            .buffer
            .get_mut(index % len)
            .unwrap()
            .replace(clone_world(world, &registry)?);

        if let Some(_) = old_world{
            self.overrides.remove(&(*index as isize));
        };

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

    pub fn add_overrides_relative(&mut self, index: &isize, overrides: impl System<In = (), Out = ()>){
        self
            .overrides
            .entry(self.current_frame as isize - *index)
            .or_insert(SystemStage::parallel())
            .add_system(overrides);
        if *index > self.rollback_needed as isize{
            self.rollback_needed = *index;
        }
    }

    pub fn remove_override(&mut self, index: &isize) -> Option<SystemStage>{
        self.overrides.remove(&index)
    }

    pub fn get_override_mut(&mut self, index: &isize) -> Option<&mut SystemStage>{
        self.overrides.get_mut(&index)
    }

    pub fn current_frame(&self) -> usize{
        self.current_frame
    }

    pub fn rollback_needed(&self) -> isize{
        self.rollback_needed
    }

    pub(crate) fn inc_frame(&mut self){
        self.current_frame += 1;
    }

    pub(crate) fn reset_rollback_needed(&mut self){
        self.rollback_needed = 0;
    }
}