use thunderdome::Arena;

struct Position(f32);
struct Velocity(f32);
struct Color(f32);

struct Player {
    pos: Position,
    vel: Velocity,
    col: Color,
}

#[derive(Default)]
struct World {
    players: Arena<Player>,
    //enemies: Arena<Enemy>,
}

fn main() {}
