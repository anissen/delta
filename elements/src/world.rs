use crate::{
    ComponentId, ComponentLayout, ComponentTypeId, Entity,
    bitset::{BitSet, BitSetIter, ClonedBitSetIter},
    column::Column,
};

pub struct QueryResultIter {
    iter: std::vec::IntoIter<(u32, Vec<Vec<u8>>)>,
}

impl QueryResultIter {
    pub fn new(results: Vec<(Entity, Vec<Vec<u8>>)>) -> Self {
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

impl Iterator for QueryResultIter {
    type Item = (Entity, Vec<Vec<u8>>);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub struct QueryResultMutIter<'a> {
    iter: std::vec::IntoIter<(u32, Vec<&'a mut [u8]>)>,
}

impl<'a> QueryResultMutIter<'a> {
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

impl<'a> Iterator for QueryResultMutIter<'a> {
    type Item = (Entity, Vec<&'a mut [u8]>);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

// --------------------

// pub struct QueryResultEntry {
//     entity: Entity,
//     // column_ids: &Vec<ComponentId>,
//     // columns: &'a Vec<&'a mut Column>,
// }

pub struct QueryResult<'a> {
    entities: std::vec::IntoIter<Entity>,
    // column_ids: Vec<ComponentId>,
    pub columns: Vec<&'a mut Column>,
}

impl<'a> QueryResult<'a> {
    pub fn new(entities: Vec<Entity>, columns: Vec<&'a mut Column>) -> Self {
        Self {
            entities: entities.into_iter(),
            columns,
        }
    }

    // pub fn empty() -> Self {
    //     Self {
    //         entities: Vec::new().into_iter(),
    //         columns: Vec::new(),
    //     }
    // }
}

impl<'a> Iterator for QueryResult<'a> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        self.entities.next()
    }
}

// --------------------

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

    pub fn get_component_layout(&self, id: ComponentTypeId) -> Option<&ComponentLayout> {
        self.components
            .get(id as usize)
            .map(|column| &column.layout)
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

    pub fn query_mut<'a>(
        &'a mut self,
        include: &Vec<ComponentId>,
        exclude: &Vec<ComponentId>,
    ) -> QueryResultMutIter<'a> {
        // ) -> Vec<(Entity, Vec<&mut [u8]>)> {
        if include.is_empty() {
            return QueryResultMutIter::empty();
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

            QueryResultMutIter::new(result)
        } else {
            QueryResultMutIter::empty()
        }
    }

    pub fn query(
        &self,
        include: &Vec<ComponentId>,
        exclude: &Vec<ComponentId>,
    ) -> Option<QueryResultIter> {
        if include.is_empty() {
            return None;
        }

        let exclude_columns: Vec<_> = self
            .components
            .iter()
            .filter(|c| exclude.contains(&c.id))
            .collect(); // TODO(anissen): Do I need to collect here?

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

        let include_columns: Vec<_> = self
            .components
            .iter()
            .filter(|c| include.contains(&c.id))
            .collect(); // TODO(anissen): Do I need to collect here?

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

            let result = intersection
                .iter_ids()
                .map(move |entity| {
                    // TODO(anissen): This is probably expensive:
                    let row = include_columns
                        .iter()
                        .map(|row| row.get(entity).unwrap().to_vec())
                        .collect();
                    (entity.clone(), row)
                })
                .collect();
            Some(QueryResultIter::new(result))
        } else {
            None
        }
    }

    pub fn query2(
        &self,
        include: &Vec<ComponentId>,
        exclude: &Vec<ComponentId>,
    ) -> ClonedBitSetIter {
        // if include.is_empty() {
        //     return None;
        // }

        // if let Some(first) = include_columns.first() {
        // }
        let mut matching_entities = BitSet::new_filled(64);

        self.components
            .iter()
            .filter(|c| include.contains(&c.id))
            .for_each(|col| matching_entities.intersect_with(&col.bitset));

        self.components
            .iter()
            .filter(|c| exclude.contains(&c.id))
            .for_each(|col| matching_entities.disjoint_with(&col.bitset));

        matching_entities.cloned_iter_ids()
    }

    pub fn query3<'a>(
        &'a mut self,
        include: &Vec<ComponentId>,
        exclude: &Vec<ComponentId>,
    ) -> QueryResult<'a> {
        let exclude_columns = self
            .components
            .iter()
            .filter(|c| exclude.contains(&c.id))
            .collect::<Vec<_>>();

        let exclude_bitmap = if let Some((first, rest)) = exclude_columns.split_first() {
            let mut bitset = first.bitset.clone();
            for col in rest {
                bitset.intersect_with(&col.bitset);
            }
            bitset
        } else {
            BitSet::new_empty(0)
        };

        let include_columns = self
            .components
            .iter_mut()
            .filter(|c| include.contains(&c.id))
            .collect::<Vec<_>>();

        let matching_entities = if let Some((first, rest)) = include_columns.split_first() {
            let mut bitset = first.bitset.clone();
            for col in rest {
                bitset.intersect_with(&col.bitset);
            }
            bitset.disjoint_with(&exclude_bitmap);

            bitset.iter_ids().collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        if !matching_entities.is_empty() {
            QueryResult::new(matching_entities, include_columns)
        } else {
            QueryResult::new(Vec::new(), Vec::new())
        }
    }
}
