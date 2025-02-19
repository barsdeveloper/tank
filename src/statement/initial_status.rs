use macros::Fragment;

#[doc(hidden)]
mod tank {
    pub use crate::*;
}

#[derive(Default, Fragment, Clone)]
pub struct InitialFragment;
