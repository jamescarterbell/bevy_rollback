use bevy::prelude::*;
use std::ops::{Deref, DerefMut};

pub mod rollback_registry;
pub mod util;
pub mod err;
pub mod reflect_resource;
pub mod rollback_buffer;
pub mod rollback_schedule;
pub mod system;

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
    use bevy::tasks::ComputeTaskPool;
use bevy::prelude::*;
    use bevy::scene::{
        DynamicScene,
        serde::*,
    };
    use bevy::ecs::entity::EntityMap;
    use bevy::reflect::*;
    use ::serde::*;

    use crate::rollback_registry::RollbackRegistry;
    use crate::util::*;
    use crate::RollbackWorld;
    use crate::system::rollback_system;
    use crate::rollback_schedule::RollbackSchedule;
    use crate::rollback_buffer::RollbackBuffer;

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

        clone_rollback_world_entities(&world, &mut other_world, &mut EntityMap::default(), &registry).unwrap();
        clone_rollback_world_resources(&world, &mut other_world, &mut EntityMap::default(), &registry).unwrap();

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
            rollback_buffer.push_world(&i, &world, &registry);
            rollback_buffer.inc_frame();
        }

        for i in 0..1000{
            if let Some(world) = rollback_buffer.get_world_mut(i){
                assert_eq!((0..=i).sum::<usize>(), world.query::<&usize>().iter(world).sum::<usize>());
            }
        }
    }

    #[test]
    fn inc_test(){
        let mut world = RollbackWorld::default();
        let mut rollback_buffer = RollbackBuffer::with_capacity(101);
        let mut rollback_schedule = RollbackSchedule::default();
        let mut registry = RollbackRegistry::default();

        registry.register::<Incer>();

        world.insert_resource(0isize);
        world.insert_resource(Incer{inc: 1});

        let system = Box::new(|mut current: ResMut<isize>, inc: Res<Incer>|{
            println!("{} - {}", *current, inc.inc);
            *current += inc.inc;
        });
        rollback_schedule.add_stage("test", SystemStage::parallel());
        rollback_schedule.add_system_to_stage("test", system.system());

        let mut larger_world = World::default();

        larger_world.insert_resource(world);
        larger_world.insert_resource(rollback_buffer);
        larger_world.insert_resource(rollback_schedule);
        larger_world.insert_resource(registry);


        let mut helper_stage = SystemStage::single_threaded();
        helper_stage.add_system(rollback_system.system());

        for i in 0..100{
            helper_stage.run(&mut larger_world);
        }

        assert_eq!(100, *larger_world.get_resource::<RollbackWorld>().unwrap().get_resource::<isize>().unwrap());

        larger_world.get_resource_mut::<RollbackBuffer>().unwrap().add_overrides(&0, Box::new(|mut incer: ResMut<Incer>|{
            incer.inc = -1;
        }).system());

        helper_stage.run(&mut larger_world);
        assert_eq!(-101, *larger_world.get_resource::<RollbackWorld>().unwrap().get_resource::<isize>().unwrap());
    }

    #[derive(Default, Reflect, Serialize)]
    #[reflect(Component)]
    struct Incer{
        inc: isize,
    }
}
