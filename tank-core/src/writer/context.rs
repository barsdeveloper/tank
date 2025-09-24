#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fragment {
    SqlCommentOnColumn,
    SqlCreateSchema,
    SqlCreateTable,
    SqlCreateTablePrimaryKey,
    SqlCreateTableUnique,
    SqlDeleteFrom,
    SqlDeleteFromWhere,
    SqlDropSchema,
    SqlDropTable,
    SqlInsertInto,
    SqlInsertIntoOnConflict,
    SqlInsertIntoValues,
    #[default]
    SqlJoin,
    SqlSelect,
    SqlSelectFrom,
    SqlSelectOrderBy,
    SqlSelectWhere,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Context {
    pub fragment: Fragment,
    pub qualify_columns: bool,
}

impl Context {
    pub fn new(qualify_columns: bool) -> Self {
        Self {
            fragment: Default::default(),
            qualify_columns,
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Context::new(true)
    }
}

impl Context {
    pub fn with_context(&self, fragment: Fragment) -> Self {
        Self { fragment, ..*self }
    }
}
