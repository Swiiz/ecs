use std::cell::{Ref, RefMut};

use components::{Components, ComponentsMask};
use generational_arena::{Arena, Index};
use query::BitQuery;

mod components;
mod query;

pub use query::Query;

pub struct Entities {
    pub(crate) entity_masks: Arena<ComponentsMask>,
    pub(crate) components: Components,
}

impl Entities {
    pub fn new() -> Self {
        Self {
            entity_masks: Arena::new(),
            components: Components::new(),
        }
    }

    pub fn spawn(&mut self) -> EntityHandle {
        let id = self.entity_masks.insert(ComponentsMask::default());
        self.get(&EntityId(id)).unwrap()
    }

    pub fn get(&mut self, id: &EntityId) -> Option<EntityHandle> {
        if self.is_present(&id) {
            Some(EntityHandle { id: *id, ecs: self })
        } else {
            None
        }
    }

    pub fn is_present(&self, entity: &EntityId) -> bool {
        self.entity_masks.contains(entity.index())
    }

    pub fn query(&self) -> BitQuery {
        BitQuery::new(self)
    }
}

#[derive(Clone, Copy)]
pub struct EntityId(Index);

impl EntityId {
    pub(crate) fn spatial(&self) -> usize {
        self.0.into_raw_parts().0
    }

    pub(crate) fn index(&self) -> Index {
        self.0
    }
}

pub struct EntityHandle<'a> {
    id: EntityId,
    ecs: &'a mut Entities,
}

impl<'a> EntityHandle<'a> {
    pub(crate) fn mask(&self) -> ComponentsMask {
        *self.ecs.entity_masks.get(self.id.index()).unwrap()
    }

    pub(crate) fn mask_mut(&mut self) -> &mut ComponentsMask {
        self.ecs
            .entity_masks
            .get_mut(self.id.index())
            .expect("Entity not present")
    }

    pub fn set<T: 'static>(&mut self, component: T) {
        let components = &mut self.ecs.components;
        components.lazy_register::<T>();
        components
            .borrow_storage_mut_of::<T>()
            .set(self.id.spatial(), component);
        *self.mask_mut() |= components.mask_of::<T>();
    }

    pub fn remove<T: 'static>(&mut self, entity: &EntityId) {
        let components = &mut self.ecs.components;
        components.lazy_register::<T>();
        components
            .borrow_storage_mut_of::<T>()
            .remove(entity.spatial());
        *self.mask_mut() &= !components.mask_of::<T>();
    }

    pub fn has<T: 'static>(&self) -> bool {
        self.ecs.components.mask_of::<T>() & self.mask() != 0
    }

    pub fn despawn(&mut self, entity: &EntityId) {
        if let Some(mask) = self.ecs.entity_masks.remove(entity.0) {
            self.ecs.components.remove_all(mask, entity.spatial());
        };
    }
}

pub trait Entity {
    fn ecs(&self) -> &Entities;
    fn id(&self) -> EntityId;
    fn has<T: 'static>(&self) -> bool;

    fn get<T: 'static>(&self) -> Option<Ref<T>> {
        self.has::<T>().then(|| {
            Ref::map(self.ecs().components.borrow_storage_of::<T>(), |s| {
                s.get(self.id().spatial()).unwrap()
            })
        })
    }

    fn get_mut<T: 'static>(&self) -> Option<RefMut<T>> {
        self.has::<T>().then(|| {
            RefMut::map(self.ecs().components.borrow_storage_mut_of::<T>(), |s| {
                s.get_mut(self.id().spatial()).unwrap()
            })
        })
    }
}

impl Entity for EntityHandle<'_> {
    fn ecs(&self) -> &Entities {
        self.ecs
    }

    fn id(&self) -> EntityId {
        self.id
    }

    fn has<T: 'static>(&self) -> bool {
        self.ecs.components.mask_of::<T>() & self.mask() != 0
    }
}
