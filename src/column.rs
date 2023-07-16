use std::mem;

pub mod iter;

// TODO: Do NonZero optimization as in `thunderdome`.

#[derive(Debug, Clone)]
pub(crate) enum Slot<C> {
    Vacant { next_free_index: Option<usize> },
    Occupied { value: C },
}

impl<C> Slot<C> {
    fn value(&self) -> Option<&C> {
        let Slot::Occupied { value } = self else {
            return None;
        };

        Some(value)
    }

    fn value_mut(&mut self) -> Option<&C> {
        let Slot::Occupied { value } = self else {
            return None;
        };

        Some(value)
    }
}

pub(crate) struct Entry<C> {
    generation: usize,
    slot: Slot<C>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ColumnKey {
    generation: usize,
    index: usize,
}

pub struct Column<C> {
    entries: Vec<Entry<C>>,
    first_free_index: Option<usize>,
    len: usize,
}

impl<C> Default for Column<C> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            first_free_index: None,
            len: 0,
        }
    }
}

impl<C> Column<C> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, key: ColumnKey) -> Option<&C> {
        self.entries
            .get(key.index)
            .filter(|entry| entry.generation == key.generation)
            .and_then(|entry| entry.slot.value())
    }

    pub fn get_mut(&mut self, key: ColumnKey) -> Option<&C> {
        self.entries
            .get_mut(key.index)
            .filter(|entry| entry.generation == key.generation)
            .and_then(|entry| entry.slot.value_mut())
    }

    pub fn contains_key(&self, key: ColumnKey) -> bool {
        self.get(key).is_some()
    }

    pub fn insert(&mut self, value: C) -> ColumnKey {
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

            ColumnKey {
                generation: next_generation,
                index,
            }
        } else {
            let index = self.entries.len();

            self.entries.push(Entry {
                generation: 0,
                slot: Slot::Occupied { value },
            });

            ColumnKey {
                generation: 0,
                index,
            }
        }
    }

    pub fn remove(&mut self, key: ColumnKey) -> Option<C> {
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
