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
    ($lhs:tt JOIN $rhs:tt ON $($cond:tt)+) => {
        $crate::join!(@make ::tank::JoinType::Inner, $lhs, $rhs, $($cond)+)
    };

    (@make $join_type:expr, $lhs:tt, $rhs:tt, $($cond:tt)+) => {
        ::tank::Join {
            join: $join_type,
            lhs: $crate::join!(@table $lhs),
            rhs: $crate::join!(@table $rhs),
            on: Some(::tank::expr!($($cond)+)),
        }
    };

    (@table $table:ident) => {
        $table::table_ref()
    };
    (@table $table:path) => {
        $table::table_ref()
    };
}
