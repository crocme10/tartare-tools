// Copyright (C) 2017 Kisio Digital and/or its affiliates.
//
// This program is free software: you can redistribute it and/or modify it
// under the terms of the GNU Affero General Public License as published by the
// Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more
// details.

// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>

//! Trait to merge objects with objects of their own type.

use std::collections::BTreeSet;
use transit_model::objects::{
    CommentLinksT, KeysValues, Line, Network, Route, StopArea, StopLocation, StopPoint,
};
use typed_index_collection::{Collection, CollectionWithId, Id};

/// Trait to merge objects of same type together.
pub trait Fuse {
    /// Take an object of same type to merge it with `self`.
    fn fuse(&mut self, other: Self);
}

impl<T> Fuse for Collection<T> {
    fn fuse(&mut self, other: Self) {
        for object in other {
            self.push(object);
        }
    }
}

impl<T: Id<T> + Fuse> Fuse for CollectionWithId<T> {
    fn fuse(&mut self, other: Self) {
        for object in other {
            if let Some(idx) = self.get_idx(object.id()) {
                self.index_mut(idx).fuse(object);
            } else {
                self.push(object).unwrap();
            }
        }
    }
}

impl Fuse for KeysValues {
    fn fuse(&mut self, other: Self) {
        self.extend(other);
    }
}

impl Fuse for CommentLinksT {
    fn fuse(&mut self, other: Self) {
        self.extend(other);
    }
}

impl Fuse for Network {
    fn fuse(&mut self, other: Self) {
        self.codes.fuse(other.codes);
    }
}

// Cannot use `.fuse` for object properties, because we do not want to keep
// duplicate keys (even if value is different).
fn object_properties_fuse(object_properties: &mut KeysValues, other_properties: KeysValues) {
    let keys: BTreeSet<String> = object_properties.iter().map(|(k, _)| k).cloned().collect();
    for (k, v) in other_properties {
        if !keys.contains(&k) {
            object_properties.insert((k, v));
        }
    }
}

impl Fuse for Line {
    fn fuse(&mut self, other: Self) {
        self.codes.fuse(other.codes);
        self.comment_links.fuse(other.comment_links);
        object_properties_fuse(&mut self.object_properties, other.object_properties);
    }
}

impl Fuse for Route {
    fn fuse(&mut self, other: Self) {
        self.codes.fuse(other.codes);
        self.comment_links.fuse(other.comment_links);
        object_properties_fuse(&mut self.object_properties, other.object_properties);
    }
}

impl Fuse for StopPoint {
    fn fuse(&mut self, other: Self) {
        self.codes.fuse(other.codes);
        self.comment_links.fuse(other.comment_links);
        object_properties_fuse(&mut self.object_properties, other.object_properties);
    }
}

impl Fuse for StopArea {
    fn fuse(&mut self, other: Self) {
        self.codes.fuse(other.codes);
        self.comment_links.fuse(other.comment_links);
        object_properties_fuse(&mut self.object_properties, other.object_properties);
    }
}

impl Fuse for StopLocation {
    fn fuse(&mut self, other: Self) {
        self.comment_links.fuse(other.comment_links);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[derive(Debug, PartialEq, Eq)]
    struct Object {
        id: String,
        name: String,
    }
    impl Id<Object> for Object {
        fn id(&self) -> &str {
            self.id.as_str()
        }
        fn set_id(&mut self, id: String) {
            self.id = id;
        }
    }
    impl Fuse for Object {
        fn fuse(&mut self, other: Self) {
            // Only keep the name of `other` if source's name is empty
            if self.name.is_empty() {
                self.name = other.name;
            }
        }
    }

    #[test]
    fn fuse_collection() {
        let mut collection1 = Collection::default();
        collection1.push(Object {
            id: "object_1".to_string(),
            name: "Object 1".to_string(),
        });
        collection1.push(Object {
            id: "object_2".to_string(),
            name: "Object 2".to_string(),
        });
        let mut collection2 = Collection::default();
        collection2.push(Object {
            id: "object_1".to_string(),
            name: "Object 1".to_string(),
        });
        collection1.fuse(collection2);
        let mut values = collection1.values();
        let object = values.next().unwrap();
        assert_eq!("Object 1", object.name);
        let object = values.next().unwrap();
        assert_eq!("Object 2", object.name);
        let object = values.next().unwrap();
        assert_eq!("Object 1", object.name);
        assert_eq!(None, values.next());
    }

    #[test]
    fn fuse_collection_with_id() {
        let mut collection1 = CollectionWithId::new(vec![
            Object {
                id: "object_1".to_string(),
                name: "Object 1".to_string(),
            },
            Object {
                id: "object_2".to_string(),
                name: String::new(),
            },
        ])
        .unwrap();
        let collection2 = CollectionWithId::new(vec![
            Object {
                id: "object_1".to_string(),
                name: "Object X".to_string(),
            },
            Object {
                id: "object_2".to_string(),
                name: "Object 2".to_string(),
            },
        ])
        .unwrap();
        collection1.fuse(collection2);
        let mut values = collection1.values();
        let object = values.next().unwrap();
        assert_eq!("Object 1", object.name);
        let object = values.next().unwrap();
        assert_eq!("Object 2", object.name);
        assert_eq!(None, values.next());
    }
}
