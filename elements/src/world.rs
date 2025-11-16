use crate::{ComponentId, ComponentLayout, ComponentTypeId, Entity, column::Column};

#[derive(Debug)]
pub struct World {
    components: Vec<Column>,
}

impl World {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    pub fn register_component(&mut self, id: ComponentTypeId, layout: ComponentLayout) {
        let column = Column::new(id, layout, 16);
        let idx = id as usize;
        if idx < self.components.len() {
            self.components[idx] = column;
        } else {
            self.components.push(column);
        }
    }

    pub fn insert(&mut self, id: ComponentTypeId, entity: Entity, data: &[u8]) {
        self.components[id as usize].insert(entity, data);
    }

    pub fn remove(&mut self, id: ComponentTypeId, entity: Entity) {
        self.components[id as usize].remove(entity);
    }

    pub fn get(&self, id: ComponentTypeId, entity: Entity) -> Option<&[u8]> {
        self.components[id as usize].get(entity)
    }

    pub fn get_mut(&mut self, id: ComponentTypeId, entity: Entity) -> Option<&mut [u8]> {
        self.components[id as usize].get_mut(entity)
    }

    pub fn iter(&self, id: ComponentTypeId) -> impl Iterator<Item = (Entity, &[u8])> + '_ {
        self.components[id as usize].iter()
    }

    pub fn system(
        &mut self,
        include: &Vec<ComponentId>,
        exclude: &Vec<ComponentId>,
        mut system: impl FnMut(Entity, &mut Vec<&mut [u8]>),
    ) {
        if include.is_empty() {
            return;
        }

        let exclude_columns: Vec<_> = self
            .components
            .iter()
            .filter(|c| exclude.contains(&c.id))
            .collect();

        // TODO(anissen): Could split in (first, rest) for optimization
        let exclude_bitset = if let Some(first) = exclude_columns.first() {
            let mut bitset = first.bitset.clone();
            self.components
                .iter()
                .filter(|c| exclude.contains(&c.id))
                .map(|col| &col.bitset)
                .for_each(|other| bitset.intersect_with(other));
            Some(bitset)
        } else {
            None
        };

        let mut include_columns: Vec<_> = self
            .components
            .iter_mut()
            .filter(|c| include.contains(&c.id))
            .collect();

        // TODO(anissen): Could split in (first, rest) for optimization
        if let Some(first) = include_columns.first() {
            let mut intersection = first.bitset.clone();
            include_columns
                .iter()
                .map(|col| &col.bitset)
                .for_each(|bitset| intersection.intersect_with(bitset));

            if let Some(exclude_bitset) = exclude_bitset {
                intersection.disjoint_with(&exclude_bitset);
            }

            intersection.iter_ids().for_each(|entity| {
                let mut row: Vec<_> = include_columns
                    .iter_mut()
                    .flat_map(|col| col.get_mut(entity))
                    .collect();
                system(entity, &mut row);
            });
        }
    }
}
