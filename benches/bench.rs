// Adapted from hecs (https://github.com/Ralith/hecs).

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

benchmark_group!(
    benches,
    spawn,
    iterate_100k,
    iterate_100k_no_id,
);
benchmark_main!(benches);
