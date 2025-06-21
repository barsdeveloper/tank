use crate::AsValue;

#[derive(Debug, Default)]
pub enum Passive<T: AsValue> {
    Set(T),
    #[default]
    NotSet,
}

impl<T: AsValue + PartialEq> PartialEq for Passive<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Set(lhs), Self::Set(rhs)) => lhs == rhs,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl<T: AsValue> Clone for Passive<T>
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
