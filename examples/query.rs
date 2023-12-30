use stecs::Or;

#[derive(Clone)]
pub struct Position(f32);

#[derive(Clone)]
pub struct Velocity(f32);

#[derive(stecs::Query)]
pub struct PhysicsObjectMut<'a> {
    position: &'a mut Position,
    velocity: &'a mut Velocity,
}

#[derive(stecs::Query, stecs::QueryShared)]
pub struct PhysicsObject<'a> {
    position: &'a Position,
    velocity: &'a Velocity,
    x: Or<&'a (), &'a ()>,
}

fn main() {}
