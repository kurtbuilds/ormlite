use core::default::Default;
use sqlx::database::HasArguments;
use sqlx::{Arguments, Database, IntoArguments};

pub struct QueryBuilderArgs<'q, DB: Database>(pub Box<<DB as HasArguments<'q>>::Arguments>, usize);

impl<'q, DB: Database> QueryBuilderArgs<'q, DB> {
    pub fn add<T: 'q + Send + sqlx::Encode<'q, DB> + sqlx::Type<DB>>(&mut self, arg: T) {
        self.0.add(arg);
        self.1 += 1;
    }

    pub fn len(&self) -> usize {
        self.1
    }
}

impl<'q, DB: Database> IntoArguments<'q, DB> for QueryBuilderArgs<'q, DB> {
    fn into_arguments(self) -> <DB as HasArguments<'q>>::Arguments {
        *self.0
    }
}

impl<'q, DB: Database> Default for QueryBuilderArgs<'q, DB> {
    fn default() -> Self {
        Self(Box::default(), 0)
    }
}
