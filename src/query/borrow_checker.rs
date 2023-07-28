use std::{
    any::{type_name, TypeId},
    collections::HashSet,
};

#[derive(Debug, Clone)]
pub struct BorrowChecker {
    query: &'static str,
    shared_refs: HashSet<TypeId>,
    exclusive_refs: HashSet<TypeId>,
}

impl BorrowChecker {
    pub fn new(query: &'static str) -> Self {
        BorrowChecker {
            query,
            shared_refs: Default::default(),
            exclusive_refs: Default::default(),
        }
    }

    pub fn borrow<C: 'static>(&mut self) {
        let type_id = TypeId::of::<C>();

        if self.exclusive_refs.contains(&type_id) {
            self.panic_exclusive_and_shared::<C>();
        }

        self.shared_refs.insert(type_id);
    }

    pub fn borrow_mut<C: 'static>(&mut self) {
        let type_id = TypeId::of::<C>();

        if self.shared_refs.contains(&type_id) {
            self.panic_exclusive_and_shared::<C>();
        }

        if self.exclusive_refs.contains(&type_id) {
            self.panic_exclusive_and_exclusive::<C>();
        }

        self.exclusive_refs.insert(type_id);
    }

    fn panic_exclusive_and_shared<C>(&self) -> ! {
        panic!(
            "Query `{}` has an exclusive and a shared reference to component `{}`",
            self.query,
            type_name::<C>(),
        );
    }

    fn panic_exclusive_and_exclusive<C>(&self) -> ! {
        panic!(
            "Query `{}` has multiple exclusive references to component `{}`",
            self.query,
            type_name::<C>(),
        );
    }
}
