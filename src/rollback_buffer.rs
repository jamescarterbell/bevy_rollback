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
    buffer: VecDeque<World>,
    overrides: HashMap<usize, SystemStage>,
    count: usize,
}

impl RollbackBuffer{
    pub fn with_capacity(capacity: usize) -> RollbackBuffer{
        RollbackBuffer{
            buffer: VecDeque::with_capacity(capacity),
            overrides: HashMap::default(),
            count: 0,
        }
    }

    /// Pushes a new world to the buffer, returns the serialized world that needs to be popped
    /// from the buffer.
    pub fn push_world(&mut self, world: &World, registry: &TypeRegistry) -> std::result::Result<Option<World>, RollbackError>{
        let throw_away = match self.buffer.len() == self.buffer.capacity(){
            true => {
                let mut throw_away = self.buffer.pop_front().unwrap();
                if let Some(mut stage) = self.overrides.remove(&self.count){
                    stage.run(&mut throw_away);
                }
                self.count += 1;
                Some(throw_away)
            },
            false => None,
        };

        self
            .buffer
            .push_back(clone_world(world, &registry)?);

        Ok(throw_away)
    }

    /// Gets the world at the given index without changing the internal buffer.
    pub fn get_world(&self, index: usize) -> Option<&World>{
        if index < self.count{
            return None;
        }
        self
            .buffer
            .get(index - self.count)
    }

    pub fn get_world_mut(&mut self, index: usize) -> Option<&mut World>{
        if index < self.count{
            return None;
        }
        self
            .buffer
            .get_mut(index - self.count)
    }

    /// Gets the world at the given index and drains all worlds after it from the buffer.
    pub fn pop_world(&mut self, index: usize) -> Option<World>{
        if index < self.count{
            return None;
        }
        self
            .buffer
            .drain((index - self.count)..)
            .nth(0)
    }

    pub fn add_overrides(&mut self, index: &usize, overrides: impl System<In = (), Out = ()>){
        if let Some(stage) = self.overrides.get_mut(&index){
            stage.add_system(overrides);
        }
    }

    pub fn get_override(&mut self, index: &usize) -> Option<&mut SystemStage>{
        self.overrides.get_mut(&index)
    }
}