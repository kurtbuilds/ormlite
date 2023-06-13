use crate::model::Model;
use async_trait::async_trait;
use sqlmo::query::Join as JoinQueryFragment;
use sqlmo::query::SelectColumn;
use sqlx::{Database, Decode, Encode, Type};
use std::ops::{Deref, DerefMut};

pub trait JoinMeta {
    type IdType: Clone + Send;
    fn _id(&self) -> Self::IdType;
}

impl<T: JoinMeta> JoinMeta for Option<T> {
    type IdType = Option<T::IdType>;

    fn _id(&self) -> Self::IdType {
        self.as_ref().map(|x| x._id())
    }
}

impl<T: JoinMeta> JoinMeta for Join<T> {
    type IdType = T::IdType;

    fn _id(&self) -> Self::IdType {
        self.id.clone()
    }
}

#[async_trait]
pub trait Loadable<DB, T: JoinMeta> {
    async fn load<'s, 'e, E>(&'s mut self, db: E) -> crate::error::Result<&'s T>
    where
        T::IdType: 'e + Send + Sync,
        E: 'e + sqlx::Executor<'e, Database = DB>;
}

#[derive(Debug)]
pub struct Join<T: JoinMeta> {
    pub id: T::IdType,
    data: JoinData<T>,
}

/// Only represents a many-to-one relationship.
#[derive(Debug)]
pub enum JoinData<T: JoinMeta> {
    NotQueried,
    QueryResult(T),
    Modified(T),
}

impl<T: JoinMeta> Join<T> {
    pub fn new_with_id(id: T::IdType) -> Self {
        Self {
            id,
            data: JoinData::NotQueried,
        }
    }

    pub fn new(obj: T) -> Self {
        Self {
            id: obj._id(),
            data: JoinData::Modified(obj),
        }
    }

    /// Whether join data has been loaded into memory.
    pub fn loaded(&self) -> bool {
        match &self.data {
            JoinData::NotQueried => false,
            JoinData::QueryResult(_) => true,
            JoinData::Modified(_) => true,
        }
    }

    pub fn is_modified(&self) -> bool {
        match &self.data {
            JoinData::NotQueried => false,
            JoinData::QueryResult(_) => false,
            JoinData::Modified(_) => true,
        }
    }

    /// Takes ownership and return any modified data. Leaves the Join in a NotQueried state.
    #[doc(hidden)]
    pub fn _take_modification(&mut self) -> Option<T> {
        let owned = std::mem::replace(&mut self.data, JoinData::NotQueried);
        match owned {
            JoinData::NotQueried => None,
            JoinData::QueryResult(_) => None,
            JoinData::Modified(obj) => Some(obj),
        }
    }
    fn transition_to_modified(&mut self) -> &mut T {
        let owned = std::mem::replace(&mut self.data, JoinData::NotQueried);
        match owned {
            JoinData::NotQueried => {
                panic!("Tried to deref_mut a joined object, but it has not been queried.")
            }
            JoinData::QueryResult(r) => {
                self.data = JoinData::Modified(r);
            }
            JoinData::Modified(r) => {
                self.data = JoinData::Modified(r);
            }
        }
        match &mut self.data {
            JoinData::Modified(r) => r,
            _ => unreachable!(),
        }
    }

    #[doc(hidden)]
    pub fn _query_result(obj: T) -> Self {
        Self {
            id: obj._id(),
            data: JoinData::QueryResult(obj),
        }
    }
}

#[async_trait]
impl<DB, T> Loadable<DB, T> for Join<T>
where
    DB: Database,
    T: JoinMeta + Model<DB> + Send,
    T::IdType: for<'a> Encode<'a, DB> + for<'a> Decode<'a, DB> + Type<DB>,
{
    async fn load<'s, 'e, E: sqlx::Executor<'e, Database = DB> + 'e>(
        &'s mut self,
        conn: E,
    ) -> crate::error::Result<&'s T>
    where
        T::IdType: 'e + Send + Sync,
    {
        let model = T::fetch_one(self.id.clone(), conn).await?;
        self.data = JoinData::QueryResult(model);
        let s = &*self;
        Ok(s.deref())
    }
}

impl<T: JoinMeta> Deref for Join<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match &self.data {
            JoinData::NotQueried => {
                panic!("Tried to deref a joined object, but it has not been queried.")
            }
            JoinData::QueryResult(r) => r,
            JoinData::Modified(r) => r,
        }
    }
}

impl<T: JoinMeta> DerefMut for Join<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.transition_to_modified()
    }
}

#[derive(Debug, Copy, Clone)]
pub enum SemanticJoinType {
    OneToMany,
    ManyToOne,
    ManyToMany(&'static str),
}

/// Not meant for end users.
#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub struct JoinDescription {
    pub joined_columns: &'static [&'static str],
    pub table_name: &'static str,
    pub relation: &'static str,
    /// The column on the local table
    pub key: &'static str,
    pub foreign_key: &'static str,
    pub semantic_join_type: SemanticJoinType,
}

impl JoinDescription {
    pub fn to_join_clause(&self, local_table: &str) -> JoinQueryFragment {
        use SemanticJoinType::*;
        let table = self.table_name;
        let relation = self.relation;
        let local_key = self.key;
        let foreign_key = self.foreign_key;
        let join = match &self.semantic_join_type {
            ManyToOne => {
                format!(r#""{relation}"."{foreign_key}" = "{local_table}"."{local_key}" "#)
            }
            OneToMany => {
                format!(r#""{relation}"."{local_key}" = "{local_table}"."{foreign_key}" "#)
            }
            ManyToMany(_join_table) => {
                unimplemented!()
            }
        };
        JoinQueryFragment::new(table)
            .alias(self.relation)
            .on_raw(join)
    }

    pub fn select_clause(&self) -> impl Iterator<Item = SelectColumn> + '_ {
        self.joined_columns
            .iter()
            .map(|c| SelectColumn::table_column(self.relation, c).alias(self.alias(c)))
    }

    pub fn alias(&self, column: &str) -> String {
        format!("__{}__{}", self.relation, column)
    }
}
