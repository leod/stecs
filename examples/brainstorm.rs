use std::{
    any::{Any, TypeId},
    iter::{Chain, Flatten},
    option::IntoIter,
};

use stecs::{
    Archetype, ArchetypeSet, Column, Component, Entity, EntityColumns, EntityId, EntityKey,
    InArchetypeSet, Query,
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
#[derive(Default, Clone)]
struct PlayerColumns {
    pos: Column<Position>,
    vel: Column<Velocity>,
    col: Column<Color>,
}

impl EntityColumns for PlayerColumns {
    type Entity = Player;

    fn column<C: Component>(&mut self) -> Option<(*mut C, usize)> {
        if TypeId::of::<C>() == TypeId::of::<Position>() {
            (&mut self.pos as &mut dyn Any).downcast_mut::<Column<C>>()
        } else if TypeId::of::<C>() == TypeId::of::<Velocity>() {
            (&mut self.vel as &mut dyn Any).downcast_mut::<Column<C>>()
        } else if TypeId::of::<C>() == TypeId::of::<Color>() {
            (&mut self.col as &mut dyn Any).downcast_mut::<Column<C>>()
        } else {
            None
        }
        .map(|column| column.as_raw_parts())
    }

    fn push(&mut self, entity: Self::Entity) {
        self.pos.push(entity.pos);
        self.vel.push(entity.vel);
    }

    fn remove(&mut self, index: usize) -> Self::Entity {
        Player {
            pos: self.pos.remove(index),
            vel: self.vel.remove(index),
            col: self.col.remove(index),
        }
    }
}

impl Entity for Player {
    type Columns = PlayerColumns;
}

#[derive(Clone)]
struct Target(EntityId<World>);

#[derive(Clone)]
struct Enemy {
    pos: Position,
    target: Target,
}

// generated
#[derive(Default, Clone)]
struct EnemyColumns {
    pos: Column<Position>,
    target: Column<Target>,
}

impl EntityColumns for EnemyColumns {
    type Entity = Enemy;

    fn column<C: Component>(&mut self) -> Option<(*mut C, usize)> {
        if TypeId::of::<C>() == TypeId::of::<Position>() {
            (&mut self.pos as &mut dyn Any).downcast_mut::<Column<C>>()
        } else if TypeId::of::<C>() == TypeId::of::<Target>() {
            (&mut self.target as &mut dyn Any).downcast_mut::<Column<C>>()
        } else {
            None
        }
        .map(|column| column.as_raw_parts())
    }

    fn push(&mut self, entity: Self::Entity) {
        self.pos.push(entity.pos);
        self.target.push(entity.target);
    }

    fn remove(&mut self, index: usize) -> Self::Entity {
        Enemy {
            pos: self.pos.remove(index),
            target: self.target.remove(index),
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
#[derive(Clone, Copy)]
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

impl stecs::ArchetypeSet for World {
    type EntityId = WorldEntityId;

    type Entity = WorldEntity;

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

    type QueryIter<'a, Q> = Chain<Flatten<IntoIter<Q::Iter<Player>>>, Flatten<IntoIter<Q::Iter<Enemy>>>>
    where
        Self: 'a,
        Q: Query<'a, Self>;

    fn iter<'a, Q>(&'a mut self) -> Self::QueryIter<'a, Q>
    where
        Q: Query<'a, Self>,
    {
        let iter = Q::iter_archetype(&mut self.players).into_iter().flatten();
        let iter = iter.chain(Q::iter_archetype(&mut self.enemies).into_iter().flatten());

        iter
    }

    /*
    type QueryIter<'a, Q: Query<'a, Self>> = Chain<
        GetterIter<'a, Self, Player, Q::Getter<Player>>,
        GetterIter<'a, Self, Enemy, Q::Getter<Enemy>>,
    >;

    fn spawn<A: ArchetypeInSet<Self>>(&mut self, entity: A) -> Self::EntityId {
        match entity.into_entity() {
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
    */
}

impl InArchetypeSet<World> for Player {
    fn id(key: EntityKey<Self>) -> EntityId<World> {
        EntityId::<World>::Player(key)
    }

    fn into_entity(self) -> <World as ArchetypeSet>::Entity {
        WorldEntity::Player(self)
    }
}

impl InArchetypeSet<World> for Enemy {
    fn id(key: EntityKey<Self>) -> EntityId<World> {
        EntityId::<World>::Enemy(key)
    }

    fn into_entity(self) -> <World as ArchetypeSet>::Entity {
        WorldEntity::Enemy(self)
    }
}

/*
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
*/

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

    for p in world.iter::<&mut Position>() {
        dbg!(p.0);
        p.0 += 3.0;
    }

    dbg!("--");

    for p in world.iter::<&Position>() {
        dbg!(p.0);
    }

    dbg!("--");

    for (p, v) in world.iter::<(&Position, &Velocity)>() {
        dbg!(p.0, v.0);
    }

    dbg!("--");

    for (p, v) in world.iter::<(&mut Position, &Velocity)>() {
        p.0 += v.0;
    }

    dbg!("--");

    while let Some((p, v, join)) = world
        .stream::<(&mut Position, &Velocity)>()
        .join::<&mut Position>()
    {
        for p in join.iter() {}
    }

    struct RopeNode {
        next: EntityId<World>,
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

    /*for (p, q) in world.query::<(&mut Position, &mut Position)>() {
        p.0 += q.0;
    }*/

    /*for (p, q) in world.query::<(&mut Position, &Position)>() {
        p.0 += q.0;
    }

    for (p, q) in world.query::<(&Position, &Position)>() {}*/

    dbg!("--");

    /*
    for (id, _) in world.query::<(EntityId<World>, &Position)>() {
        dbg!(id);
    }

    dbg!("--");

    for (id, target) in world.query::<(EntityId<World>, &Target)>() {
        println!("{:?} targeting {:?}", id, target);
    }
    */

    /*
    let id: EntityId<MyWorld> = todo!();

    match id {
        EntityId::<MyWorld>::Player(_) => todo!(),
        WorldEntityId::Enemy(_) => todo!(),
    }
    */
}
