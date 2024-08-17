use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
};

pub const COMPONENTS_MASK_SIZE: usize = 64;
pub type ComponentsMask = u64;

pub struct Components {
    component_masks: HashMap<TypeId, (usize, ComponentsMask)>,
    next_component_mask: ComponentsMask,
    columns: Vec<RefCell<Box<dyn ComponentsColumn>>>,
}

impl Components {
    pub fn new() -> Self {
        Self {
            component_masks: HashMap::new(),
            next_component_mask: 1,
            columns: Vec::new(),
        }
    }

    pub fn lazy_register<T: 'static>(&mut self) {
        if self.component_masks.contains_key(&TypeId::of::<T>()) {
            return;
        }

        let new_mask = self.next_component_mask;
        self.next_component_mask <<= 1;
        self.component_masks
            .insert(TypeId::of::<T>(), (self.component_masks.len(), new_mask));
        self.columns
            .push(RefCell::new(Box::new(SparseSet::<T>::new())));
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
            |x| x.as_any().downcast_ref::<SparseSet<T>>().unwrap(),
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
            |x| x.as_any_mut().downcast_mut::<SparseSet<T>>().unwrap(),
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
                sparse_set.get_mut().remove(entity_spatial_idx);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct SparseSet<T> {
    dense: Vec<T>,
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
            self.dense.push(value);
            self.sparse[index] = Some(self.dense.len() - 1);
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.sparse
            .get(index)
            .and_then(|&i| i.map(|i| &self.dense[i]))
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.sparse
            .get(index)
            .and_then(|&i| i.map(|i| &mut self.dense[i]))
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        self.sparse
            .get_mut(index)
            .and_then(|i| i.take().map(|i| self.dense.remove(i)))
    }

    pub fn contains(&self, index: usize) -> bool {
        self.sparse.get(index).map(|i| i.is_some()).unwrap_or(false)
    }
}

pub trait ComponentsColumn: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn remove(&mut self, index: usize);
}

impl<T: Any> ComponentsColumn for SparseSet<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn remove(&mut self, index: usize) {
        self.remove(index);
    }
}
