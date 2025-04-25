use crate::{DataSet, Expression};

#[derive(Default)]
pub enum JoinType {
    #[default]
    Inner,
    Outer,
    Left,
    Right,
    Cross,
    Natural,
}

pub struct Join<L: DataSet, R: DataSet, E: Expression> {
    pub join: JoinType,
    pub lhs: L,
    pub rhs: R,
    pub on: Option<E>,
}

impl<L: DataSet, R: DataSet, E: Expression> DataSet for Join<L, R, E> {}

#[macro_export]
macro_rules! join {
    ($lhs:ident JOIN $rhs:ident ON $($on:tt)*) => {
        $crate::join!(@make ::tank::JoinType::Inner, $lhs, $rhs, $($on)*)
    };
    (@make $join_type:expr, $lhs:ident, $rhs:ident, $($on:tt)*) => {
        ::tank::Join {
            join: $join_type,
            lhs: $lhs,
            rhs: $rhs,
            on: $($on)*,
        }
    };
}
