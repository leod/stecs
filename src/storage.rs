use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub enum Entry<T> {
    Vacant { next_free_index: usize },
    Occupied { data: T },
}

pub struct Slot<T> {
    generation: usize,
    slot: Entry<T>,
}

pub struct EntityIndex<T> {
    generation: usize,
    index: usize,
    _phantom: PhantomData<T>,
}

pub struct Storage<T> {
    generation: usize,
    slots: Vec<Slot<T>>,
    first_free_index: Option<usize>,
}

impl<T> Storage<T> {
    fn insert(&mut self, value: T) -> EntityIndex<T> {
        if let Some(slot) = self.first_free_slot {}
    }
}
