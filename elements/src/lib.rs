mod bitset;
mod column;
pub mod world;

pub type Entity = u32;
pub type ComponentId = u32;
pub type ComponentTypeId = u32;

#[derive(Debug)]
pub struct FieldLayout {
    pub name: String, // For debugging purposes
    pub type_id: u8,
    pub size: u16, // Needed when we already have type_id?
}

#[derive(Debug)]
pub struct ComponentLayout {
    pub fields: Vec<FieldLayout>,
    pub size: usize, // TODO(anissen): Change to u16?
}

impl ComponentLayout {
    pub fn new(fields: Vec<FieldLayout>) -> Self {
        let size = fields.iter().map(|f| f.size as usize).sum();
        ComponentLayout { fields, size }
    }
}

pub struct EntityManager {
    next_id: Entity,
}

impl EntityManager {
    pub fn new() -> Self {
        EntityManager { next_id: 0 }
    }

    pub fn create(&mut self) -> Entity {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}
