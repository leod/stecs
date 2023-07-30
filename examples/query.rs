#[derive(Clone)]
pub struct Position(f32);

#[derive(Clone)]
pub struct Velocity(f32);

#[derive(stecs::Query)]
pub struct PhysicsObject<'a> {
    position: &'a mut Position,
    velocity: &'a mut Velocity,
}

fn main() {}
