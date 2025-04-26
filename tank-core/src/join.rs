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
    ($lhs:tt INNER JOIN $rhs:tt ON $($cond:tt)+) => {
        $crate::join!(@make ::tank::JoinType::Inner, $lhs, $rhs, $($cond)+)
    };
    ($lhs:tt JOIN $rhs:tt ON $($cond:tt)+) => {
        $crate::join!(@make ::tank::JoinType::Inner, $lhs, $rhs, $($cond)+)
    };
    ($lhs:tt FULL OUTER JOIN $rhs:tt ON $($cond:tt)+) => {
        $crate::join!(@make ::tank::JoinType::Outer, $lhs, $rhs, $($cond)+)
    };
    ($lhs:tt OUTER JOIN $rhs:tt ON $($cond:tt)+) => {
        $crate::join!(@make ::tank::JoinType::Outer, $lhs, $rhs, $($cond)+)
    };
    ($lhs:tt LEFT OUTER JOIN $rhs:tt ON $($cond:tt)+) => {
        $crate::join!(@make ::tank::JoinType::Left, $lhs, $rhs, $($cond)+)
    };
    ($lhs:tt LEFT JOIN $rhs:tt ON $($cond:tt)+) => {
        $crate::join!(@make ::tank::JoinType::Left, $lhs, $rhs, $($cond)+)
    };
    ($lhs:tt RIGHT OUTER JOIN $rhs:tt ON $($cond:tt)+) => {
        $crate::join!(@make ::tank::JoinType::Right, $lhs, $rhs, $($cond)+)
    };
    ($lhs:tt RIGHT JOIN $rhs:tt ON $($cond:tt)+) => {
        $crate::join!(@make ::tank::JoinType::Right, $lhs, $rhs, $($cond)+)
    };
    ($lhs:tt CROSS JOIN $rhs:tt) => {
        $crate::join!(@make ::tank::JoinType::Right, $lhs, $rhs)
    };
    ($lhs:tt CROSS $rhs:tt) => {
        $crate::join!(@make ::tank::JoinType::Cross, $lhs, $rhs)
    };
    ($lhs:tt NATURAL JOIN $rhs:tt) => {
        $crate::join!(@make ::tank::JoinType::Natural, $lhs, $rhs)
    };

    (@make $join_type:expr, $lhs:tt, $rhs:tt, $($cond:tt)+) => {
        ::tank::Join {
            join: $join_type,
            lhs: $crate::join!(@table $lhs),
            rhs: $crate::join!(@table $rhs),
            on: Some(::tank::expr!($($cond)+)),
        }
    };
    (@make $join_type:expr, $lhs:tt, $rhs:tt) => {
        ::tank::Join::<_, _, ()> {
            join: $join_type,
            lhs: $crate::join!(@table $lhs),
            rhs: $crate::join!(@table $rhs),
            on: None,
        }
    };

    (@table $table:ident) => {
        $table::table_ref()
    };
    (@table $table:path) => {
        $table::table_ref()
    };
    (@table ( $($nested:tt)+ )) => {
        $crate::join!($($nested)+)
    }
}
