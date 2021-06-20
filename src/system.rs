use crate::util::{overwrite_world};
use bevy::tasks::ComputeTaskPool;
use crate::rollback_registry::RollbackRegistry;
use crate::rollback_schedule::RollbackSchedule;
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
        let target = rollback_buffer.current_frame() - rollback_buffer.rollback_needed();
        let rollback_world = rollback_buffer.get_world_mut(target).expect("Couldn't find world in buffer");
        overwrite_world(&rollback_world, &mut current_world, &rollback_registry);
    }

    for relative in (0..=rollback_buffer.rollback_needed()).rev(){
        let target = rollback_buffer.current_frame() - relative;
        if let Some(mut overrides) = rollback_buffer.remove_override(&target){
            overrides.run(&mut current_world);
        }
        rollback_buffer.push_world(&target, &current_world, &rollback_registry).unwrap();
        rollback_schedule.run_once(&mut current_world);
    }

    rollback_buffer.reset_rollback_needed();

    rollback_buffer.inc_frame();
}
