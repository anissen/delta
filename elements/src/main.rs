use std::collections::HashMap;

type Entity = u32;

pub struct EntityManager {
    next_id: Entity,
}

impl EntityManager {
    fn new() -> Self {
        EntityManager { next_id: 0 }
    }

    fn create(&mut self) -> Entity {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

#[derive(Debug)]
pub struct ComponentColumn {
    dense_components: Vec<Component>,
    dense_entities: Vec<Entity>,
    sparse: Vec<Option<usize>>,
}

impl ComponentColumn {
    fn new() -> Self {
        Self {
            dense_components: Vec::new(),
            dense_entities: Vec::new(),
            sparse: Vec::new(),
        }
    }

    fn insert(&mut self, entity: Entity, component: Component) {
        let id = entity as usize;

        if let Some(Some(index)) = self.sparse.get(id) {
            // Replace component
            self.dense_components[*index] = component;
        } else {
            // Insert component
            if id >= self.sparse.len() {
                self.sparse.resize(id + 1, None);
            }

            let index = self.dense_components.len();
            self.dense_components.push(component);
            self.dense_entities.push(entity);
            self.sparse[id] = Some(index);
        }
    }

    fn get(&self, entity: Entity) -> Option<&Component> {
        let id = entity as usize;
        self.sparse
            .get(id)?
            .map(|index| &self.dense_components[index])
    }

    fn get_mut(&mut self, entity: Entity) -> Option<&mut Component> {
        let id = entity as usize;
        self.sparse
            .get(id)?
            .map(|index| &mut self.dense_components[index])
    }

    fn remove(&mut self, entity: Entity) -> Option<Component> {
        let id = entity as usize;
        let index = self.sparse.get_mut(id)?.take()?;

        let last_index = self.dense_components.len() - 1;

        if index != last_index {
            self.dense_components.swap(index, last_index);
            self.dense_entities.swap(index, last_index);
            let moved_entity = self.dense_entities[index];
            self.sparse[moved_entity as usize] = Some(index);
        }

        self.dense_entities.pop();
        self.dense_components.pop()
    }

    fn iter(&self) -> impl Iterator<Item = (&Entity, &Component)> {
        self.dense_entities.iter().zip(&self.dense_components)
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = (&Entity, &mut Component)> {
        self.dense_entities.iter().zip(&mut self.dense_components)
    }
}

pub struct ComponentStorage {
    component_sets: HashMap<ComponentId, ComponentColumn>, // TODO: Make this a Vec or SparseSet instead of a HashMap
}

impl ComponentStorage {
    fn new() -> Self {
        Self {
            component_sets: HashMap::new(),
        }
    }

    fn set(&mut self, entity: Entity, component_id: ComponentId, component: Component) {
        self.component_sets
            .entry(component_id)
            .or_insert_with(ComponentColumn::new)
            .insert(entity, component);
    }

    fn get(&self, component_id: &ComponentId) -> Option<&ComponentColumn> {
        self.component_sets.get(component_id)
    }

    fn get_mut(&mut self, component_id: &ComponentId) -> Option<&mut ComponentColumn> {
        self.component_sets.get_mut(component_id)
    }

    fn remove(&mut self, entity: Entity, component_id: ComponentId) -> Option<Component> {
        self.component_sets.get_mut(&component_id)?.remove(entity)
    }

    fn iter(&self, component_id: ComponentId) -> impl Iterator<Item = (&Entity, &Component)> {
        self.component_sets.get(&component_id).unwrap().iter()
    }

    fn iter_mut(
        &mut self,
        component_id: ComponentId,
    ) -> impl Iterator<Item = (&Entity, &mut Component)> {
        self.component_sets
            .get_mut(&component_id)
            .unwrap()
            .iter_mut()
    }
}

enum Value {
    Float(f32),
    Integer(i32),
}

impl Value {
    fn as_float(&self) -> f32 {
        match self {
            Value::Float(f) => *f,
            _ => panic!("Cannot convert non-float value to float"),
        }
    }
}

// #[derive(Debug, Clone, Copy)]
struct Component {
    values: HashMap<String, Value>, // TODO(anissen): Could be mapped to Vec<Value> where field names are mapped to indexes.
                                    // x: f32,
                                    // y: f32,
}

type ComponentId = u32;

const POSITION_ID: ComponentId = 0;
const VELOCITY_ID: ComponentId = 1;

fn movement_system(components: &mut ComponentStorage) {
    let positions = components.get(&POSITION_ID).unwrap();
    let velocities = components.get(&VELOCITY_ID).unwrap();

    // for (entity, position) in positions.iter_mut() {
    //     if let Some(vel) = velocities.get(entity) {
    //         position.x += vel.x;
    //         position.y += vel.y;
    //     }
    // }

    // Collect entities and velocity values that need updating
    let x = components.get_two_mut(&POSITION_ID, &VELOCITY_ID);

    // Apply updates
    // let positions = components.get_mut(&POSITION_ID).unwrap();
    for (pos, vel) in x {
        // if let Some(pos) = positions.get_mut(*entity) {
        // pos.x += vel.x;
        // pos.y += vel.y;
        let dx = vel.values.get("x").unwrap().as_float();
        let dy = vel.values.get("y").unwrap().as_float();
        let pos_x = pos.values.get("x").unwrap().as_float();
        let pos_y = pos.values.get("y").unwrap().as_float();
        pos.values.insert("x".to_string(), Value::Float(pos_x + dx));
        pos.values.insert("y".to_string(), Value::Float(pos_y + dy));
        // }
    }
}

fn position(x: f32, y: f32) -> Component {
    Component {
        values: HashMap::from([
            ("x".to_string(), Value::Float(x)),
            ("y".to_string(), Value::Float(y)),
        ]),
    }
}

fn velocity(dx: f32, dy: f32) -> Component {
    Component {
        values: HashMap::from([
            ("dx".to_string(), Value::Float(dx)),
            ("dy".to_string(), Value::Float(dy)),
        ]),
    }
}

fn main() {
    let mut entity_manager = EntityManager::new();
    let mut components = ComponentStorage::new();

    // Create a few entities
    let e1 = entity_manager.create();
    let e2 = entity_manager.create();
    let e3 = entity_manager.create();

    // Add components
    components.set(e1, POSITION_ID, position(0.0, 0.0));
    components.set(e1, VELOCITY_ID, velocity(1.0, 1.0));
    components.set(e1, VELOCITY_ID, velocity(1.0, 1.0));

    dbg!(&components.get(&VELOCITY_ID));
    components.remove(e1, VELOCITY_ID);
    dbg!(&components.get(&VELOCITY_ID));
    components.set(e2, POSITION_ID, position(10.0, -5.0));
    components.set(e2, VELOCITY_ID, velocity(-2.0, 0.5));

    components.set(e3, POSITION_ID, position(3.0, 3.0));
    // e3 has no velocity—won’t move

    // Run the movement system a few times
    for frame in 0..3 {
        println!("--- Frame {} ---", frame);
        movement_system(&mut components);

        for (entity, pos) in components.get(&POSITION_ID).unwrap().iter() {
            println!("Entity {}: Position = ({:.1}, {:.1})", entity, pos.x, pos.y);
        }
        components.set(e3, VELOCITY_ID, velocity(1.0, 1.0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
