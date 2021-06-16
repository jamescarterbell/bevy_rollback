use bevy::ecs::component::ComponentInfo;

#[derive(Debug)]
pub enum RollbackError{
    UnregisteredType(String)
}