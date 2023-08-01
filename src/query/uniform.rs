use std::{any::Any, mem::MaybeUninit};

use super::fetch::Fetch;

pub trait HasUniform<G, U>: Sized {
    fn get(global: &G) -> U {
        let asdf: MaybeUninit<Self> = MaybeUninit::uninit();
        unsafe { asdf.assume_init_ref() as &dyn Any }.downcast_ref();
        todo!()
    }
}

pub trait DynHasUniform<G, U> {
    fn get(&self, global: &G) -> U;
}

#[derive(Clone, Copy)]
pub struct Uniform<G, U> {
    f: fn(&G) -> U,
}

impl<G, U> Uniform<G, U> {
    fn get(self, global: &G) -> U {
        (self.f)(global)
    }
}

impl<G, U> Fetch for Uniform<G, U> {}
