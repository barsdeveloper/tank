use crate::{InitialFragment, Statement};

pub fn sql() -> Statement<InitialFragment> {
    Statement::<InitialFragment>::default()
}
