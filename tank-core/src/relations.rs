use crate::{AsValue, ColumnDef, Entity, TableRef};
use std::{marker::PhantomData, mem};

#[derive(Debug, Default)]
pub enum Passive<T: AsValue> {
    Set(T),
    #[default]
    NotSet,
}

impl<T: AsValue> Passive<T> {
    pub fn expect(self, msg: &str) -> T {
        match self {
            Passive::Set(v) => v,
            Passive::NotSet => panic!("{}", msg),
        }
    }
    pub fn unwrap(self) -> T {
        match self {
            Passive::Set(v) => v,
            Passive::NotSet => panic!("called `Passive::unwrap()` on a `NotSet` value"),
        }
    }
}

impl<T: AsValue + PartialEq> PartialEq for Passive<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Set(lhs), Self::Set(rhs)) => lhs == rhs,
            _ => mem::discriminant(self) == mem::discriminant(other),
        }
    }
}

impl<T: AsValue + Clone> Clone for Passive<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Set(v) => Self::Set(v.clone()),
            Self::NotSet => Self::NotSet,
        }
    }
}

impl<T: AsValue> From<T> for Passive<T> {
    fn from(value: T) -> Self {
        Self::Set(value)
    }
}

pub struct References<T: Entity> {
    entity: PhantomData<T>,
    columns: Box<[ColumnDef]>,
}

impl<T: Entity> References<T> {
    pub fn new(columns: Box<[ColumnDef]>) -> Self {
        Self {
            columns,
            entity: Default::default(),
        }
    }
    pub fn table_ref(&self) -> TableRef {
        T::table_ref().clone()
    }
    pub fn columns(&self) -> &[ColumnDef] {
        &self.columns
    }
}
