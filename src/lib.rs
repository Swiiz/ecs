use std::cell::{Ref, RefMut};

use components::{Components, ComponentsMask};
use generational_arena::{Arena, Index};

mod components;
mod query;

pub mod serde;

use ::serde::{Deserialize, Serialize};
pub use query::Query;
use serde::{ComponentSelection, EcsState};

#[derive(Debug)]
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
        self.edit(EntityId(id)).unwrap()
    }

    pub fn edit(&mut self, id: EntityId) -> Option<EntityHandle> {
        if self.is_present(id) {
            Some(EntityHandle { id, ecs: self })
        } else {
            None
        }
    }

    pub fn is_present(&self, entity: EntityId) -> bool {
        self.entity_masks.contains(entity.index())
    }

    pub fn save_entity<S: ComponentSelection>(&mut self, id: EntityId) -> S::EntityState {
        S::save_entity(id, &mut self.components)
    }

    pub fn load_entity<S: ComponentSelection>(
        &mut self,
        entity_id: AliveEntityId,
        state: S::EntityState,
    ) -> EntityHandle {
        let mut entity = self.spawn();
        assert!(entity_id.0 == entity.id().spatial(), "Ecs desync detected");
        S::load_entity(&mut entity, state);
        entity
    }

    pub fn save<S: ComponentSelection>(&mut self) -> EcsState<S> {
        EcsState {
            entity_masks: self.entity_masks.clone(),
            components: S::save_columns(&mut self.components),
        }
    }

    pub fn load<S: ComponentSelection>(state: EcsState<S>) -> Self {
        let mut components = Components::new();

        S::load_columns(&mut components, state.components);

        Self {
            entity_masks: state.entity_masks,
            components,
        }
    }
}

/// /!\ Entity that is known to be alive, used for serialization and deserialization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AliveEntityId(usize);

impl From<EntityId> for AliveEntityId {
    fn from(id: EntityId) -> Self {
        Self(id.spatial())
    }
}

impl AliveEntityId {
    pub fn validate(&self, entities: &Entities) -> EntityId {
        EntityId(
            entities
                .entity_masks
                .get_unknown_gen(self.0)
                .expect("Entity not present! This should never happen")
                .1,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    pub fn set<T: 'static>(&mut self, component: T) -> &mut Self {
        let components = &mut self.ecs.components;
        components.lazy_register::<T>();
        components
            .borrow_storage_mut_of::<T>()
            .set(self.id.spatial(), component);
        *self.mask_mut() |= components.mask_of::<T>();

        self
    }

    pub fn remove<T: 'static>(&mut self, entity: &EntityId) {
        let components = &mut self.ecs.components;
        components.lazy_register::<T>();
        components
            .borrow_storage_mut_of::<T>()
            .remove(entity.spatial());
        *self.mask_mut() &= !components.mask_of::<T>();
    }

    pub fn despawn(&mut self) {
        if let Some(mask) = self.ecs.entity_masks.remove(self.id.index()) {
            self.ecs.components.remove_all(mask, self.id.spatial());
        };
    }
}

pub trait Entity {
    fn ecs(&self) -> &Entities;
    fn id(&self) -> EntityId;
    fn has<T: 'static>(&self) -> bool;

    fn get<T: 'static>(&self) -> Option<Ref<T>> {
        self.has::<T>().then_some(
            Ref::filter_map(self.ecs().components.borrow_storage_of::<T>(), |s| {
                s.get(self.id().spatial())
            })
            .ok()?,
        )
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
