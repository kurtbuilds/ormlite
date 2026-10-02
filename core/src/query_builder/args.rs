use core::default::Default;
use sqlx::{Arguments, Database, IntoArguments};

pub struct QueryBuilderArgs<DB: Database>(pub Box<DB::Arguments>, usize);

impl<DB: Database> QueryBuilderArgs<DB> {
    pub fn add<'t, T: Send + sqlx::Encode<'t, DB> + sqlx::Type<DB>>(&mut self, arg: T) {
        self.0.add(arg).unwrap();
        self.1 += 1;
    }

    pub fn len(&self) -> usize {
        self.1
    }
}

impl<DB: Database> IntoArguments<DB> for QueryBuilderArgs<DB> {
    fn into_arguments(self) -> DB::Arguments {
        *self.0
    }
}

impl<DB: Database> Default for QueryBuilderArgs<DB> {
    fn default() -> Self {
        Self(Box::default(), 0)
    }
}
