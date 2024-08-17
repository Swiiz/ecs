# Swiiz's ECS

> An Entity Component System (ECS) is a design pattern in game development where entities are data containers, components hold data, and systems define behavior. This separation enhances flexibility, reusability, and performance in complex simulations.

## How to use ?


```rust
// examples/walkthrough.rs

use ecs::{Entities, Entity, Query};

pub struct Age(u8);

fn main() {
    let mut entities = Entities::new();

    let mut a = entities.spawn();
    a.set("Entity A");

    assert!(a.get::<&str>().is_some());
    assert!(a.get::<Age>().is_none());

    let mut b = entities.spawn();
    b.set("Entity B");
    b.set(Age(10));

    assert!(b.get::<&str>().is_some());
    assert!(b.get::<Age>().is_some());

    let mut c = entities.spawn();
    c.set(Age(20));

    let unamed_entity_id = c.id();

    assert!(c.get::<&str>().is_none());
    assert!(c.get::<Age>().is_some());

    for _ in 0..5 {
        age_entities(&mut entities);
    }

    print_entities(&mut entities);

    let unamed_entity = entities.get(&unamed_entity_id).unwrap();
    print!(
        "Unamed entity age: {}",
        unamed_entity.get::<Age>().unwrap().0
    );
}

fn age_entities(entities: &mut Entities) {
    for entity in entities.with::<Age>().iter() {
        entity.get_mut::<Age>().unwrap().0 += 1;
    }
}

fn print_entities(entities: &mut Entities) {
    for entity in entities.with::<&str>().iter() {
        let name = entity.get::<&str>().unwrap();

        let age = entity
            .get::<Age>()
            .map(|x| x.0.to_string())
            .unwrap_or("Unknown".to_string());

        println!("{} has age {}", name, age);
    }
}

```