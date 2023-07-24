use std::{
    any::{type_name, Any, TypeId},
    cell::RefCell,
};

use stecs::{
    internal::{BorrowChecker, FetchEntityId},
    Archetype, ArchetypeSet, ArchetypeSetFetch, Column, Component, Entity, EntityColumns, EntityId,
    EntityKey, Fetch, InArchetypeSet, Query,
};
use thunderdome::Arena;

#[derive(Clone)]
struct Position(f32);

#[derive(Clone)]
struct Velocity(f32);

#[derive(Clone)]
struct Color(f32);

#[derive(Clone)]
struct Player {
    pos: Position,
    vel: Velocity,
    col: Color,
}

// generated
#[derive(Default, Clone)]
struct PlayerColumns {
    pos: RefCell<Column<Position>>,
    vel: RefCell<Column<Velocity>>,
    col: RefCell<Column<Color>>,
}

impl EntityColumns for PlayerColumns {
    type Entity = Player;

    fn column<C: Component>(&self) -> Option<&RefCell<Column<C>>> {
        if TypeId::of::<C>() == TypeId::of::<Position>() {
            (&self.pos as &dyn Any).downcast_ref::<RefCell<Column<C>>>()
        } else if TypeId::of::<C>() == TypeId::of::<Velocity>() {
            (&self.vel as &dyn Any).downcast_ref::<RefCell<Column<C>>>()
        } else if TypeId::of::<C>() == TypeId::of::<Color>() {
            (&self.col as &dyn Any).downcast_ref::<RefCell<Column<C>>>()
        } else {
            None
        }
    }

    fn push(&mut self, entity: Self::Entity) {
        self.pos.borrow_mut().push(entity.pos);
        self.vel.borrow_mut().push(entity.vel);
    }

    fn remove(&mut self, index: usize) -> Self::Entity {
        Player {
            pos: self.pos.borrow_mut().remove(index),
            vel: self.vel.borrow_mut().remove(index),
            col: self.col.borrow_mut().remove(index),
        }
    }
}

impl Entity for Player {
    type Columns = PlayerColumns;
}

#[derive(Clone, Debug)]
struct Target(EntityId<World>);

#[derive(Clone)]
struct Enemy {
    pos: Position,
    target: Target,
}

// generated
#[derive(Default, Clone)]
struct EnemyColumns {
    pos: RefCell<Column<Position>>,
    target: RefCell<Column<Target>>,
}

impl EntityColumns for EnemyColumns {
    type Entity = Enemy;

    fn column<C: Component>(&self) -> Option<&RefCell<Column<C>>> {
        if TypeId::of::<C>() == TypeId::of::<Position>() {
            (&self.pos as &dyn Any).downcast_ref::<RefCell<Column<C>>>()
        } else if TypeId::of::<C>() == TypeId::of::<Target>() {
            (&self.target as &dyn Any).downcast_ref::<RefCell<Column<C>>>()
        } else {
            None
        }
    }

    fn push(&mut self, entity: Self::Entity) {
        self.pos.borrow_mut().push(entity.pos);
        self.target.borrow_mut().push(entity.target);
    }

    fn remove(&mut self, index: usize) -> Self::Entity {
        Enemy {
            pos: self.pos.borrow_mut().remove(index),
            target: self.target.borrow_mut().remove(index),
        }
    }
}

impl Entity for Enemy {
    type Columns = EnemyColumns;
}

#[derive(Default, Clone)]
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

impl<'a, F> ArchetypeSetFetch<'a, World> for WorldFetch<'a, F>
where
    F: Fetch<'a, World>,
{
    type Fetch = F;

    type Iter = std::iter::Flatten<std::array::IntoIter<Option<F>, 2>>;

    unsafe fn get<'b>(&self, id: EntityId<World>) -> Option<F::Item<'b>> {
        println!("getting {:?} {}", id, type_name::<F>());
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

    type Fetch<'a, F: Fetch<'a, Self>> = WorldFetch<'a, F>;

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

    fn fetch<'a, F>(&'a self) -> Self::Fetch<'a, F>
    where
        F: Fetch<'a, Self>,
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

impl<'a> Query<'a, World> for WorldEntityId {
    type Fetch = FetchEntityId<WorldEntityId>;

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

    world.spawn(Player {
        pos: Position(1.5),
        vel: Velocity(2.0),
        col: Color(3.0),
    });

    world.spawn(Enemy {
        pos: Position(-1.5),
        target: Target(p0),
    });

    world.spawn(Enemy {
        pos: Position(-1.5),
        target: Target(p0),
    });

    for p in world.query::<&mut Position>() {
        dbg!(p.0);
        p.0 += 3.0;
    }

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

        println!("{:?} targeting {:?} @ {:?}", id, target, target_pos.0);

        // FIXME
        let violation: Vec<&mut Position> = join.iter([target.0, target.0].into_iter()).collect();

        println!(
            "{} {:?} {:?}",
            violation.len(),
            violation[0] as *const _,
            violation[1] as *const _
        );
    }

    /*
    let id: EntityId<MyWorld> = todo!();

    match id {
        EntityId::<MyWorld>::Player(_) => todo!(),
        WorldEntityId::Enemy(_) => todo!(),
    }
    */

    // This panics:
    println!("mut Position, Position");
    for (p, q) in world.query::<(&mut Position, &Position)>() {
        p.0 += q.0;
    }
}
