use std::{marker::PhantomData, mem};

pub mod iter;

// TODO: Do NonZero optimization as in `thunderdome`.

#[derive(Debug, Clone)]
pub(crate) enum Slot<T> {
    Vacant { next_free_index: Option<usize> },
    Occupied { value: T },
}

impl<T> Slot<T> {
    fn value(&self) -> Option<&T> {
        let Slot::Occupied { value } = self else {
            return None;
        };

        Some(value)
    }

    fn value_mut(&mut self) -> Option<&T> {
        let Slot::Occupied { value } = self else {
            return None;
        };

        Some(value)
    }
}

pub(crate) struct Entry<T> {
    generation: usize,
    slot: Slot<T>,
}

pub struct EntityKey<T> {
    generation: usize,
    index: usize,
    _phantom: PhantomData<T>,
}

pub struct Storage<T> {
    entries: Vec<Entry<T>>,
    first_free_index: Option<usize>,
    len: usize,
}

impl<T> Default for Storage<T> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            first_free_index: None,
            len: 0,
        }
    }
}

impl<T> Storage<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, key: EntityKey<T>) -> Option<&T> {
        self.entries
            .get(key.index)
            .filter(|entry| entry.generation == key.generation)
            .and_then(|entry| entry.slot.value())
    }

    pub fn get_mut(&mut self, key: EntityKey<T>) -> Option<&T> {
        self.entries
            .get_mut(key.index)
            .filter(|entry| entry.generation == key.generation)
            .and_then(|entry| entry.slot.value_mut())
    }

    pub fn contains_key(&self, key: EntityKey<T>) -> bool {
        self.get(key).is_some()
    }

    pub fn insert(&mut self, value: T) -> EntityKey<T> {
        self.len
            .checked_add(1)
            .expect("Storage `len` overflowed `usize`");

        if let Some(index) = self.first_free_index {
            let Entry {
                generation,
                slot: Slot::Vacant { next_free_index },
            } = &self.entries[index]
            else {
                panic!("Expected entry at index {index} to be vacant")
            };

            let next_generation = generation
                .checked_add(1)
                .expect("Storage `generation` overflowed `usize`");

            self.first_free_index = *next_free_index;
            self.entries[index] = Entry {
                generation: next_generation,
                slot: Slot::Occupied { value },
            };

            EntityKey {
                generation: next_generation,
                index,
                _phantom: PhantomData,
            }
        } else {
            let index = self.entries.len();

            self.entries.push(Entry {
                generation: 0,
                slot: Slot::Occupied { value },
            });

            EntityKey {
                generation: 0,
                index,
                _phantom: PhantomData,
            }
        }
    }

    pub fn remove(&mut self, key: EntityKey<T>) -> Option<T> {
        let Entry {
            generation,
            slot: slot @ Slot::Occupied { .. },
        } = self.entries.get_mut(key.index)?
        else {
            return None;
        };

        if *generation != key.generation {
            return None;
        }

        self.len -= 1;

        let next_free_index = self.first_free_index;
        self.first_free_index = Some(key.index);

        let Slot::Occupied { value } = mem::replace(slot, Slot::Vacant { next_free_index }) else {
            unreachable!()
        };

        Some(value)
    }
}
