use std::ops::{Deref, DerefMut};
use sqlmo::query::{Join as JoinQueryFragment};
use sqlmo::query::SelectColumn;

pub trait JoinMeta {
    type IdType: Clone;
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

    pub async fn load<'a, A: sqlx::Acquire<'a>>(&mut self, _acq: A) -> Result<(), ()> {
        unimplemented!()
    }

    /// Takes ownership and return any modified data. Leaves the Join in a NotQueried state.
    #[doc(hidden)]
    pub fn _take_modification(&mut self) -> Option<T> {
        let owned = std::mem::replace(&mut self.data, JoinData::NotQueried);
        match owned {
            JoinData::NotQueried => None,
            JoinData::QueryResult(_) => None,
            JoinData::Modified(obj) => {
                Some(obj)
            }
        }
    }

    fn transition_to_modified(&mut self) -> &mut T {
        let owned = std::mem::replace(&mut self.data, JoinData::NotQueried);
        match owned {
            JoinData::NotQueried => panic!("Tried to deref_mut a joined object, but it has not been queried."),
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

// impl<T> Default for Join<T> {
//     fn default() -> Self {
//         Join::NotQueried
//     }
// }


// impl<T> Join<Vec<T>> {
//     pub fn new_only(obj: T) -> Self {
//         Join::Modified(vec![obj])
//     }
//
//     pub fn push(&mut self, obj: T) {
//         match self {
//             Join::QueryResult(t) => {
//                 let mut inner = std::mem::take(t);
//                 inner.push(obj);
//                 *self = Join::Modified(inner);
//             }
//             Join::Modified(t) => {
//                 t.push(obj);
//             }
//             Join::NotQueried => {
//                 *self = Join::Modified(vec![obj]);
//             }
//         }
//     }
//
//     pub fn iter(&self) -> core::slice::Iter<'_, T> {
//         match self {
//             Join::QueryResult(t) => t.iter(),
//             Join::Modified(t) => t.iter(),
//             Join::NotQueried => panic!("Tried to iterate over a joined collection, but it has not been queried."),
//         }
//     }
// }

// impl<T> Index<usize> for Join<Vec<T>> {
//     type Output = T;
//
//     fn index(&self, index: usize) -> &Self::Output {
//         match self {
//             Join::NotQueried => panic!("Tried to index into a joined collection, but it has not been queried."),
//             Join::QueryResult(r) => {
//                 &r[index]
//             }
//             Join::Modified(r) => {
//                 &r[index]
//             }
//         }
//     }
// }
//
// impl<T> IndexMut<usize> for Join<Vec<T>> {
//     fn index_mut(&mut self, index: usize) -> &mut Self::Output {
//         let inner = self.transition_to_modified();
//         &mut inner[index]
//     }
// }

impl<T: JoinMeta> Deref for Join<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match &self.data {
            JoinData::NotQueried => panic!("Tried to deref a joined object, but it has not been queried."),
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

    pub fn select_clause(&self) -> impl Iterator<Item=SelectColumn> + '_ {
        self.joined_columns.iter()
            .map(|c| SelectColumn::table_column(self.relation, c)
                .alias(self.alias(c)))
    }

    pub fn alias(&self, column: &str) -> String {
        format!("__{}__{}", self.relation, column)
    }
}

#[cfg(test)]
mod test {
    use sqlx::types::Uuid;
    use super::*;

    #[derive(Debug)]
    pub struct Org {
        pub id: Uuid,
        pub name: String,
    }

    impl JoinMeta for Org {
        type IdType = Uuid;
        fn _id(&self) -> Self::IdType {
            self.id.clone()
        }
    }

    #[derive(Debug)]
    pub struct User {
        pub id: Uuid,
        pub name: String,
        // #[ormlite(join_column = "org_id")]
        pub org: Join<Org>,
    }

    #[test]
    fn test_join() {
        let _user = User {
            id: Uuid::new_v4(),
            name: "Alice".to_string(),
            org: Join::new_with_id(Uuid::new_v4()),
        };
    }
}