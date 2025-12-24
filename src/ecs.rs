use std::any::{Any, TypeId};
use std::collections::HashMap;

pub type Entity = u32;

pub trait Component: Any + Send + Sync {}
impl<T: Any + Send + Sync> Component for T {}

pub struct World {
    next_entity_id: Entity,
    components: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_entity_id: 0,
            components: HashMap::new(),
        }
    }

    pub fn create_entity(&mut self) -> Entity {
        let id = self.next_entity_id;
        self.next_entity_id += 1;
        id
    }

    pub fn register_component<T: Component>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.components.entry(type_id).or_insert_with(|| Box::new(HashMap::<Entity, T>::new()));
    }

    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();
        if let Some(store) = self.components.get_mut(&type_id) {
            if let Some(map) = store.downcast_mut::<HashMap<Entity, T>>() {
                map.insert(entity, component);
            }
        } else {
            // Auto-register if not exists
            let mut map = HashMap::<Entity, T>::new();
            map.insert(entity, component);
            self.components.insert(type_id, Box::new(map));
        }
    }

    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        if let Some(store) = self.components.get(&type_id) {
            if let Some(map) = store.downcast_ref::<HashMap<Entity, T>>() {
                return map.get(&entity);
            }
        }
        None
    }
    
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        if let Some(store) = self.components.get_mut(&type_id) {
            if let Some(map) = store.downcast_mut::<HashMap<Entity, T>>() {
                return map.get_mut(&entity);
            }
        }
        None
    }

    // A simple query iterator could be added here, but for now direct access is enough.
}
