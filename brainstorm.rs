#[derive(stecs::Component)]
struct Position(Vec3);
struct Velocity(Vec3);
struct Color(Vec3);

#[derive(stecs::Archetype)]
struct Player {
    pos: Position,
    vel: Velocity,
    col: Color,
}

#[derive(stecs::Archetype)]
struct Enemy {
    pos: Position,
    vel: Velocity,
}

#[derive(stecs::World)]
struct World {
    players: Arena<Player>,
    enemies: Arena<Enemy>,
}

// stecs
trait Archetype {
    fn query<Q: Query>(&self, arena: &Arena<Self>) -> impl Iterator<Q>;
}

trait Query {
    fn query<A: Archetype>() -> impl Iterator<Self>;
}

impl<C1: Component> Query for C1 {
    fn query<A: Archetype>(arena: &Arena<A>) -> impl Iterator<Self> {
        A::query_simple(arena)
    }
}

impl<C1: Component, C2: Component> Query for (C1, C2) {
    fn query<A: Archetype>(arena: &Arena<A>) -> impl Iterator<Self> {
        A::query_simple(arena)
    }
}

trait Lookup {
    fn has<C: Component>(&self) -> bool;
    fn lookup<C: Component>(&self) -> &C;
}

// generated
impl Archetype for Player {
    fn query<Q: Query>(arena: &Arena<Self>) -> Option<impl Iterator<Q>> {}

    fn query_simple<C: Component>(arena: &Arena<Self>) -> Option<impl Iterator<Item = C>> {
        if <Self as Lookup>::has::<C>() {
            Some(arena.iter().map(Lookup::lookup::<C>))
        } else {
            None
        }
    }
}

const fn basfd<T: Blub>(x: T) -> bool {
    !x.blub()
}

impl Lookup for Player {
    fn has<C: Component>() -> bool {
        if typeid::<C> == typeid::<Position> {
            true
        } else if typeid::<C> == typeid::<Velocity> {
            true
        } else {
            false
        }
    }

    fn lookup<C: Component>(&self) -> &C {
        if typeid::<C> == typeid::<Position> {
            &self.position
        } else if typeid::<C> == typeid::<Velocity> {
            &self.velocity
        } else {
            panic!()
        }
    }
}

impl World {
    fn query<Q: Query>(&self) -> impl Iterator<Q> {
        self.players.query::<Q>().chain(self.enemies.query::<Q>())
    }
}

// usage
fn main() {
    let world = World::new();

    for pos in world.query::<&Position>() {}

    for (pos, vel) in world.query::<(&Position, &mut Velocity)>() {}
}
