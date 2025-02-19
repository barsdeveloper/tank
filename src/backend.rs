use metadata::DataType;

use crate::{Fragment, Statement};

pub trait Backend {
    fn write_sql<F: Fragment>(&self, _statement: &Statement<F>) -> String {
        let result = String::with_capacity(512);

        result
    }

    fn write_sql_fragment<F: Fragment>(&self, status: &F, output: &mut String) -> bool {
        false
    }

    fn write_sql_type(&self, data_type: &DataType);
}
