use bevy::prelude::*;
use std::ops::{Deref, DerefMut};

#[derive(Default)]
pub struct RollbackSchedule{
    schedule: Schedule
}

impl Deref for RollbackSchedule{
    type Target = Schedule;

    fn deref(&self) -> &Self::Target{
        &self.schedule
    }
}

impl DerefMut for RollbackSchedule{
    fn deref_mut(&mut self) -> &mut Self::Target{
        &mut self.schedule
    }
}

