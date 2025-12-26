use crate::{ComponentId, ComponentLayout, ComponentTypeId, Entity, column::Column};

pub struct QueryResultIter<'a> {
    iter: std::vec::IntoIter<(u32, Vec<&'a mut [u8]>)>,
}

impl<'a> QueryResultIter<'a> {
    pub fn new(results: Vec<(Entity, Vec<&'a mut [u8]>)>) -> Self {
        Self {
            iter: results.into_iter(),
        }
    }

    pub fn empty() -> Self {
        Self {
            iter: Vec::new().into_iter(),
        }
    }
}

impl<'a> Iterator for QueryResultIter<'a> {
    type Item = (Entity, Vec<&'a mut [u8]>);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

#[derive(Debug)]
pub struct World {
    components: Vec<Column>,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
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

    pub fn query<'a>(
        &'a mut self,
        include: &Vec<ComponentId>,
        exclude: &Vec<ComponentId>,
    ) -> QueryResultIter<'a> {
        // ) -> Vec<(Entity, Vec<&mut [u8]>)> {
        if include.is_empty() {
            return QueryResultIter::empty();
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

            let entities: Vec<_> = intersection.iter_ids().collect();
            let mut result = Vec::new();

            // SAFETY: We use raw pointers here to work around the borrow checker.
            // This is safe because:
            // 1. Each entity accesses a unique memory location within each column
            // 2. We never access the same memory location twice in the same iteration
            // 3. The pointers remain valid for the lifetime 'a (tied to &'a mut self)
            let column_ptrs: Vec<*mut Column> = include_columns
                .iter_mut()
                .map(|col| *col as *mut Column)
                .collect();

            for entity in entities {
                let mut row = Vec::new();
                for &col_ptr in &column_ptrs {
                    unsafe {
                        if let Some(data) = (*col_ptr).get_mut(entity) {
                            row.push(data);
                        }
                    }
                }
                result.push((entity, row));
            }

            QueryResultIter::new(result)
        } else {
            QueryResultIter::empty()
        }
    }
}
