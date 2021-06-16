use bevy::tasks::ComputeTaskPool;
use bevy::ecs::entity::MapEntities;
use crate::reflect_resource::ReflectMapEntitiesResources;
use bevy::ecs::reflect::ReflectMapEntities;
use bevy::{
    reflect::{
        TypeRegistry, FromType, Reflect, GetTypeRegistration,
        erased_serde::private::serde::Serialize},
    ecs::reflect::ReflectComponent,
    ecs::world::FromWorld,
};
use std::ops::{Deref, DerefMut};
use std::collections::HashSet;
use std::any::{Any, TypeId};

use crate::reflect_resource::ReflectResource;

/// A wrapped TypeRegistry with primatives preinserted and serializable.
pub struct RollbackRegistry{
    pub(crate) registry: TypeRegistry,
    pub(crate) unregisterable: HashSet<TypeId>,
}

impl Default for RollbackRegistry{
   fn default() -> Self{
       let mut registry = RollbackRegistry{
           registry: TypeRegistry::default(),
           unregisterable: HashSet::default(),
        };
        
        registry.register::<u8>();
        registry.register::<bool>();
        registry.register::<u16>();
        registry.register::<u32>();
        registry.register::<u64>();
        registry.register::<u128>();
        registry.register::<usize>();
        registry.register::<i8>();
        registry.register::<i16>();
        registry.register::<i32>();
        registry.register::<i64>();
        registry.register::<i128>();
        registry.register::<isize>();
        registry.register::<f32>();
        registry.register::<f64>();
        registry.register::<String>();

        registry.register_unreflectable::<ComputeTaskPool>();
        
        registry
   } 
}

impl RollbackRegistry{
    pub fn register<T: Any + Reflect + GetTypeRegistration + FromWorld + Serialize>(&mut self) -> &mut Self{
        let mut registry = self.registry.write();
        registry.register::<T>();
        let registration = registry
            .get_mut(std::any::TypeId::of::<T>())
            .unwrap();
        registration.insert(<ReflectComponent as FromType<T>>::from_type());
        registration.insert(<ReflectResource as FromType<T>>::from_type());
        drop(registry);
        self
    }

    pub fn register_entity_mappable<T: Any + Reflect + GetTypeRegistration + FromWorld + Serialize + MapEntities>(&mut self) -> &mut Self{
        let mut registry = self.registry.write();
        registry.register::<T>();
        let registration = registry
            .get_mut(std::any::TypeId::of::<T>())
            .unwrap();
        registration.insert(<ReflectComponent as FromType<T>>::from_type());
        registration.insert(<ReflectResource as FromType<T>>::from_type());
        registration.insert(<ReflectMapEntities as FromType<T>>::from_type());
        registration.insert(<ReflectMapEntitiesResources as FromType<T>>::from_type());
        drop(registry);
        self
    }

    pub fn register_unreflectable<T: Any>(&mut self) -> &mut Self{
        self.unregisterable.insert(std::any::TypeId::of::<T>());
        self
    }
}