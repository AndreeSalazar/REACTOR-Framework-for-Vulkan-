use std::any::{Any, TypeId};
use std::collections::HashMap;

pub type Entity = u32;

pub trait Component: Any + Send + Sync {}
impl<T: Any + Send + Sync> Component for T {}

pub struct World {
    next_entity_id: Entity,
    components: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    entities: Vec<Entity>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_entity_id: 0,
            components: HashMap::new(),
            entities: Vec::new(),
        }
    }

    pub fn create_entity(&mut self) -> Entity {
        let id = self.next_entity_id;
        self.next_entity_id += 1;
        self.entities.push(id);
        id
    }

    pub fn destroy_entity(&mut self, entity: Entity) {
        self.entities.retain(|&e| e != entity);
        // Note: Components are not automatically removed
        // A proper ECS would handle this
    }

    pub fn entity_count(&self) -> usize {
        self.entities.len()
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
            let mut map = HashMap::<Entity, T>::new();
            map.insert(entity, component);
            self.components.insert(type_id, Box::new(map));
        }
    }

    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> Option<T> {
        let type_id = TypeId::of::<T>();
        if let Some(store) = self.components.get_mut(&type_id) {
            if let Some(map) = store.downcast_mut::<HashMap<Entity, T>>() {
                return map.remove(&entity);
            }
        }
        None
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

    pub fn has_component<T: Component>(&self, entity: Entity) -> bool {
        self.get_component::<T>(entity).is_some()
    }

    pub fn query<T: Component>(&self) -> impl Iterator<Item = (Entity, &T)> {
        let type_id = TypeId::of::<T>();
        self.components
            .get(&type_id)
            .and_then(|store| store.downcast_ref::<HashMap<Entity, T>>())
            .into_iter()
            .flat_map(|map| map.iter().map(|(&e, c)| (e, c)))
    }

    pub fn query_mut<T: Component>(&mut self) -> impl Iterator<Item = (Entity, &mut T)> {
        let type_id = TypeId::of::<T>();
        self.components
            .get_mut(&type_id)
            .and_then(|store| store.downcast_mut::<HashMap<Entity, T>>())
            .into_iter()
            .flat_map(|map| map.iter_mut().map(|(&e, c)| (e, c)))
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
