use stecs::{
    entity::EntityFetch, query::fetch::Fetch, Component, EntityId, EntityRef, EntityRefMut, Query,
    WorldData,
};

#[derive(Clone)]
struct Position(f32);

#[derive(Clone)]
struct Velocity(f32);

#[derive(Clone)]
struct Color(f32);

/*
#[derive(Entity)]
enum MyEntity {
    Player(Player),
    Enemy(Enemy),
    Foo {
        x: Position,
    },
}
*/

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
enum InnerEntity {
    Player(Player),
    Enemy(Enemy),
    Boier(Boier<Position, Velocity>),
}

#[derive(stecs::Entity, Clone)]
enum Entity {
    Inner(InnerEntity),
    Player(Player),
    Enemy(Enemy),
    Boier(Boier<Position, Velocity>),
}

type World = stecs::World<Entity>;

#[derive(Copy, Clone)]
enum InnerEntityFetchRef<'w> {
    Player(<Player as EntityFetch>::Fetch<'w>),
    Enemy(<Enemy as EntityFetch>::Fetch<'w>),
}

unsafe impl<'w> Fetch for InnerEntityFetchRef<'w> {
    type Item<'f> = InnerEntityStecsInternalRef<'f>
    where
        Self: 'f;

    fn new<A: stecs::entity::Columns>(
        ids: &stecs::column::Column<thunderdome::Index>,
        columns: &A,
    ) -> Option<Self> {
        use std::any::TypeId;

        if TypeId::of::<A::Entity>() == TypeId::of::<Player>() {
            Fetch::new(ids, columns).map(|fetch| InnerEntityFetchRef::Player(fetch))
        } else if TypeId::of::<A::Entity>() == TypeId::of::<Enemy>() {
            Fetch::new(ids, columns).map(|fetch| InnerEntityFetchRef::Enemy(fetch))
        } else {
            None
        }
    }

    fn len(&self) -> usize {
        match self {
            InnerEntityFetchRef::Player(fetch) => fetch.len(),
            InnerEntityFetchRef::Enemy(fetch) => fetch.len(),
        }
    }

    unsafe fn get<'f>(&self, index: usize) -> Self::Item<'f>
    where
        Self: 'f,
    {
        match self {
            InnerEntityFetchRef::Player(fetch) => {
                InnerEntityStecsInternalRef::Player(fetch.get(index))
            }
            InnerEntityFetchRef::Enemy(fetch) => {
                InnerEntityStecsInternalRef::Enemy(fetch.get(index))
            }
        }
    }

    fn check_borrows(checker: &mut stecs::query::borrow_checker::BorrowChecker) {
        <Player as EntityFetch>::Fetch::<'w>::check_borrows(checker);
        <Enemy as EntityFetch>::Fetch::<'w>::check_borrows(checker);
    }

    fn filter_by_outer<DOuter: WorldData>(fetch: &mut Option<Self>) {
        use std::any::TypeId;

        if TypeId::of::<DOuter>() != TypeId::of::<InnerEntity>() {
            *fetch = None;
        }
    }
}

#[derive(Copy, Clone)]
enum InnerEntityFetchRefMut<'w> {
    Player(<Player as EntityFetch>::FetchMut<'w>),
    Enemy(<Enemy as EntityFetch>::FetchMut<'w>),
}

unsafe impl<'w> Fetch for InnerEntityFetchRefMut<'w> {
    type Item<'f> = InnerEntityStecsInternalRefMut<'f>
    where
        Self: 'f;

    fn new<A: stecs::entity::Columns>(
        ids: &stecs::column::Column<thunderdome::Index>,
        columns: &A,
    ) -> Option<Self> {
        use std::any::TypeId;

        if TypeId::of::<A::Entity>() == TypeId::of::<Player>() {
            Fetch::new(ids, columns).map(|fetch| InnerEntityFetchRefMut::Player(fetch))
        } else if TypeId::of::<A::Entity>() == TypeId::of::<Player>() {
            Fetch::new(ids, columns).map(|fetch| InnerEntityFetchRefMut::Enemy(fetch))
        } else {
            None
        }
    }

    fn len(&self) -> usize {
        match self {
            InnerEntityFetchRefMut::Player(fetch) => fetch.len(),
            InnerEntityFetchRefMut::Enemy(fetch) => fetch.len(),
        }
    }

    unsafe fn get<'f>(&self, index: usize) -> Self::Item<'f>
    where
        Self: 'f,
    {
        match self {
            InnerEntityFetchRefMut::Player(fetch) => {
                InnerEntityStecsInternalRefMut::Player(fetch.get(index))
            }
            InnerEntityFetchRefMut::Enemy(fetch) => {
                InnerEntityStecsInternalRefMut::Enemy(fetch.get(index))
            }
        }
    }

    fn check_borrows(checker: &mut stecs::query::borrow_checker::BorrowChecker) {
        <Player as EntityFetch>::FetchMut::<'w>::check_borrows(checker);
        <Enemy as EntityFetch>::FetchMut::<'w>::check_borrows(checker);
    }
}

impl EntityFetch for InnerEntity {
    type Fetch<'w> = InnerEntityFetchRef<'w>;

    type FetchMut<'w> = InnerEntityFetchRefMut<'w>;
}

impl<'q> Query for InnerEntityStecsInternalRef<'q> {
    type Fetch<'w> = InnerEntityFetchRef<'w>;
}

fn main() {
    let mut world = World::default();

    let p0 = world.spawn(Player {
        pos: Position(1.0),
        vel: Velocity(2.0),
        col: Color(3.0),
    });

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

    for p in world.query::<&mut Position>() {
        dbg!(p.0);
        p.0 += 3.0;
    }

    // /println!("p0: {:?}", world.entity::<Player>(p1).unwrap().pos.0);

    println!("Position");
    for p in world.query::<&Position>() {
        dbg!(p.0);
    }

    println!("Position, Velocity");
    for (p, v) in world.query::<(&Position, &Velocity)>() {
        dbg!(p.0, v.0);
    }

    println!("mut Position, Velocity");
    for (p, v) in world.query::<(&mut Position, &Velocity)>() {
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
    for (p, q) in world.query::<(&Position, &Position)>() {
        dbg!(p.0, q.0);
    }

    /*println!("EntityId, Position");
    for (id, _) in world.query::<(AnyEntityId<World>, &Position)>() {
        dbg!(id);
    }

    println!("EntityId, Position, With<Target>");
    for (id, pos) in world
        .query::<(AnyEntityId<World>, &Position)>()
        .with::<&Target>()
    {
        dbg!(id, pos.0);
    }

    println!("EntityId, Position, Without<Target>");
    for (id, pos) in world
        .query::<(AnyEntityId<World>, &Position)>()
        .without::<&Target>()
    {
        dbg!(id, pos.0);
    }

    println!("EntityId, Target");
    for (id, target) in world.query::<(AnyEntityId<World>, &Target)>() {
        println!("{:?} targeting {:?}", id, target);
    }

    println!("EntityId, Target, nest with Position");
    for ((id, target), mut nest) in world
        .query::<(AnyEntityId<World>, &Target)>()
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
        .query::<(AnyEntityId<World>, &Target)>()
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
    */

    println!("Target, nest with Position");
    for (target, mut nest) in world.query::<&Target>().nest::<&mut Position>() {
        let Some(target_pos) = nest.get(target.0) else {
            continue;
        };
        /*let Some(target_pos_2) = nest.get(target.0) else {
            continue;
        };*/

        println!("targeting {:?} @ {:?}", target, target_pos.0);
        //println!("{:?} targeting {:?} @ {:?}", id, target, target_pos_2.0);
    }

    println!("Target, nest with Position as EntityRefMut");

    for (target, mut nest) in world.query::<&Target>().nest::<EntityRefMut<Player>>() {
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

    for (target, mut nest) in world.query::<&Target>().nest::<EntityRef<Player>>() {
        let Some(target_pos) = nest.get(target.0) else {
            continue;
        };
        /*let Some(target_pos_2) = nest.get(target.0) else {
            continue;
        };*/

        println!("targeting {:?} @ {:?}", target, target_pos.pos.0);
        //println!("{:?} targeting {:?} @ {:?}", id, target, target_pos_2.0);
    }

    println!("Enemies query");
    //let iter = world.enemies.iter_mut();

    for enemy in world.query::<EntityRefMut<Enemy>>() {
        dbg!(enemy.target.0, enemy.pos.0);

        *enemy.pos = Position(enemy.pos.0 + 100.0);
    }

    println!("Enemies, Enemies query");
    for (enemy0, enemy1) in world.query::<(EntityRef<Enemy>, EntityRef<Enemy>)>() {
        dbg!(enemy0.pos.0 - enemy1.pos.0);
    }

    for enemy in world.query::<EntityRef<Enemy>>() {
        dbg!(enemy.target.0, enemy.pos.0);
    }

    println!("Any Entity");
    for entity in world.query::<EntityRef<InnerEntity>>() {
        println!("got some entity!!!");

        type Ref<'a> = EntityRef<'a, InnerEntity>;

        match entity {
            Ref::Player(_) => println!("player"),
            Ref::Enemy(_) => println!("enemy"),
            Ref::Boier(_) => println!("boier"),
        }
    }

    // Panics
    /*
    println!("Make miri sad");
    for (enemy, pos) in world.query::<(EntityRefMut<Enemy>, &Position)>() {
        dbg!(enemy.target.0, enemy.pos.0);

        *enemy.pos = Position(enemy.pos.0 + 100.0);
    }
    */

    /*
    let foo: Vec<_> = world
        .query::<&Target>()
        .nest::<&mut Position>()
        .into_iter()
        .map(|(target, mut nest)| nest.get(target.0))
        .collect();
    */

    /*
    let id: EntityId<MyWorld> = todo!();

    match id {
        EntityId::<MyWorld>::Player(_) => todo!(),
        WorldEntityId::Enemy(_) => todo!(),
    }
    */

    /*
    println!("Enemies");
    let iter = world.enemies.iter_mut();

    for (key, enemy) in iter {
        dbg!(key, enemy.target.0, enemy.pos.0);

        *enemy.pos = Position(enemy.pos.0 + 100.0);
    }

    for (key, enemy) in world.enemies.iter() {
        dbg!(key, enemy.target.0, enemy.pos.0);
    }

    */

    // This panics:
    println!("mut Position, Position");
    for (p, q) in world.query::<(&mut Position, &Position)>() {
        p.0 += q.0;
    }

    for (p, q) in world.query::<(&mut Position, &mut Position)>() {
        p.0 += q.0;
    }
}
