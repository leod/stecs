#[derive(Clone)]
pub struct Position(f32);

#[derive(Clone)]
pub struct Velocity(f32);

#[derive(stecs::Query, stecs::QueryShared)]
pub struct PhysicsObject<'a> {
    position: &'a mut Position,
    velocity: &'a mut Velocity,
}

#[derive(stecs::Query, stecs::QueryShared)]
pub struct PhysicsObjectMut<'a> {
    position: &'a Position,
    velocity: &'a Velocity,
}

fn main() {}
