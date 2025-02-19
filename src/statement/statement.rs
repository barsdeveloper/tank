use crate::{Backend, InitialFragment};
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use super::Fragment;

pub struct Statement<S: Fragment> {
    _s: PhantomData<S>,
    data: Vec<Box<dyn Fragment>>,
}

impl<S: Fragment> Statement<S> {
    pub fn get_current_fragment(&mut self) -> &mut S {
        self.data
            .last_mut()
            .unwrap()
            .deref_mut()
            .as_any_mut()
            .downcast_mut::<S>()
            .unwrap()
    }
}

impl<S: Fragment> Statement<S> {
    pub fn add_fragment<F: Fragment>(mut self, new: Box<F>) -> Statement<F> {
        self.data.push(new);
        Statement {
            _s: Default::default(),
            data: self.data,
        }
    }

    pub fn to_sql<B: Backend>(&self, backend: &B) -> String {
        backend.write_sql(self)
    }
}

impl Default for Statement<InitialFragment> {
    fn default() -> Self {
        Self {
            _s: Default::default(),
            data: Vec::default(),
        }
    }
}

impl<F: Fragment> Deref for Statement<F> {
    type Target = F;

    fn deref(&self) -> &Self::Target {
        self.data
            .last()
            .unwrap()
            .deref()
            .as_any_ref()
            .downcast_ref::<F>()
            .unwrap()
    }
}

impl<F: Fragment> DerefMut for Statement<F> {
    fn deref_mut(&mut self) -> &mut F {
        self.get_current_fragment()
    }
}
