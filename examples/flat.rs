use stecs::{EntityFromRef, EntityId, EntityRef};

#[derive(stecs::Entity, Clone, Debug)]
#[stecs(derive_columns(Clone), derive_ref(Debug))]
struct Projectile {
    position: f32,
    velocity: i32,
}

#[derive(stecs::Entity, Clone)]
#[stecs(derive_columns(Clone), derive_ref(Debug))]
struct Bullet {
    #[stecs(flat)]
    projectile: Projectile,
}

#[derive(stecs::Entity, Clone)]
#[stecs(derive_ref(Debug))]
enum Entity {
    Projectile(Projectile),
    Bullet(Bullet),
}

type World = stecs::World<Entity>;

fn main() {
    let mut world = World::default();

    world.spawn(Projectile {
        position: 1.0,
        velocity: -3,
    });

    let id = world.spawn(Bullet {
        projectile: Projectile {
            position: 2.0,
            velocity: -4,
        },
    });

    for id in world.query::<EntityId<Entity>>().with::<(&f32, &i32)>() {
        dbg!(id);
    }

    let entity_ref = world.entity(id).unwrap();
    let _ = Bullet::from_ref(entity_ref);

    let entity_ref = world.entity(EntityId::<Entity>::from(id)).unwrap();
    let _ = Entity::from_ref(entity_ref);

    for (id, projectile) in world.query::<(EntityId<Entity>, EntityRef<Projectile>)>() {
        println!("{:?}: {:?}", id, projectile);
    }

    println!("---");

    for (id, projectile) in world.query::<(EntityId<Projectile>, EntityRef<Projectile>)>() {
        println!("{:?}: {:?}", id, projectile);
    }

    println!("---");

    for (id, projectile) in world.query::<(EntityId<Entity>, EntityRef<Entity>)>() {
        println!("{:?}: {:?}", id, projectile);
    }
}
 