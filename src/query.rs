use crate::{components::ComponentsMask, Entities, Entity, EntityId};

pub trait Query {
    fn ecs(&self) -> &Entities;
    fn matches(&self, mask: &ComponentsMask) -> bool;
    fn iter(&self) -> impl Iterator<Item = QueryEntity> + '_ {
        self.ecs().entity_masks.iter().filter_map(|(idx, bitmask)| {
            self.matches(bitmask).then_some(QueryEntity {
                id: EntityId(idx),
                bitmask,
                ecs: self.ecs(),
            })
        })
    }
}

impl Query for Entities {
    fn ecs(&self) -> &Entities {
        self
    }

    fn matches(&self, _mask: &ComponentsMask) -> bool {
        true
    }
}

pub struct BitQuery<'a> {
    ecs: &'a Entities,
    included_mask: ComponentsMask,
    excluded_mask: ComponentsMask,
}

impl<'a> BitQuery<'a> {
    pub(crate) fn new(ecs: &'a Entities) -> Self {
        Self {
            ecs,
            included_mask: 0,
            excluded_mask: 0,
        }
    }

    pub fn with<T: 'static>(&mut self) -> &mut Self {
        self.included_mask |= self.ecs.components.mask_of::<T>();
        self
    }

    pub fn without<T: 'static>(&mut self) -> &mut Self {
        self.excluded_mask |= self.ecs.components.mask_of::<T>();
        self
    }
}

impl Entities {
    pub fn with<T: 'static>(&mut self) -> BitQuery<'_> {
        let mut q = BitQuery::new(self);
        q.with::<T>();
        q
    }

    pub fn without<T: 'static>(&mut self) -> BitQuery<'_> {
        let mut q = BitQuery::new(self);
        q.without::<T>();
        q
    }
}

impl<'a> Query for BitQuery<'a> {
    fn ecs(&self) -> &Entities {
        self.ecs
    }

    fn matches(&self, mask: &ComponentsMask) -> bool {
        mask & self.included_mask == self.included_mask && mask & self.excluded_mask == 0
    }
}

pub struct QueryEntity<'a> {
    id: EntityId,
    bitmask: &'a ComponentsMask,
    ecs: &'a Entities,
}

impl Entity for QueryEntity<'_> {
    fn ecs(&self) -> &Entities {
        self.ecs
    }

    fn id(&self) -> EntityId {
        self.id
    }

    fn has<T: 'static>(&self) -> bool {
        self.ecs.components.mask_of::<T>() & self.bitmask != 0
    }
}
