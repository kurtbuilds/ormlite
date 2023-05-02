use std::ops::{Deref, DerefMut, Index, IndexMut};
use sqlmo::query::{Join as JoinQueryFragment};
use sqlmo::query::SelectColumn;

pub trait JoinMeta {
    type IdType: Clone;
    fn id(&self) -> Self::IdType;
}

impl<T: JoinMeta> JoinMeta for Option<T> {
    type IdType = Option<T::IdType>;

    fn id(&self) -> Self::IdType {
        self.as_ref().map(|x| x.id())
    }
}

/// Only represents a many-to-one relationship.
#[derive(Debug)]
pub enum Join<T: JoinMeta> {
    InsertById(T::IdType),
    NotQueried(T::IdType),
    QueryResult(T),
    Modified(T),
}


impl<T: JoinMeta> Join<T> {
    /// To load data for insertion, use `new`.
    pub fn id(&self) -> T::IdType {
        match self {
            Join::InsertById(id) => id.clone(),
            Join::NotQueried(id) => id.clone(),
            Join::QueryResult(obj) => obj.id(),
            Join::Modified(obj) => obj.id(),
        }
    }

    pub fn new_by_id(id: T::IdType) -> Self {
        Join::InsertById(id)
    }

    pub fn new(obj: T) -> Self {
        Join::Modified(obj)
    }

    /// Whether join data has been loaded into memory.
    pub fn loaded(&self) -> bool {
        match self {
            Join::InsertById(_) => false,
            Join::NotQueried(_) => false,
            Join::QueryResult(_) => true,
            Join::Modified(_) => true,
        }
    }

    pub async fn load<'a, A: sqlx::Acquire<'a>>(&mut self, _acq: A) -> Result<(), ()> {
        unimplemented!()
    }

    /// Takes ownership and return any modified data. Leaves the Join in a NotQueried state.
    #[doc(hidden)]
    pub fn _take_modification(&mut self) -> Option<T> {
        let id = self.id();
        let owned = std::mem::replace(self, Join::NotQueried(id));
        match owned {
            Join::InsertById(_) => None,
            Join::NotQueried(_) => None,
            Join::QueryResult(_) => None,
            Join::Modified(obj) => {
                Some(obj)
            }
        }
    }

    fn transition_to_modified(&mut self) -> &mut T {
        let id = self.id();
        let owned = std::mem::replace(self, Join::NotQueried(id));
        match owned {
            Join::NotQueried(_) | Join::InsertById(_) => panic!("Tried to deref_mut a joined object, but it has not been queried."),
            Join::QueryResult(r) => {
                *self = Join::Modified(r);
            }
            Join::Modified(r) => {
                *self = Join::Modified(r);
            }
        }
        match self {
            Join::Modified(r) => r,
            _ => unreachable!(),
        }
    }

    #[doc(hidden)]
    pub fn _query_result(obj: T) -> Self {
        Join::QueryResult(obj)
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
        match self {
            Join::NotQueried(_) | Join::InsertById(_) => panic!("Tried to deref a joined object, but it has not been queried."),
            Join::QueryResult(r) => r,
            Join::Modified(r) => r,
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
        id: Uuid,
        name: String,
    }

    impl JoinMeta for Org {
        type IdType = Uuid;
        fn id(&self) -> Self::IdType {
            self.id.clone()
        }
    }

    #[derive(Debug)]
    pub struct User {
        id: Uuid,
        name: String,
        // #[ormlite(join_column = "org_id")]
        org: Join<Org>,
    }

    #[test]
    fn test_join() {
        let user = User {
            id: Uuid::new_v4(),
            name: "Alice".to_string(),
            org: Join::new_by_id(Uuid::new_v4()),
        };
    }
}