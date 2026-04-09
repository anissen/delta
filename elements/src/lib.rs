mod bitset;
mod column;
pub mod world;

pub type Entity = u32;
pub type ComponentId = u32;
// pub type ComponentTypeId = u32;

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

    // return an iterator for byte slices representing fields, each offset by the size of the previous
    // pub fn iter(&self, data: &[u8]) -> impl Iterator<Item = &[u8]> {
    //     self.fields.iter().scan(0, |offset, field| {
    //         let start = *offset;
    //         *offset += field.size as usize;
    //         Some(&data[start..*offset])
    //     })
    // }
}

pub struct EntityManager {
    next_id: Entity,
}

impl Default for EntityManager {
    fn default() -> Self {
        Self::new()
    }
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
