use generational_arena::Arena;
use impl_trait_for_tuples::impl_for_tuples;
use serde::{de::DeserializeOwned, Serialize};

use crate::components::{Components, ComponentsMask, SparseSet};

//TODO: Add compression

#[derive(serde::Serialize, serde::Deserialize)]
pub struct EcsState<S: ComponentSelection> {
    pub entity_masks: Arena<ComponentsMask>,
    pub components: S::Columns,
}

pub trait ComponentSelection {
    type Columns: Serialize + DeserializeOwned + Clone + 'static;
    fn save_columns(components: &Components) -> Self::Columns;
    fn load_columns(columns: Self::Columns) -> Components;
}

#[impl_for_tuples(16)]
#[tuple_types_no_default_trait_bound]
impl ComponentSelection for Tuple {
    for_tuples!( where #( Tuple: Serialize + DeserializeOwned + Clone + 'static )* );

    for_tuples!(type Columns = ( #( SparseSet<Tuple> ),* ); );

    fn save_columns(components: &Components) -> Self::Columns {
        for_tuples!( ( #( components.borrow_storage_of::<Tuple>().clone() ),* ) )
    }

    fn load_columns(columns: Self::Columns) -> Components {
        let mut components = Components::new();
        for_tuples!(  #( components.insert_column::<Tuple>(columns.Tuple); )*  );
        components
    }
}
