use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
};

use serde::{Deserialize, Serialize};

pub const COMPONENTS_MASK_SIZE: usize = 64;
pub type ComponentsMask = u64;

#[derive(Debug)]
pub struct Components {
    component_masks: HashMap<TypeId, (usize, ComponentsMask)>,
    next_component_mask: ComponentsMask,

    columns: Vec<RefCell<Column>>,
}

struct Column(Box<dyn ComponentsColumn>);

impl std::fmt::Debug for Column {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl Components {
    pub fn new() -> Self {
        Self {
            component_masks: HashMap::new(),
            next_component_mask: 1,
            columns: Vec::new(),
        }
    }

    pub fn insert_column<T: 'static>(&mut self, storage: SparseSet<T>) {
        if self.component_masks.contains_key(&TypeId::of::<T>()) {
            return;
        }

        let new_mask = self.next_component_mask;
        self.next_component_mask <<= 1;
        self.component_masks
            .insert(TypeId::of::<T>(), (self.component_masks.len(), new_mask));
        self.columns.push(RefCell::new(Column(Box::new(storage))));
    }

    pub fn lazy_register<T: 'static>(&mut self) {
        self.insert_column(SparseSet::<T>::new());
    }

    pub fn borrow_storage_of<T: 'static>(&self) -> Ref<SparseSet<T>> {
        Ref::map(
            self.columns
                .get(
                    self.component_masks
                        .get(&TypeId::of::<T>())
                        .expect("Component not registered")
                        .0,
                )
                .expect("Component not registered")
                .borrow(),
            |x| x.0.as_any_ref().downcast_ref::<SparseSet<T>>().unwrap(),
        )
    }

    pub fn borrow_storage_mut_of<T: 'static>(&self) -> RefMut<SparseSet<T>> {
        RefMut::map(
            self.columns
                .get(
                    self.component_masks
                        .get(&TypeId::of::<T>())
                        .expect("Component not registered")
                        .0,
                )
                .expect("Component not registered")
                .borrow_mut(),
            |x| x.0.as_any_mut().downcast_mut::<SparseSet<T>>().unwrap(),
        )
    }

    pub fn mask_of<T: 'static>(&self) -> ComponentsMask {
        self.component_masks
            .get(&TypeId::of::<T>())
            .map(|(_, mask)| *mask)
            .unwrap_or(0)
    }

    pub fn remove_all(&mut self, entity_mask: ComponentsMask, entity_spatial_idx: usize) {
        for i in 0..COMPONENTS_MASK_SIZE {
            if entity_mask & (1 << i) != 0 {
                let sparse_set = &mut self.columns[i];
                sparse_set.get_mut().0.remove(entity_spatial_idx);
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SparseSet<T> {
    dense: Vec<(T, usize)>,
    sparse: Vec<Option<usize>>,
}

impl<T> SparseSet<T> {
    pub fn new() -> Self {
        Self {
            dense: Vec::new(),
            sparse: Vec::new(),
        }
    }

    pub fn set(&mut self, index: usize, value: T) {
        if self.contains(index) {
            self.get_mut(index).map(|x| *x = value).unwrap();
        } else {
            if self.sparse.len() <= index {
                self.sparse.resize(index + 1, None);
            }
            self.dense.push((value, index));
            self.sparse[index] = Some(self.dense.len() - 1);
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.sparse
            .get(index)
            .map(|&i| self.dense.get(i.unwrap()).unwrap())
            .map(|x| &x.0)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.sparse
            .get(index)
            .map(|&i| self.dense.get_mut(i.unwrap()).unwrap())
            .map(|x| &mut x.0)
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        if let Some(&Some(i)) = self.sparse.get(index) {
            self.dense
                .last()
                .map(|x| &x.1)
                .copied()
                .and_then(|lastd_idx| {
                    if index != lastd_idx {
                        let len = self.dense.len();
                        self.dense.swap(i, len - 1);
                        self.sparse[lastd_idx] = Some(i);
                    }
                    self.sparse[index] = None;
                    self.dense.pop().map(|x| x.0)
                })
        } else {
            None
        }
    }

    pub fn contains(&self, index: usize) -> bool {
        self.sparse.get(index).map(|i| i.is_some()).unwrap_or(false)
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, &T)> + '_ {
        self.sparse
            .iter()
            .enumerate()
            .filter_map(|(idx, &i)| i.map(|i| (idx, &self.dense[i].0)))
    }
}

pub trait ComponentsColumn {
    fn as_any(self: Box<Self>) -> Box<dyn Any>;
    fn as_any_ref(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn remove(&mut self, index: usize);
}

impl<T: Any> ComponentsColumn for SparseSet<T> {
    fn as_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
    fn as_any_ref(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn remove(&mut self, index: usize) {
        self.remove(index);
    }
}
