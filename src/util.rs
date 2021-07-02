use bevy::ecs::schedule::Stage;
use crate::RollbackStartupSchedule;
use crate::rollback_schedule::RollbackSchedule;
use bevy::app::AppBuilder;
use bevy::ecs::schedule::{StageLabel, SystemDescriptor};
use bevy::reflect::TypeRegistry;
use bevy::ecs::reflect::ReflectMapEntities;
use crate::reflect_resource::ReflectResource;
use crate::rollback_registry::RollbackRegistry;
use crate::RollbackWorld;
use crate::err::RollbackError;

use bevy::{
    ecs::reflect::{ReflectComponent, ReflectMut},
    scene::{DynamicScene, Entity},
    reflect::{Reflect, FromType},
    ecs::world::{World, FromWorld},
    ecs::component::Component,
    ecs::entity::EntityMap,
};

pub fn clone_rollback_world_entities(source_world: &World, target_world: &mut World, entity_map: &mut EntityMap, registry: &RollbackRegistry) -> Result<(), RollbackError>{
    let type_registry = registry.registry.read();
    for archetype in source_world.archetypes().iter() {
        for entity in archetype.entities() {
            entity_map
                .entry(entity.clone())
                .or_insert_with(|| target_world.spawn().id());
        }

        for component_id in archetype.components() {
            let reflect_component = source_world
                .components()
                .get_info(component_id)
                .and_then(|info| type_registry.get(info.type_id().unwrap()))
                .and_then(|registration| registration.data::<ReflectComponent>())
                .ok_or(
                    source_world
                        .components()
                        .get_info(component_id)
                        .unwrap()
                        .type_id()
                        .unwrap());
            
            match reflect_component{
                Ok(reflect_component) => for (i, entity) in archetype.entities().iter().enumerate() {
                        reflect_component
                            .copy_component(
                                source_world,
                                target_world,
                                entity.clone(),
                                entity_map.get(entity.clone()).unwrap(),
                            )
                    },
                Err(id) => {
                    if let None = registry.unregisterable.get(&id){
                        return Err(RollbackError::UnregisteredType(source_world
                            .components()
                            .get_info(component_id)
                            .unwrap()
                            .name()
                            .to_owned()));
                    }
                }
            };
        }
    }

    for registration in type_registry.iter() {
        if let Some(map_entities_reflect) = registration.data::<ReflectMapEntities>() {
            map_entities_reflect
                .map_entities(target_world, &entity_map)
                .unwrap();
        }
    }

    Ok(())
}

pub fn clone_rollback_world_resources(source_world: &World, target_world: &mut World, entity_map: &mut EntityMap, registry: &RollbackRegistry) -> Result<(), RollbackError>{
    let type_registry = registry.registry.read();
    let archetype = source_world.archetypes().resource();

    for component_id in archetype.unique_components().indices(){
        let reflect_resource = source_world
            .components()
            .get_info(component_id)
            .and_then(|info| type_registry.get(info.type_id().unwrap()))
            .and_then(|registration| registration.data::<ReflectResource>())
            .ok_or(
                source_world
                    .components()
                    .get_info(component_id)
                    .unwrap()
                    .type_id()
                    .unwrap());
        match reflect_resource{
            Ok(reflect_resource) =>{
                reflect_resource.copy_resource(
                    source_world,
                    target_world,
                )
            }
            Err(id) => {
                if let None = registry.unregisterable.get(&id){
                    return Err(RollbackError::UnregisteredType(source_world
                        .components()
                        .get_info(component_id)
                        .unwrap()
                        .name()
                        .to_owned()));
                }
            }
        }
    }

    for registration in type_registry.iter() {
        if let Some(map_entities_reflect) = registration.data::<ReflectMapEntities>() {
            map_entities_reflect
                .map_entities(target_world, &entity_map)
                .unwrap();
        }
    }
    return Ok(());
}

pub fn clone_world(source_world: &World, registry: &RollbackRegistry) -> Result<World, RollbackError>{
    let mut target_world = World::default();
    let mut entity_map = EntityMap::default();
    clone_rollback_world_entities(source_world, &mut target_world, &mut entity_map, &registry)?;
    clone_rollback_world_resources(source_world, &mut target_world, &mut entity_map, &registry)?;
    Ok(target_world)
}

pub fn clear_entities(world: &mut World){
    let mut removals = Vec::new();
    for entity in world.query::<bevy::ecs::entity::Entity>().iter(world){
        removals.push(entity);
    }
    for entity in removals{
        world.despawn(entity);
    }
}

pub fn clear_resources(world: &mut World, registry: &RollbackRegistry) -> Result<(), RollbackError>{
    let type_registry = registry.registry.read();
    let archetype = world.archetypes().resource();

    let mut removals = Vec::new();

    for component_id in archetype.unique_components().indices(){
        let reflect_resource = world
            .components()
            .get_info(component_id)
            .and_then(|info| type_registry.get(info.type_id().unwrap()))
            .and_then(|registration| registration.data::<ReflectResource>())
            .ok_or(
                world
                    .components()
                    .get_info(component_id)
                    .unwrap()
                    .type_id()
                    .unwrap());
        match reflect_resource{
            Ok(reflect_resource) =>{
                removals.push(reflect_resource);
            }
            Err(id) => {
                if let None = registry.unregisterable.get(&id){
                    return Err(RollbackError::UnregisteredType(world
                        .components()
                        .get_info(component_id)
                        .unwrap()
                        .name()
                        .to_owned()));
                }
            }
        }
    }

    for reflect_resource in removals{
        reflect_resource.remove_resource(
            world,
        )
    }
    return Ok(());
}

pub fn clear_world(world: &mut World, registry: &RollbackRegistry) -> Result<(), RollbackError>{
    clear_entities(world);
    clear_resources(world, registry)
}

pub fn overwrite_world(source_world: &World, target_world: &mut World, registry: &RollbackRegistry) -> Result<(), RollbackError>{
    clear_world(target_world, registry)?;
    let mut entity_map = EntityMap::default();
    clone_rollback_world_entities(source_world, target_world, &mut entity_map, &registry)?;
    clone_rollback_world_resources(source_world, target_world, &mut entity_map, &registry)?;
    Ok(())
}

pub trait AppBuilderRollbackUtil{
    fn add_rollback_startup_stage<S: Stage>(
        &mut self,
        label: impl StageLabel,
        stage: S
    ) -> &mut AppBuilder;

    fn add_rollback_startup_stage_after<S: Stage>(
        &mut self,
        target: impl StageLabel,
        label: impl StageLabel,
        stage: S
    ) -> &mut AppBuilder;

    fn add_rollback_startup_stage_before<S: Stage>(
        &mut self,
        target: impl StageLabel,
        label: impl StageLabel,
        stage: S
    ) -> &mut AppBuilder;

    fn add_rollback_startup_system_to_stage(
        &mut self,
        label: impl StageLabel,
        system: impl Into<SystemDescriptor>
    ) -> &mut AppBuilder;

    fn add_rollback_stage<S: Stage>(
        &mut self,
        label: impl StageLabel,
        stage: S
    ) -> &mut AppBuilder;

    fn add_rollback_stage_after<S: Stage>(
        &mut self,
        target: impl StageLabel,
        label: impl StageLabel,
        stage: S
    ) -> &mut AppBuilder;

    fn add_rollback_stage_before<S: Stage>(
        &mut self,
        target: impl StageLabel,
        label: impl StageLabel,
        stage: S
    ) -> &mut AppBuilder;

    fn add_rollback_system_to_stage(
        &mut self,
        label: impl StageLabel,
        system: impl Into<SystemDescriptor>
    ) -> &mut AppBuilder;
}

impl AppBuilderRollbackUtil for AppBuilder{
    fn add_rollback_startup_stage<S: Stage>(
        &mut self,
        label: impl StageLabel,
        stage: S
    ) -> &mut AppBuilder {
        self
            .world_mut()
            .get_resource_mut::<RollbackStartupSchedule>()
            .expect("Add RollbackStartupSchedule to app!")
            .add_stage(label, stage);

        self
    }

    fn add_rollback_startup_stage_after<S: Stage>(
        &mut self,
        target: impl StageLabel,
        label: impl StageLabel,
        stage: S
    ) -> &mut AppBuilder {
        self
            .world_mut()
            .get_resource_mut::<RollbackStartupSchedule>()
            .expect("Add RollbackStartupSchedule to app!")
            .add_stage_after(target, label, stage);

        self
    }

    fn add_rollback_startup_stage_before<S: Stage>(
        &mut self,
        target: impl StageLabel,
        label: impl StageLabel,
        stage: S
    ) -> &mut AppBuilder {
        self
            .world_mut()
            .get_resource_mut::<RollbackStartupSchedule>()
            .expect("Add RollbackStartupSchedule to app!")
            .add_stage_before(target, label, stage);

        self
    }

    fn add_rollback_startup_system_to_stage(
        &mut self,
        label: impl StageLabel,
        system: impl Into<SystemDescriptor>
    ) -> &mut AppBuilder {
        self
            .world_mut()
            .get_resource_mut::<RollbackStartupSchedule>()
            .expect("Add RollbackStartupSchedule to app!")
            .add_system_to_stage(label, system);

        self
    }

    fn add_rollback_stage<S: Stage>(
        &mut self,
        label: impl StageLabel,
        stage: S
    ) -> &mut AppBuilder {
        self
            .world_mut()
            .get_resource_mut::<RollbackSchedule>()
            .expect("Add RollbackSchedule to app!")
            .add_stage(label, stage);

        self
    }

    fn add_rollback_stage_after<S: Stage>(
        &mut self,
        target: impl StageLabel,
        label: impl StageLabel,
        stage: S
    ) -> &mut AppBuilder {
        self
            .world_mut()
            .get_resource_mut::<RollbackSchedule>()
            .expect("Add RollbackSchedule to app!")
            .add_stage_after(target, label, stage);

        self
    }

    fn add_rollback_stage_before<S: Stage>(
        &mut self,
        target: impl StageLabel,
        label: impl StageLabel,
        stage: S
    ) -> &mut AppBuilder {
        self
            .world_mut()
            .get_resource_mut::<RollbackSchedule>()
            .expect("Add RollbackSchedule to app!")
            .add_stage_before(target, label, stage);

        self
    }

    fn add_rollback_system_to_stage(
        &mut self,
        label: impl StageLabel,
        system: impl Into<SystemDescriptor>
    ) -> &mut AppBuilder {
        self
            .world_mut()
            .get_resource_mut::<RollbackSchedule>()
            .expect("Add RollbackSchedule to app!")
            .add_system_to_stage( label, system);

        self
    }
}