use sqlx::Connection;

pub enum Join<T> {
    NotQueried,
    QueryResult(T),
    Modified(T),
}

impl<T> Join<T> {
    pub fn new(obj: T) -> Self {
        Join::Modified(obj)
    }

    pub fn loaded(&self) -> bool {
        match self {
            Join::NotQueried => false,
            Join::QueryResult(_) => true,
            Join::Modified(_) => true,
        }
    }

    pub fn modification(&self) -> Option<&T> {
        match self {
            Join::NotQueried => None,
            Join::QueryResult(_) => None,
            Join::Modified(obj) => Some(obj),
        }
    }

    pub async fn load<C: Connection>(&mut self, _conn: &mut C) -> Result<(), ()> {
        unimplemented!()
    }
}

impl<T> Default for Join<T> {
    fn default() -> Self {
        Join::NotQueried
    }
}


impl<T> Join<Vec<T>> {
    pub fn new_only(obj: T) -> Self {
        Join::Modified(vec![obj])
    }

    pub fn push(&mut self, obj: T) {
        match self {
            Join::QueryResult(t) => {
                let mut inner = std::mem::replace(t, Vec::new());
                inner.push(obj);
                *self = Join::Modified(inner);
            }
            Join::Modified(t) => {
                t.push(obj);
            }
            Join::NotQueried => {
                *self = Join::Modified(vec![obj]);
            }
        }
    }

    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        match self {
            Join::QueryResult(t) => t.iter(),
            Join::Modified(t) => t.iter(),
            Join::NotQueried => panic!("Tried to iterate over a joined collection, but it has not been queried."),
        }
    }
}

impl<T> std::ops::Index<usize> for Join<Vec<T>> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            Join::NotQueried => panic!("Tried to index into a joined collection, but it has not been queried."),
            Join::QueryResult(r) => {
                &r[index]
            }
            Join::Modified(r) => {
                &r[index]
            }
        }
    }
}

impl<T> std::ops::Deref for Join<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Join::NotQueried => panic!("Tried to deref a joined object, but it has not been queried."),
            Join::QueryResult(r) => {
                r
            }
            Join::Modified(r) => {
                r
            }
        }
    }
}
