pub struct TableRef {
    pub name: String,
}

impl ToString for TableRef {
    fn to_string(&self) -> String {
        self.name.to_string()
    }
}
