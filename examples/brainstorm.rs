use stecs::{query::fetch::Fetch, ArchetypeSet, Entity, EntityId, EntityRef, EntityRefMut};
use thunderdome::Arena;

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

#[derive(Entity, Clone)]
struct Player {
    pos: Position,
    vel: Velocity,
    col: Color,
}

#[derive(Entity, Clone)]
struct Boier<T, S> {
    pos: T,
    vel: S,
    col: Color,
}

/*
#[derive(Clone, Debug)]
struct Target(EntityId<World>);

#[derive(Entity, Clone)]
struct Enemy {
    pos: Position,
    target: Target,
}

#[derive(Entity, Clone)]
struct Blob;

#[derive(Entity, Clone)]
struct Blub(u32);

// TODO: Clone in Derive?
#[derive(Default, ArchetypeSet)]
struct World {
    players: Archetype<Player>,
    enemies: Archetype<Enemy>,
}

// generated
#[derive(Clone, Copy, Debug, PartialEq)]
enum WorldEntityId {
    Player(EntityId<Player>),
    Enemy(EntityId<Enemy>),
}

enum WorldEntity {
    Player(Player),
    Enemy(Enemy),
}

impl From<Player> for WorldEntity {
    fn from(entity: Player) -> Self {
        Self::Player(entity)
    }
}

impl From<Enemy> for WorldEntity {
    fn from(entity: Enemy) -> Self {
        Self::Enemy(entity)
    }
}

#[derive(Clone)]
struct WorldFetch<'a, F> {
    players: Option<(&'a Arena<usize>, F)>,
    enemies: Option<(&'a Arena<usize>, F)>,
}

impl<'a, F> ArchetypeSetFetch<World> for WorldFetch<'a, F>
where
    F: Fetch,
{
    type Fetch = F;

    type Iter = std::iter::Flatten<std::array::IntoIter<Option<F>, 2>>;

    unsafe fn get<'b>(&self, id: AnyEntityId<World>) -> Option<F::Item<'b>> {
        match id {
            WorldEntityId::Player(key) => self
                .players
                .as_ref()
                .and_then(|(arena, fetch)| arena.get(key.0).map(|&index| fetch.get(index))),
            WorldEntityId::Enemy(key) => self
                .enemies
                .as_ref()
                .and_then(|(arena, fetch)| arena.get(key.0).map(|&index| fetch.get(index))),
        }
    }

    fn iter(&mut self) -> Self::Iter {
        [
            self.players.as_ref().map(|(_, fetch)| *fetch),
            self.enemies.as_ref().map(|(_, fetch)| *fetch),
        ]
        .into_iter()
        .flatten()
    }
}

impl stecs::ArchetypeSet for World {
    type AnyEntityId = WorldEntityId;

    type AnyEntity = WorldEntity;

    type Fetch<'w, F: Fetch + 'w> = WorldFetch<'w, F>;

    fn spawn<E>(&mut self, entity: E) -> Self::AnyEntityId {
        match entity {
            WorldEntity::Player(entity) => WorldEntityId::Player(self.players.spawn(entity)),
            WorldEntity::Enemy(entity) => WorldEntityId::Enemy(self.enemies.spawn(entity)),
        }
    }

    fn despawn(&mut self, id: Self::AnyEntityId) -> Option<Self::AnyEntity> {
        match id {
            WorldEntityId::Player(key) => self.players.despawn(key).map(WorldEntity::Player),
            WorldEntityId::Enemy(key) => self.enemies.despawn(key).map(WorldEntity::Enemy),
        }
    }

    fn fetch<'w, F>(&'w self) -> Self::Fetch<'w, F>
    where
        F: Fetch + 'w,
    {
        let players = F::new(self.players.ids(), self.players.columns())
            .map(|fetch| (self.players.indices(), fetch));
        let enemies = F::new(self.enemies.ids(), self.enemies.columns())
            .map(|fetch| (self.enemies.indices(), fetch));

        WorldFetch { players, enemies }
    }
}

impl InArchetypeSet<World> for Player {
    fn embed_entity(self) -> WorldEntity {
        WorldEntity::Player(self)
    }
}

impl SubArchetypeSet<World> for Archetype<Player> {
    fn embed_entity_id(id: EntityId<Player>) -> WorldEntityId {
        WorldEntityId::Player(id)
    }
}

impl InArchetypeSet<World> for Enemy {
    fn embed_entity(self) -> WorldEntity {
        WorldEntity::Enemy(self)
    }
}

impl SubArchetypeSet<World> for Archetype<Enemy> {
    fn embed_entity_id(id: EntityId<Enemy>) -> WorldEntityId {
        WorldEntityId::Enemy(id)
    }
}

/*impl Query<World> for WorldEntityId {
    type Fetch<'f> = FetchAnyEntityId<World>;
}*/

fn main() {
    //let id = EntityId::<World>::Player(0);

    let mut world = World::default();

    let p0 = world.spawn(Player {
        pos: Position(1.0),
        vel: Velocity(2.0),
        col: Color(3.0),
    });

    let p1 = world.players.spawn(Player {
        pos: Position(1.5),
        vel: Velocity(2.0),
        col: Color(3.0),
    });

    world.spawn(Enemy {
        pos: Position(-1.5),
        target: Target(p0),
    });

    world.spawn(Enemy {
        pos: Position(-1.6),
        target: Target(p0),
    });

    for p in world.query::<&mut Position>() {
        dbg!(p.0);
        p.0 += 3.0;
    }

    println!("p0: {:?}", world.players.get_mut(p1).unwrap().pos.0);

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

    println!("p0: {:?}", world.players.get_mut(p1).unwrap().pos.0);

    dbg!("--");

    /*
    while let Some((p, v, nest)) = world
        .stream::<(&mut Position, &Velocity)>()
        .nest::<&mut Position>()
    {
        for p in nest.iter() {}
    }
    */

    struct Link {}

    struct RopeNode {
        next: Option<(AnyEntityId<World>, f32)>,
    }

    struct RopeNodePair {
        a: AnyEntityId<World>,
        b: AnyEntityId<World>,
    }

    /*
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

    /*for (p, q) in world.query::<(&mut Position, &mut Position)>() {
        p.0 += q.0;
    }*/

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
    for (target, mut nest) in world.query::<(&Target)>().nest::<&mut Position>() {
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

    for (target, mut nest) in world.query::<(&Target)>().nest::<EntityRefMut<Player>>() {
        let Some(target_pos) = nest.get(target.0) else {
            continue;
        };
        /*let Some(target_pos_2) = nest.get(target.0) else {
            continue;
        };*/

        println!("targeting {:?} @ {:?}", target, target_pos.pos.0);
        //println!("{:?} targeting {:?} @ {:?}", id, target, target_pos_2.0);
    }

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

    println!("Enemies");
    let iter = world.enemies.iter_mut();

    for (key, enemy) in iter {
        dbg!(key, enemy.target.0, enemy.pos.0);

        *enemy.pos = Position(enemy.pos.0 + 100.0);
    }

    for (key, enemy) in world.enemies.iter() {
        dbg!(key, enemy.target.0, enemy.pos.0);
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

    println!("Make miri sad");

    for (enemy, pos) in world.query::<(EntityRefMut<Enemy>, &Position)>() {
        dbg!(enemy.target.0, enemy.pos.0);

        *enemy.pos = Position(enemy.pos.0 + 100.0);
    }

    // This panics:
    println!("mut Position, Position");
    for (p, q) in world.query::<(&mut Position, &Position)>() {
        p.0 += q.0;
    }
}

*/

fn main() {}
