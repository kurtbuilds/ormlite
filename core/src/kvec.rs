use crate::join::JoinMeta;
use indexmap::{
    map::{Entry, Keys, Values, ValuesMut},
    IndexMap,
};
use serde::{Serialize, Serializer};
use std::fmt::Debug;
use std::fmt::Formatter;

pub struct KVec<T: JoinMeta>(IndexMap<T::IdType, T>);

impl<T: JoinMeta> KVec<T> {
    pub fn insert(&mut self, key: T::IdType, value: T) -> Option<T> {
        self.0.insert(key, value)
    }
    pub fn keys(&self) -> Keys<T::IdType, T> {
        self.0.keys()
    }

    pub fn values(&self) -> Values<T::IdType, T> {
        self.0.values()
    }

    pub fn values_mut(&mut self) -> ValuesMut<T::IdType, T> {
        self.0.values_mut()
    }

    pub fn entry(&mut self, key: T::IdType) -> Entry<T::IdType, T> {
        self.0.entry(key)
    }

    pub fn iter_mut_cloned(&mut self) -> impl Iterator<Item = (T::IdType, &mut T)> {
        self.0.iter_mut().map(|v| (v.0.clone(), v.1))
    }
}

impl<T: JoinMeta> From<Vec<T>> for KVec<T> {
    fn from(value: Vec<T>) -> Self {
        Self(value.into_iter().map(|row| (row._id(), row)).collect::<IndexMap<_, _>>())
    }
}

impl<T: JoinMeta> Default for KVec<T> {
    fn default() -> Self {
        Self(IndexMap::default())
    }
}

impl<T: JoinMeta + Serialize> Serialize for KVec<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.values().collect::<Vec<&T>>().serialize(serializer)
    }
}

impl<T: JoinMeta + Debug> Debug for KVec<T> {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        f.debug_list().entries(self.0.values()).finish()
    }
}
