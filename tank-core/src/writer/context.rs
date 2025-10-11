#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fragment {
    #[default]
    None,
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
    pub fn new(fragment: Fragment, qualify_columns: bool) -> Self {
        Self {
            counter: 0,
            fragment,
            qualify_columns,
        }
    }
    pub fn update_from(&mut self, context: &Context) {
        self.counter = context.counter;
    }
}

impl Context {
    pub fn switch_fragment<'s>(&'s mut self, fragment: Fragment) -> ContextUpdater<'s> {
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
