use crate::util::{overwrite_world};
use bevy::tasks::ComputeTaskPool;
use crate::rollback_registry::RollbackRegistry;
use crate::rollback_schedule::{RollbackSchedule, RollbackStartupSchedule};
use crate::rollback_buffer::RollbackBuffer;
use crate::RollbackWorld;
use bevy::prelude::*;

pub(crate) fn rollback_system(
    mut current_world: ResMut<RollbackWorld>,
    mut rollback_buffer: ResMut<RollbackBuffer>,
    mut rollback_schedule: ResMut<RollbackSchedule>,
    rollback_registry: Res<RollbackRegistry>,
){
    if rollback_buffer.rollback_needed() > 0{
        let target = rollback_buffer.current_frame() as isize - rollback_buffer.rollback_needed();
        let rollback_world = rollback_buffer.get_world_mut(target as usize).expect("Couldn't find world in buffer");
        overwrite_world(&rollback_world, &mut current_world, &rollback_registry).unwrap();
    }
    for target in (rollback_buffer.current_frame() as isize - rollback_buffer.rollback_needed())..=rollback_buffer.current_frame() as isize{
        if let Some(overrides) = rollback_buffer.get_override_mut(&(target as isize)){
            overrides.run(&mut current_world);
        }
        rollback_buffer.push_world(&(target as usize), &current_world, &rollback_registry).unwrap();
        rollback_schedule.run_once(&mut current_world);
    }

    rollback_buffer.reset_rollback_needed();

    rollback_buffer.inc_frame();
}

/// A component on a rollback entity to mark if it's been synced.
pub(crate) struct SyncedRollback;

/// A component on an outer world entity with a handle to a Rollback World entity.
pub struct Synced{
    pub target: Entity,
}

pub fn sync_rollback_entities(
    mut commands: Commands,
    mut rollback_world: ResMut<RollbackWorld>,
    synced: Query<(Entity, &Synced)>,
){
    for (entity, synced) in synced.iter(){
        if let None = rollback_world.get_entity(synced.target){
            commands
                .entity(entity)
                .despawn();
        }
    }

    let mut syncable = Vec::new();

    for entity in rollback_world.query_filtered::<Entity, Without<SyncedRollback>>().iter(&mut rollback_world){
        commands
            .spawn()
            .insert(
                Synced{target: entity.clone()}
            );
        syncable.push(entity.clone());
    }

    for syncable in syncable{
        rollback_world
            .entity_mut(syncable)
            .insert(SyncedRollback);
    }
}

pub fn rollback_startup(
    mut rollback_world: ResMut<RollbackWorld>,
    mut rollback_startup_schedule: ResMut<RollbackStartupSchedule>,
){
    rollback_startup_schedule.run(&mut rollback_world);
}