use std::{
    any::Any,
    cell::RefCell,
    fmt::Debug,
    iter::{Chain, Flatten},
};

use frunk::{HCons, HNil};
use stecs::{Archetype, Column, EntityId, EntityIndex, Query, Storage, World};

struct Position(f32);
struct Velocity(f32);
struct Color(f32);

struct Player {
    pos: Position,
    vel: Velocity,
    col: Color,
}

// generated
#[derive(Default)]
struct PlayerStorage {
    pos: Column<Position>,
    vel: Column<Velocity>,
    col: Column<Color>,
}

impl Archetype for Player {
    type Components = HCons<Position, HCons<Velocity, HCons<Velocity, HCons<Color, HNil>>>>;

    type Storage = PlayerStorage;

    fn column<'a, C: stecs::Component>(
        storage: &'a Self::Storage,
    ) -> Option<&'a RefCell<Column<C>>> {
        let any: &dyn Any = &storage.pos;

        any.downcast_ref()
    }

    fn insert(storage: &mut Self::Storage, entity: Self) -> EntityIndex {
        storage.pos.borrow_mut().insert(entity.pos);
        storage.vel.borrow_mut().insert(entity.vel);
        storage.col.borrow_mut().insert(entity.col)
    }
}

struct Enemy {
    pos: Position,
}

// generated
#[derive(Default)]
struct EnemyStorage {
    pos: RefCell<Column<Position>>,
}

impl Archetype for Enemy {
    type Components = HCons<Position, HNil>;

    type Storage = EnemyStorage;

    fn column<'a, C: 'a + stecs::Component>(
        storage: &'a Self::Storage,
    ) -> Option<&'a RefCell<Column<C>>> {
        todo!()
    }

    fn insert(storage: &mut Self::Storage, entity: Self) -> EntityIndex {
        storage.pos.borrow_mut().insert(entity.pos)
    }
}

#[derive(Default)]
struct MyWorld {
    players: Storage<Player>,
    enemies: Storage<Enemy>,
}

// generated
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
enum WorldEntityId {
    Player(EntityIndex),
    Enemy(EntityIndex),
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

impl stecs::World for MyWorld {
    type EntityId = WorldEntityId;

    type Entity = WorldEntity;

    type QueryIter<'a, Q: Query> = Chain<
        Flatten<std::option::IntoIter<Q::Iter<'a>>>,
        Flatten<std::option::IntoIter<Q::Iter<'a>>>>
        where Q: 'a;

    fn spawn(&mut self, entity: impl Into<Self::Entity>) -> Self::EntityId {
        match entity.into() {
            WorldEntity::Player(entity) => WorldEntityId::Player(self.players.insert(entity)),
            WorldEntity::Enemy(entity) => WorldEntityId::Enemy(self.enemies.insert(entity)),
        }
    }

    fn despawn(&mut self, id: Self::EntityId) -> Option<Self::Entity> {
        todo!()
    }

    fn query<'a, Q>(&'a mut self) -> Self::QueryIter<'a, Q>
    where
        Q: stecs::Query,
    {
        Q::query(&self.players)
            .into_iter()
            .flatten()
            .chain(Q::query(&self.enemies).into_iter().flatten())
    }
}

fn main() {
    //let id = EntityId::<World>::Player(0);

    let mut world = MyWorld::default();

    world.spawn(Player {
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
    });

    for p in world.query::<&Position>() {}

    let id: EntityId<MyWorld> = todo!();

    match id {
        EntityId::<MyWorld>::Player(_) => todo!(),
        WorldEntityId::Enemy(_) => todo!(),
    }
}
