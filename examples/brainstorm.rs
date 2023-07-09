use std::{any::TypeId, fmt::Debug, iter::Chain};

use frunk::{HCons, HNil};
use stecs::{Archetype, Arena, EntityId, EntityIndex, GetterIter, Query, World};

struct Position(f32);
struct Velocity(f32);
struct Color(f32);

struct Player {
    pos: Position,
    vel: Velocity,
    col: Color,
}

// generated
unsafe impl Archetype for Player {
    type Components = HCons<Position, HCons<Velocity, HCons<Color, HNil>>>;

    fn offset_of<C: stecs::Component>() -> Option<usize> {
        if TypeId::of::<C>() == TypeId::of::<Position>() {
            Some(memoffset::offset_of!(Player, pos))
        } else {
            None
        }
    }
}

struct Enemy {
    pos: Position,
}

// generated
unsafe impl Archetype for Enemy {
    type Components = HCons<Position, HNil>;

    fn offset_of<C: stecs::Component>() -> Option<usize> {
        if TypeId::of::<C>() == TypeId::of::<Position>() {
            Some(memoffset::offset_of!(Player, pos))
        } else {
            None
        }
    }
}

#[derive(Default)]
struct MyWorld {
    players: Arena<Player>,
    enemies: Arena<Enemy>,
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

    type QueryIter<'a, Q: Query<'a>> =
        Chain<GetterIter<'a, Player, Q::Getter<Player>>, GetterIter<'a, Enemy, Q::Getter<Enemy>>>;

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
        Q: stecs::Query<'a>,
    {
        GetterIter::new(self.players.iter_mut(), Q::getter::<Player>()).chain(GetterIter::new(
            self.enemies.iter_mut(),
            Q::getter::<Enemy>(),
        ))
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

    world.spawn(Enemy {
        pos: Position(-1.5),
    });

    for p in world.query::<&mut Position>() {
        dbg!(p.0);
        p.0 += 3.0;
    }

    dbg!("--");

    for p in world.query::<&Position>() {
        dbg!(p.0);
    }

    /*
    let id: EntityId<MyWorld> = todo!();

    match id {
        EntityId::<MyWorld>::Player(_) => todo!(),
        WorldEntityId::Enemy(_) => todo!(),
    }
    */
}
