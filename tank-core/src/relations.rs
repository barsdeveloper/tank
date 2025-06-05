use crate::AsValue;

#[derive(Default)]
pub enum Passive<T: AsValue> {
    Set(T),
    #[default]
    NotSet,
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
