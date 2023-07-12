use std::{any::TypeId, fmt::Debug, iter::Chain};

use frunk::{HCons, HNil};
use stecs::{
    arena, Archetype, ArchetypeInSet, ArchetypeSet, Arena, BorrowChecker, Entity, EntityId,
    EntityIdGetter, GetterIter, Query,
};

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
unsafe impl Archetype for Player {
    type Components = HCons<Position, HCons<Velocity, HCons<Color, HNil>>>;

    fn offset_of<C: stecs::Component>() -> Option<usize> {
        let type_id = TypeId::of::<C>();

        if type_id == TypeId::of::<Position>() {
            Some(memoffset::offset_of!(Player, pos))
        } else if type_id == TypeId::of::<Velocity>() {
            Some(memoffset::offset_of!(Player, vel))
        } else if type_id == TypeId::of::<Color>() {
            Some(memoffset::offset_of!(Player, col))
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
struct Target(EntityId<MyWorld>);

#[derive(Clone)]
struct Enemy {
    pos: Position,
    target: Target,
}

// generated
unsafe impl Archetype for Enemy {
    type Components = HCons<Position, HNil>;

    fn offset_of<C: stecs::Component>() -> Option<usize> {
        let type_id = TypeId::of::<C>();

        if type_id == TypeId::of::<Position>() {
            Some(memoffset::offset_of!(Enemy, pos))
        } else if type_id == TypeId::of::<Target>() {
            Some(memoffset::offset_of!(Enemy, target))
        } else {
            None
        }
    }
}

#[derive(Default, Clone)]
struct MyWorld {
    players: Arena<Player>,
    enemies: Arena<Enemy>,
}

// generated
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
enum WorldEntityId {
    Player(arena::Index),
    Enemy(arena::Index),
}

enum WorldAnyEntity {
    Player(Player),
    Enemy(Enemy),
}

impl From<Player> for WorldAnyEntity {
    fn from(entity: Player) -> Self {
        Self::Player(entity)
    }
}

impl From<Enemy> for WorldAnyEntity {
    fn from(entity: Enemy) -> Self {
        Self::Enemy(entity)
    }
}

impl stecs::ArchetypeSet for MyWorld {
    type EntityId = WorldEntityId;

    type Entity = WorldAnyEntity;

    type QueryIter<'a, Q: Query<'a, Self>> = Chain<
        GetterIter<'a, Self, Player, Q::Getter<Player>>,
        GetterIter<'a, Self, Enemy, Q::Getter<Enemy>>,
    >;

    fn spawn<A: ArchetypeInSet<Self>>(&mut self, entity: A) -> Self::EntityId {
        match entity.into_any() {
            WorldAnyEntity::Player(entity) => WorldEntityId::Player(self.players.insert(entity)),
            WorldAnyEntity::Enemy(entity) => WorldEntityId::Enemy(self.enemies.insert(entity)),
        }
    }

    fn despawn(&mut self, id: Self::EntityId) -> Option<Self::Entity> {
        todo!()
    }

    fn query<'a, Q>(&'a mut self) -> Self::QueryIter<'a, Q>
    where
        Q: Query<'a, Self>,
    {
        let iter = GetterIter::new::<Q>(self.players.iter_mut());
        let iter = iter.chain(GetterIter::new::<Q>(self.enemies.iter_mut()));

        iter
    }
}

impl ArchetypeInSet<MyWorld> for Player {
    fn id(index: arena::Index) -> EntityId<MyWorld> {
        EntityId::<MyWorld>::Player(index)
    }

    fn into_any(self) -> Entity<MyWorld> {
        Entity::<MyWorld>::Player(self)
    }
}

impl ArchetypeInSet<MyWorld> for Enemy {
    fn id(index: arena::Index) -> EntityId<MyWorld> {
        EntityId::<MyWorld>::Enemy(index)
    }

    fn into_any(self) -> Entity<MyWorld> {
        Entity::<MyWorld>::Enemy(self)
    }
}

impl<'a> Query<'a, MyWorld> for WorldEntityId {
    type Getter<A> = EntityIdGetter
    where
        A: 'a + ArchetypeInSet<MyWorld>;

    fn check_borrows(_: &mut BorrowChecker) {}

    fn getter<A>() -> Option<Self::Getter<A>>
    where
        A: 'a + ArchetypeInSet<MyWorld>,
    {
        Some(EntityIdGetter)
    }
}

fn main() {
    //let id = EntityId::<World>::Player(0);

    let mut world = MyWorld::default();

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

    dbg!("--");

    for p in world.query::<&Position>() {
        dbg!(p.0);
    }

    dbg!("--");

    for (p, v) in world.query::<(&Position, &Velocity)>() {
        dbg!(p.0, v.0);
    }

    dbg!("--");

    for (p, v) in world.query::<(&mut Position, &Velocity)>() {
        p.0 += v.0;
    }

    dbg!("--");

    /*for (p, q) in world.query::<(&mut Position, &mut Position)>() {
        p.0 += q.0;
    }*/

    /*for (p, q) in world.query::<(&mut Position, &Position)>() {
        p.0 += q.0;
    }*/

    for (p, q) in world.query::<(&Position, &Position)>() {}

    dbg!("--");

    for (id, _) in world.query::<(EntityId<MyWorld>, &Position)>() {
        dbg!(id);
    }

    dbg!("--");

    for (id, target) in world.query::<(EntityId<MyWorld>, &Target)>() {
        println!("{:?} targeting {:?}", id, target);
    }

    /*
    let id: EntityId<MyWorld> = todo!();

    match id {
        EntityId::<MyWorld>::Player(_) => todo!(),
        WorldEntityId::Enemy(_) => todo!(),
    }
    */
}
