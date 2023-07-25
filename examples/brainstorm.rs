use std::any::type_name;

use stecs::{
    archetype_set::{ArchetypeSetFetch, InArchetypeSet},
    internal::BorrowChecker,
    query::fetch::{FetchEntityId, FetchFromSet},
    Archetype, ArchetypeSet, Entity, EntityId, EntityKey, Query,
};
use thunderdome::Arena;

#[derive(Clone)]
struct Position(f32);

#[derive(Clone)]
struct Velocity(f32);

#[derive(Clone)]
struct Color(f32);

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
#[derive(Default)]
struct World {
    players: Archetype<Player>,
    enemies: Archetype<Enemy>,
}

// generated
#[derive(Clone, Copy, Debug, PartialEq)]
enum WorldEntityId {
    Player(EntityKey<Player>),
    Enemy(EntityKey<Enemy>),
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
    F: FetchFromSet<World>,
{
    type Fetch = F;

    type Iter = std::iter::Flatten<std::array::IntoIter<Option<F>, 2>>;

    unsafe fn get<'b>(&self, id: EntityId<World>) -> Option<F::Item<'b>> {
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
    type EntityId = WorldEntityId;

    type Entity = WorldEntity;

    type Fetch<'w, F: FetchFromSet<Self> + 'w> = WorldFetch<'w, F>;

    fn spawn<E: InArchetypeSet<Self>>(&mut self, entity: E) -> Self::EntityId {
        match entity.into_entity() {
            WorldEntity::Player(entity) => WorldEntityId::Player(self.players.spawn(entity)),
            WorldEntity::Enemy(entity) => WorldEntityId::Enemy(self.enemies.spawn(entity)),
        }
    }

    fn despawn(&mut self, id: Self::EntityId) -> Option<Self::Entity> {
        match id {
            WorldEntityId::Player(key) => self.players.despawn(key).map(WorldEntity::Player),
            WorldEntityId::Enemy(key) => self.enemies.despawn(key).map(WorldEntity::Enemy),
        }
    }

    fn fetch<'w, F>(&'w self) -> Self::Fetch<'w, F>
    where
        F: FetchFromSet<Self> + 'w,
    {
        let players = F::new::<Player>(self.players.untyped_keys(), self.players.columns())
            .map(|fetch| (self.players.indices(), fetch));
        let enemies = F::new::<Enemy>(self.enemies.untyped_keys(), self.enemies.columns())
            .map(|fetch| (self.enemies.indices(), fetch));

        WorldFetch { players, enemies }
    }
}

impl InArchetypeSet<World> for Player {
    fn untyped_key_to_key(key: thunderdome::Index) -> EntityKey<Self> {
        EntityKey::new_unchecked(key)
    }

    fn key_to_id(key: EntityKey<Self>) -> EntityId<World> {
        EntityId::<World>::Player(key)
    }

    fn into_entity(self) -> <World as ArchetypeSet>::Entity {
        WorldEntity::Player(self)
    }
}

impl InArchetypeSet<World> for Enemy {
    fn untyped_key_to_key(key: thunderdome::Index) -> EntityKey<Self> {
        EntityKey::new_unchecked(key)
    }

    fn key_to_id(key: EntityKey<Self>) -> EntityId<World> {
        EntityId::<World>::Enemy(key)
    }

    fn into_entity(self) -> <World as ArchetypeSet>::Entity {
        WorldEntity::Enemy(self)
    }
}

impl Query<World> for WorldEntityId {
    type Fetch<'f> = FetchEntityId<World>;

    fn check_borrows(checker: &mut BorrowChecker) {}
}

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
    while let Some((p, v, join)) = world
        .stream::<(&mut Position, &Velocity)>()
        .join::<&mut Position>()
    {
        for p in join.iter() {}
    }
    */

    struct Link {}

    struct RopeNode {
        next: Option<(EntityId<World>, f32)>,
    }

    struct RopeNodePair {
        a: EntityId<World>,
        b: EntityId<World>,
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

    println!("EntityId, Position");
    for (id, _) in world.query::<(EntityId<World>, &Position)>() {
        dbg!(id);
    }

    println!("EntityId, Position, With<Target>");
    for (id, pos) in world
        .query::<(EntityId<World>, &Position)>()
        .with::<&Target>()
    {
        dbg!(id, pos.0);
    }

    println!("EntityId, Position, Without<Target>");
    for (id, pos) in world
        .query::<(EntityId<World>, &Position)>()
        .without::<&Target>()
    {
        dbg!(id, pos.0);
    }

    println!("EntityId, Target");
    for (id, target) in world.query::<(EntityId<World>, &Target)>() {
        println!("{:?} targeting {:?}", id, target);
    }

    println!("EntityId, Target, join with Position");
    for ((id, target), mut join) in world
        .query::<(EntityId<World>, &Target)>()
        .join::<&mut Position>()
    {
        let Some(target_pos) = join.get(target.0) else {
            continue;
        };
        /*let Some(target_pos_2) = join.get(target.0) else {
            continue;
        };*/

        println!("{:?} targeting {:?} @ {:?}", id, target, target_pos.0);
        //println!("{:?} targeting {:?} @ {:?}", id, target, target_pos_2.0);
    }

    /*
    let foo: Vec<_> = world
        .query::<&Target>()
        .join::<&mut Position>()
        .into_iter()
        .map(|(target, mut join)| join.get(target.0))
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

    // This panics:
    println!("mut Position, Position");
    for (p, q) in world.query::<(&mut Position, &Position)>() {
        p.0 += q.0;
    }
}
