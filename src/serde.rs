use generational_arena::Arena;
use impl_trait_for_tuples::impl_for_tuples;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    components::{Components, ComponentsMask, SparseSet},
    EntityHandle, EntityId,
};

//TODO: Add compression

#[derive(serde::Serialize, serde::Deserialize)]
pub struct EcsState<S: ComponentSelection> {
    pub entity_masks: Arena<ComponentsMask>,
    pub components: S::Columns,
}

pub type EntityState<S> = <S as ComponentSelection>::EntityState;

pub trait ComponentSelection {
    type Columns: Serialize + DeserializeOwned + Clone + 'static;
    type EntityState: Serialize + DeserializeOwned + Clone + 'static;

    fn save_columns(components: &mut Components) -> Self::Columns;
    fn save_entity(id: EntityId, components: &mut Components) -> Self::EntityState;

    fn load_columns(components: &mut Components, columns: Self::Columns);
    fn load_entity(entity: &mut EntityHandle, state: Self::EntityState);
}

#[impl_for_tuples(16)]
#[tuple_types_no_default_trait_bound]
impl ComponentSelection for Tuple {
    for_tuples!( where #( Tuple: Serialize + DeserializeOwned + Clone + 'static )* );

    for_tuples!(type Columns = ( #( SparseSet<Tuple> ),* ); );

    fn save_columns(components: &mut Components) -> Self::Columns {
        for_tuples!( #( components.lazy_register::<Tuple>(); )*  );
        for_tuples!( ( #( components.borrow_storage_of::<Tuple>().clone() ),* ) )
    }

    for_tuples!(type EntityState = ( #( Option<Tuple> ),* ); );

    fn save_entity(id: EntityId, components: &mut Components) -> Self::EntityState {
        for_tuples!( #( components.lazy_register::<Tuple>(); )*  );
        for_tuples!( ( #( components.borrow_storage_of::<Tuple>().get(id.spatial()).cloned() ),* ) )
    }

    fn load_columns(components: &mut Components, columns: Self::Columns) {
        for_tuples!(  #( components.insert_column::<Tuple>(columns.Tuple); )*  )
    }

    fn load_entity(entity: &mut EntityHandle, state: Self::EntityState) {
        for_tuples!(  #( state.Tuple.map(|c| entity.set(c) );)* )
    }
}
