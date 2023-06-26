pub trait WorldData {}

pub struct World<D> {
    data: D,
}
