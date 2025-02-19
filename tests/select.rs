#[cfg(test)]
mod tests {
    use tank::prelude::*;
    use tank::sql;
    use tank::AllowFrom;

    #[test]
    fn query_1() {
        sql().select(vec!["first".into(), "second".into()]);
    }
}
