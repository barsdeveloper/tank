use crate::{AsValue, ColumnDef, Entity, TableRef};
use rust_decimal::Decimal;
use std::{marker::PhantomData, mem};

/// Decimal wrapper enforcing compile-time width/scale.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FixedDecimal<const WIDTH: u8, const SCALE: u8>(pub Decimal);

impl<const W: u8, const S: u8> From<Decimal> for FixedDecimal<W, S> {
    fn from(value: Decimal) -> Self {
        Self(value)
    }
}

impl<const W: u8, const S: u8> From<FixedDecimal<W, S>> for Decimal {
    fn from(value: FixedDecimal<W, S>) -> Self {
        value.0
    }
}

/// Wrapper marking whether a column should be considered set or skipped (passive) on INSERT.
#[derive(Debug, Default)]
pub enum Passive<T: AsValue> {
    /// Active value.
    Set(T),
    /// Skip during value emission (DEFAULT used by `SqlWriter`).
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

/// Foreign key reference to another Entity's columns.
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
        T::table().clone()
    }
    pub fn columns(&self) -> &[ColumnDef] {
        &self.columns
    }
}
