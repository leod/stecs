use stecs::{entity::EntityVariant, Component, EntityId, EntityRef, EntityRefMut, WorldData};

#[derive(Clone)]
struct Position(f32);

#[derive(Clone)]
struct Velocity(f32);

#[derive(Clone)]
struct Color(f32);

#[derive(stecs::Entity, Clone)]
struct Player {
    pos: Position,
    vel: Velocity,

    col: Color,
}

#[derive(stecs::Entity, Clone)]
struct Boier<T: Component, S: Component> {
    pos: T,
    vel: S,
    col: Color,
}

#[derive(stecs::Entity, Clone)]
struct Blob;

#[derive(stecs::Entity, Clone)]
struct Blub(u32);

#[derive(Clone, Debug)]
struct Target(EntityId<Entity>);

#[derive(stecs::Entity, Clone)]
struct Enemy {
    pos: Position,
    target: Target,
}

#[derive(stecs::Entity, Clone)]
struct InnerEnemy {
    pos: Position,
    target: Target,
}

#[derive(stecs::Entity, Clone)]
enum InnerEntity {
    Enemy(InnerEnemy),
    Boier(Boier<Position, Blub>),
}

#[derive(stecs::Entity, Clone)]
enum Entity {
    Inner(InnerEntity),
    Player(Player),
    Enemy(Enemy),
    Boier(Boier<Position, Velocity>),
}

type World = stecs::World<Entity>;

fn main() {
    let mut world = World::default();

    let p0 = world.spawn(Player {
        pos: Position(1.0),
        vel: Velocity(2.0),
        col: Color(3.0),
    });

    println!("muh id {:?}", p0);

    let p1 = world.spawn(Player {
        pos: Position(1.5),
        vel: Velocity(2.0),
        col: Color(3.0),
    });

    world.spawn(Enemy {
        pos: Position(-1.5),
        target: Target(p0.to_outer()),
    });

    world.spawn(Enemy {
        pos: Position(-1.6),
        target: Target(p0.to_outer()),
    });

    world.spawn(Entity::Inner(
        InnerEnemy {
            pos: Position(-1.67777),
            target: Target(p1.to_outer()),
        }
        .into_outer(),
    ));

    world.spawn::<InnerEntity>(
        InnerEnemy {
            pos: Position(-1.67777),
            target: Target(p1.to_outer()),
        }
        .into_outer(),
    );

    for p in world.query_mut::<&mut Position>() {
        dbg!(p.0);
        p.0 += 3.0;
    }

    println!(
        "entity get p0: {:?}",
        world.entity::<Player>(p1).unwrap().pos.0
    );

    world.entity_mut::<Player>(p1).unwrap().pos.0 += 2.0;

    println!(
        "entity get p0: {:?}",
        world.entity::<Player>(p1).unwrap().pos.0
    );

    println!("Position");
    for p in world.query_mut::<&Position>() {
        dbg!(p.0);
    }

    println!("Position, Velocity");
    for (p, v) in world.query_mut::<(&Position, &Velocity)>() {
        dbg!(p.0, v.0);
    }

    println!("mut Position, Velocity");
    for (p, v) in world.query_mut::<(&mut Position, &Velocity)>() {
        p.0 += v.0;
    }

    //println!("p0: {:?}", world.players.get_mut(p1).unwrap().pos.0);

    dbg!("--");

    /*
    while let Some((p, v, nest)) = world
        .stream::<(&mut Position, &Velocity)>()
        .nest::<&mut Position>()
    {
        for p in nest.iter() {}
    }
    */

    /*
    struct Link {}

    struct RopeNode {
        next: Option<(AnyEntityId<World>, f32)>,
    }

    struct RopeNodePair {
        a: AnyEntityId<World>,
        b: AnyEntityId<World>,
    }

    while let Some(((node, pos), join)) = world
        .stream::<(&mut RopeNode, &Position)>()
        .join::<(&mut RopeNode, &Position)>()
    {
        for (next_node, pos) in join.iter(node.next.into_iter()) {}
    }

    while let Some(((node, pos), (next_node, next_pos))) = world
        .stream::<(&RopeNode, &mut Position)>()
        .join_flat::<(&RopeNode, &mut Position)>(|(node, _)| node.next.into_iter())
    {}
    */

    println!("Position, Position");
    for (p, q) in world.query_mut::<(&Position, &Position)>() {
        dbg!(p.0, q.0);
    }

    println!("EntityId, Position");
    for (id, _) in world.query_mut::<(EntityId<Entity>, &Position)>() {
        dbg!(id);
    }

    println!("EntityId, Position, With<Target>");
    for (id, pos) in world
        .query_mut::<(EntityId<Entity>, &Position)>()
        .with::<&Target>()
    {
        dbg!(id, pos.0);
    }

    println!("EntityId, Position, Without<Target>");
    for (id, pos) in world
        .query_mut::<(EntityId<Entity>, &Position)>()
        .without::<&Target>()
    {
        dbg!(id, pos.0);
    }

    println!("EntityId, Target");
    for (id, target) in world.query_mut::<(EntityId<Entity>, &Target)>() {
        println!("{:?} targeting {:?}", id, target);
    }

    println!("EntityId, Target, nest with Position");
    for ((id, target), mut nest) in world
        .query_mut::<(EntityId<Entity>, &Target)>()
        .nest::<&mut Position>()
    {
        let Some(target_pos) = nest.get(target.0) else {
            continue;
        };
        /*let Some(target_pos_2) = nest.get(target.0) else {
            continue;
        };*/

        println!("{:?} targeting {:?} @ {:?}", id, target, target_pos.0);
        //println!("{:?} targeting {:?} @ {:?}", id, target, target_pos_2.0);
    }

    println!("EntityId, Target, nest with Position as EntityRefMut");

    for ((id, target), mut nest) in world
        .query_mut::<(EntityId<Entity>, &Target)>()
        .nest::<EntityRefMut<Player>>()
    {
        let Some(target_pos) = nest.get(target.0) else {
            continue;
        };
        /*let Some(target_pos_2) = nest.get(target.0) else {
            continue;
        };*/

        println!("{:?} targeting {:?} @ {:?}", id, target, target_pos.pos.0);
        //println!("{:?} targeting {:?} @ {:?}", id, target, target_pos_2.0);
    }

    println!("Target, nest with Position");
    /*for (target, mut nest) in world.query::<&Target>()
    .nest::<&mut Position>()
    .nest::<&mut Position>()

    for (target, mut nest) in world
        .query::<&Target>()
        .nest3::<(&mut Position, &mut Position)>()
    {
        let Some(target_pos) = nest.get(target.0, target.1) else {
            continue;
        };
        /*let Some(target_pos_2) = nest.get(target.0) else {
            continue;
        };*/

        println!("targeting {:?} @ {:?}", target, target_pos.0);
        //println!("targeting {:?} @ {:?}", target, target_pos_2.0);
    }*/

    println!("Target, nest with Position as EntityRefMut");

    for (target, mut nest) in world.query_mut::<&Target>().nest::<EntityRefMut<Player>>() {
        let Some(target_pos) = nest.get(target.0) else {
            continue;
        };
        /*let Some(target_pos_2) = nest.get(target.0) else {
            continue;
        };*/

        println!("targeting {:?} @ {:?}", target, target_pos.pos.0);
        //println!("targeting {:?} @ {:?}", target, target_pos_2.pos.0);
    }

    println!("Target, nest with Position as EntityRef");

    for (target, mut nest) in world.query_mut::<&Target>().nest::<EntityRef<Player>>() {
        let Some(target_pos) = nest.get(target.0) else {
            continue;
        };
        /*let Some(target_pos_2) = nest.get(target.0) else {
            continue;
        };*/

        println!("targeting {:?} @ {:?}", target, target_pos.pos.0);
        //println!("targeting {:?} @ {:?}", target, target_pos_2.pos.0);
    }

    println!("Enemies query");
    //let iter = world.enemies.iter_mut();

    for enemy in world.query_mut::<EntityRefMut<Enemy>>() {
        dbg!(enemy.target.0, enemy.pos.0);

        *enemy.pos = Position(enemy.pos.0 + 100.0);
    }

    println!("Enemies, Enemies query");
    for (enemy0, enemy1) in world.query_mut::<(EntityRef<Enemy>, EntityRef<Enemy>)>() {
        dbg!(enemy0.pos.0 - enemy1.pos.0);
    }

    for enemy in world.query_mut::<EntityRef<Enemy>>() {
        dbg!(enemy.target.0, enemy.pos.0);
    }

    println!("Any InnerEntity");
    for entity in world.query_mut::<EntityRef<InnerEntity>>() {
        println!("got some entity!!!");

        type Ref<'a> = EntityRef<'a, InnerEntity>;

        match entity {
            Ref::Enemy(_) => println!("enemy"),
            Ref::Boier(_) => println!("boier"),
        }
    }

    println!("Any Entity");
    for entity in world.query_mut::<EntityRef<Entity>>() {
        println!("got some entity!!!");

        type Ref<'a> = EntityRef<'a, Entity>;

        match entity {
            Ref::Player(_) => println!("player"),
            Ref::Enemy(_) => println!("enemy"),
            Ref::Boier(_) => println!("boier"),
            Ref::Inner(_) => println!("no way"),
        }
    }

    for key in world.query_mut::<EntityId<Enemy>>() {
        println!("got enemy: {:?}", key);
    }

    for id in world.query_mut::<EntityId<Entity>>() {
        println!("got id: {:?}", id);
    }

    // These panic:
    println!("mut Position, Position");
    for (p, q) in world.query_mut::<(&mut Position, &Position)>() {
        p.0 += q.0;
    }

    for (p, q) in world.query_mut::<(&mut Position, &mut Position)>() {
        p.0 += q.0;
    }

    for (enemy, pos) in world.query_mut::<(EntityRefMut<Enemy>, &Position)>() {
        dbg!(enemy.target.0, enemy.pos.0);

        *enemy.pos = Position(enemy.pos.0 + 100.0);
    }
}
