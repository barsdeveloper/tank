#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fragment {
    #[default]
    None,
    Casting,
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
    SqlJoin,
    SqlSelect,
    SqlSelectFrom,
    SqlSelectOrderBy,
    SqlSelectWhere,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Context {
    pub counter: u32,
    pub fragment: Fragment,
    pub qualify_columns: bool,
}

impl Context {
    pub const fn new(fragment: Fragment, qualify_columns: bool) -> Self {
        Self {
            counter: 0,
            fragment,
            qualify_columns,
        }
    }
    pub const fn new_qualify(qualify_columns: bool) -> Self {
        Self {
            counter: 0,
            fragment: Fragment::None,
            qualify_columns,
        }
    }
    pub const fn update_from(&mut self, context: &Context) {
        self.counter = context.counter;
    }
}

impl Context {
    pub const fn switch_fragment<'s>(&'s mut self, fragment: Fragment) -> ContextUpdater<'s> {
        ContextUpdater {
            current: Context { fragment, ..*self },
            previous: self,
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Context::new(Fragment::None, true)
    }
}

pub struct ContextUpdater<'a> {
    pub current: Context,
    pub previous: &'a mut Context,
}

impl<'a> Drop for ContextUpdater<'a> {
    fn drop(&mut self) {
        self.previous.counter = self.current.counter;
    }
}
