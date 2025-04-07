use crate::Connection;

pub trait Driver {
    type Connection: Connection;
}
