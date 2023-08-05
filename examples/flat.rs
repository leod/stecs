use stecs::{EntityFromRef, EntityId};

#[derive(stecs::Entity, Clone)]
#[stecs(derive_columns(Clone))]
struct Projectile {
    position: f32,
    velocity: i32,
}

#[derive(stecs::Entity, Clone)]
#[stecs(derive_columns(Clone))]
struct Bullet {
    #[stecs(flat)]
    projectile: Projectile,
}

#[derive(stecs::Entity, Clone)]
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
}
