use stecs::{EntityId, EntityRef, EntityRefMut, WorldData};

// Components

#[derive(Debug, Clone, Copy)]
struct Position(i32);

impl Position {
    fn distance(&self, other: &Position) -> i32 {
        (self.0 - other.0).abs()
    }
}

#[derive(Debug)]
struct Velocity(i32);

#[derive(Debug)]
struct Health(i32);

#[derive(Debug)]
struct Target(Option<EntityId<Entity>>);

// Entities

#[derive(stecs::Entity)]
struct Player {
    pos: Position,
    vel: Velocity,
    health: Health,
}

#[derive(stecs::Entity)]
struct Enemy {
    pos: Position,
    vel: Velocity,
    health: Health,
    target: Target,
}

#[derive(stecs::Entity)]
struct Bullet {
    pos: Position,
    vel: Velocity,
    owner: EntityId<Entity>,
}

// World

// Define your world by declaring an enum that contains all the entity variants:
#[derive(stecs::Entity)]
enum Entity {
    Player(Player),
    Enemy(Enemy),
    Bullet(Bullet),
}

type World = stecs::World<Entity>;

fn create_world() -> World {
    let mut world = World::new();

    let id = world.spawn(Player {
        pos: Position(0),
        vel: Velocity(1),
        health: Health(10),
    });

    println!("First player's ID: {id:?}");

    for x in -5..5 {
        world.spawn(Enemy {
            pos: Position(x * 2),
            vel: Velocity(0),
            health: Health(3),
            target: Target(None),
        });
    }

    // Can also spawn entities via the `Entity` enum:
    let id = world.spawn(Entity::Player(Player {
        pos: Position(5),
        vel: Velocity(-1),
        health: Health(10),
    }));

    println!("Second player's ID: {id:?}");

    world
}

// Game logic

fn integrate_time(world: &mut World) {
    for (pos, vel) in world.query_mut::<(&mut Position, &Velocity)>() {
        pos.0 += vel.0;
    }
}

fn align_to_target(world: &mut World) {
    // Acquire targets.
    for ((pos, target), nest) in world
        .query_mut::<(&Position, &mut Target)>()
        .nest_off_diagonal::<(EntityId<Player>, EntityRef<Player>)>()
    {
        // We can use `nest` to perform nested queries. `nest` prevents aliasing
        // by disallowing to query the current entity. (In this case, the two
        // queries are non-overlapping anyway, so we could also use
        // `World::queries()` to obtain multiple independent queries, but I
        // haven't actually implemented that yet.)

        if target.0.is_some() {
            continue;
        }

        target.0 = nest
            .into_iter()
            .min_by_key(|(_, player)| player.pos.distance(pos))
            .map(|(id, _)| id.to_outer());
    }

    // Set velocities to point to targets.
    for ((vel, pos, target), mut nest) in world
        .query_mut::<(&mut Velocity, &Position, &mut Target)>()
        .nest_off_diagonal::<&Position>()
    {
        let Some(target_pos) = target.0.and_then(|target| nest.get_mut(target)) else {
            continue;
        };

        let dist = pos.distance(target_pos);

        if dist > 3 {
            // Oh noes, lost sight of our target!
            target.0 = None;
        }

        vel.0 = (pos.0 - target_pos.0).signum();
    }
}

fn spawn_bullets(world: &mut World) {
    let bullets: Vec<_> = world
        .query::<(EntityId<Entity>, &Position, &Velocity)>()
        .with::<&Target>()
        .into_iter()
        .map(|(id, pos, vel)| Bullet {
            pos: *pos,
            vel: Velocity(vel.0 * 2),
            owner: id,
        })
        .collect();

    for bullet in bullets {
        world.spawn(bullet);
    }
}

fn update_bullets(world: &mut World) {
    for (bullet, nest) in world
        .query_mut::<EntityRefMut<Bullet>>()
        .nest_off_diagonal::<(EntityId<Entity>, &Position, &mut Health)>()
    {
        // For performance reasons, this check would usually be done with
        // a spatial acceleration structure rather than an inner loop.
        for (id, pos, health) in nest {
            if bullet.pos.0 == pos.0 {
                println!("Bullet by {:?} hit {:?}", bullet.owner, id);
                health.0 -= 1;
            }
        }
    }
}

fn despawn_dead(world: &mut World) {
    let dead: Vec<_> = world
        .query::<(EntityId<Entity>, &Health)>()
        .into_iter()
        .filter(|(_, health)| health.0 <= 0)
        .map(|(id, _)| id)
        .collect();

    for id in dead {
        let Some(entity) = world.despawn(id) else {
            continue;
        };

        match entity {
            Entity::Player(entity) => println!("Killed Player: {:?}", entity.pos),
            Entity::Enemy(entity) => println!("Killed Enemy: {:?}", entity.pos),
            _ => (),
        }
    }
}

fn run_tick(world: &mut World) {
    integrate_time(world);
    align_to_target(world);
    update_bullets(world);
    spawn_bullets(world);
    despawn_dead(world);
}

fn print_world(world: &World) {
    for entity in world.query::<EntityRef<Entity>>() {
        // We can pattern match entity references to do type-specific things.
        // TODO: Allow specifying traits that should be derived for the
        // generated `EntityRef` etc. types.
        match entity {
            EntityRef::<Entity>::Player(entity) => {
                println!("Good Player: {:?} {:?}", entity.pos, entity.health)
            }
            EntityRef::<Entity>::Enemy(entity) => {
                println!("Evil Enemy: {:?} {:?}", entity.pos, entity.health)
            }
            EntityRef::<Entity>::Bullet(entity) => println!("Meh Bullet: {:?}", entity.pos),
        }
    }
}

fn main() {
    let mut world = create_world();

    for _ in 0..10 {
        run_tick(&mut world);
    }

    print_world(&world);
}
