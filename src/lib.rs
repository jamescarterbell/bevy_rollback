use bevy::prelude::*;
use std::ops::{Deref, DerefMut};

pub mod rollback_registry;
pub mod util;
pub mod err;
pub mod reflect_resource;
pub mod rollback_buffer;

pub struct RollbackWorld{
    world: World,
}

impl Default for RollbackWorld{
    fn default() -> Self{
        RollbackWorld{
            world: World::default(),
        }
    }
}

impl Deref for RollbackWorld{
    type Target = World;

    fn deref(&self) -> &Self::Target{
        &self.world
    }
}

impl DerefMut for RollbackWorld{
    fn deref_mut(&mut self) -> &mut Self::Target{
        &mut self.world
    }
}

#[cfg(test)]
mod tests {
    use crate::rollback_buffer::RollbackBuffer;
use bevy::prelude::*;
    use bevy::scene::{
        DynamicScene,
        serde::*,
    };
    use bevy::ecs::entity::EntityMap;
    use bevy::reflect::*;

    use crate::rollback_registry::RollbackRegistry;
    use crate::util::*;
    use crate::RollbackWorld;

    #[test]
    fn resource_clone() {
        let mut world = RollbackWorld::default();
        world
            .spawn()
            .insert(10usize)
            .insert(20isize)
            .insert("High".to_string());
        
        world.insert_resource(10usize);
        world.insert_resource(-10isize);

        let mut other_world = RollbackWorld::default();
        
        let registry = RollbackRegistry::default();

        clone_rollback_world_entities(&world, &mut other_world, &mut EntityMap::default(), &registry.registry).unwrap();
        clone_rollback_world_resources(&world, &mut other_world, &mut EntityMap::default(), &registry.registry).unwrap();

        assert_eq!(-10, *other_world.get_resource::<isize>().unwrap());
    }

    #[test]
    fn sum_test(){
        let mut world = RollbackWorld::default();
        let mut rollback_buffer = RollbackBuffer::with_capacity(100);
        let registry = RollbackRegistry::default();
        
        for i in 0..1000{
            world
                .spawn()
                .insert(i as usize);
            rollback_buffer.push_world(&world, &registry.registry);
        }

        for i in 0..1000{
            if let Some(world) = rollback_buffer.get_world_mut(i){
                assert_eq!((0..=i).sum::<usize>(), world.query::<&usize>().iter(world).sum::<usize>());
            }
        }
    }
}
