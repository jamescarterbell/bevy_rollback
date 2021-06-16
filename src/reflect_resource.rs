use bevy::ecs::entity::MapEntities;
use bevy::ecs::entity::EntityMap;
use bevy::ecs::entity::MapEntitiesError;
use bevy::ecs::component::Component;
use bevy::prelude::*;
use bevy::reflect::*;

#[derive(Clone)]
pub struct ReflectResource {
    add_resource: fn(&mut World, &dyn Reflect),
    apply_resource: fn(&mut World, &dyn Reflect),
    reflect_resource: fn(&World) -> Option<&dyn Reflect>,
    copy_resource: fn(&World, &mut World),
}

impl ReflectResource {
    pub fn add_resource(&self, world: &mut World, resource: &dyn Reflect) {
        (self.add_resource)(world, resource);
    }

    pub fn apply_resource(&self, world: &mut World, resource: &dyn Reflect) {
        (self.apply_resource)(world, resource);
    }

    pub fn reflect_resource<'a>(
        &self,
        world: &'a World
    ) -> Option<&'a dyn Reflect> {
        (self.reflect_resource)(world)
    }

    pub fn copy_resource(
        &self,
        source_world: &World,
        destination_world: &mut World
    ) {
        (self.copy_resource)(
            source_world,
            destination_world,
        );
    }
}

impl<C: Component + Reflect + FromWorld> FromType<C> for ReflectResource {
    fn from_type() -> Self {
        ReflectResource {
            add_resource: |world, reflected_resource| {
                let mut resource = C::from_world(world);
                resource.apply(reflected_resource);
                world.insert_resource(resource);
            },
            apply_resource: |world, reflected_resource| {
                let mut resource = world.get_resource_mut::<C>().unwrap();
                resource.apply(reflected_resource);
            },
            copy_resource: |source_world, destination_world| {
                let source_resource = source_world.get_resource::<C>().unwrap();
                let mut destination_resource = C::from_world(destination_world);
                destination_resource.apply(source_resource);
                destination_world
                    .insert_resource(destination_resource);
            },
            reflect_resource: |world| {
                world
                    .get_resource::<C>()
                    .map(|c| c as &dyn Reflect)
            },
        }
    }
}


#[derive(Clone)]
pub struct ReflectMapEntitiesResources {
    map_entities: fn(&mut World, &EntityMap) -> Result<(), MapEntitiesError>,
}

impl ReflectMapEntitiesResources {
    pub fn map_entities(
        &self,
        world: &mut World,
        entity_map: &EntityMap,
    ) -> Result<(), MapEntitiesError> {
        (self.map_entities)(world, entity_map)
    }
}

impl<C: Component + MapEntities> FromType<C> for ReflectMapEntitiesResources {
    fn from_type() -> Self {
        ReflectMapEntitiesResources {
            map_entities: |world, entity_map| {
                if let Some(mut resource) = world.get_resource_mut::<C>(){
                    resource.map_entities(entity_map)?;
                }
                Ok(())
            },
        }
    }
}