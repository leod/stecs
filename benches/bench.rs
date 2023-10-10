// Adapted from hecs (https://github.com/Ralith/hecs).

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use bencher::{benchmark_group, benchmark_main, Bencher};

use stecs::{EntityId, World};

#[derive(Clone)]
struct Position(f32);

#[derive(Clone)]
struct Velocity(f32);

#[derive(stecs::Entity)]
struct Bundle {
    pos: Position,
    vel: Velocity,
}

fn spawn(b: &mut Bencher) {
    let mut world = World::<Bundle>::new();
    b.iter(|| {
        world.spawn(Bundle {
            pos: Position(0.0),
            vel: Velocity(0.0),
        });
    });
}

fn iterate_100k(b: &mut Bencher) {
    let mut world = World::<Bundle>::new();

    for i in 0..100_000 {
        world.spawn(Bundle {
            pos: Position(-(i as f32)),
            vel: Velocity(i as f32),
        });
    }

    b.iter(|| {
        for (_, pos, vel) in world.query_mut::<(EntityId<Bundle>, &mut Position, &Velocity)>() {
            pos.0 += vel.0;
        }
    })
}

fn iterate_100k_no_id(b: &mut Bencher) {
    let mut world = World::<Bundle>::new();

    for i in 0..100_000 {
        world.spawn(Bundle {
            pos: Position(-(i as f32)),
            vel: Velocity(i as f32),
        });
    }

    b.iter(|| {
        for (pos, vel) in world.query_mut::<(&mut Position, &Velocity)>() {
            pos.0 += vel.0;
        }
    })
}

fn iterate_100k_random_access(b: &mut Bencher) {
    #[derive(stecs::Entity)]
    struct Enemy {
        pos: Position,
        vel: Velocity,
        target: EntityId<Entity>,
    }

    #[derive(stecs::Entity)]
    struct Target {
        pos: Position,
    }

    #[derive(stecs::Entity)]
    enum Entity {
        Enemy(Enemy),
        Target(Target),
    }

    let mut world = World::<Entity>::new();

    let mut targets = Vec::new();

    for i in 0..100_000 {
        let target = world
            .spawn(Target {
                pos: Position(i as f32),
            })
            .to_outer();

        targets.push(target);
    }

    for i in 0..100_000 {
        let mut hasher = DefaultHasher::new();
        i.hash(&mut hasher);

        let target = targets[hasher.finish() as usize % targets.len()];

        world.spawn(Enemy {
            pos: Position(-(i as f32)),
            vel: Velocity(i as f32),
            target,
        });
    }

    b.iter(|| {
        let (query_a, query_b) =
            world.queries_mut::<((&EntityId<Entity>, &Position, &mut Velocity), &Position)>();

        for (target_a, pos_a, vel_a) in query_a {
            let pos_b = query_b.get(*target_a).unwrap();

            vel_a.0 = pos_b.0 - pos_a.0;
        }
    })
}

benchmark_group!(
    benches,
    spawn,
    iterate_100k,
    iterate_100k_no_id,
    iterate_100k_random_access
);
benchmark_main!(benches);
