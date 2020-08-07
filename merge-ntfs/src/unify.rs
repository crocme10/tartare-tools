//! Trait to unify objects with objects of their own type.

use std::collections::BTreeSet;
use transit_model::objects::{
    CommentLinksT, KeysValues, Line, Network, Route, StopArea, StopLocation, StopPoint,
};
use typed_index_collection::{Collection, CollectionWithId, Id};

/// Trait to unify objects of same type together.
pub trait Unify {
    /// Take an object of same type to unify it with `self`.
    fn unify(&mut self, other: Self);
}

impl<T> Unify for Collection<T> {
    fn unify(&mut self, other: Self) {
        for object in other {
            self.push(object);
        }
    }
}

impl<T: Id<T> + Unify> Unify for CollectionWithId<T> {
    fn unify(&mut self, other: Self) {
        for object in other {
            if let Some(idx) = self.get_idx(object.id()) {
                self.index_mut(idx).unify(object);
            } else {
                self.push(object).unwrap();
            }
        }
    }
}

impl Unify for KeysValues {
    fn unify(&mut self, other: Self) {
        self.extend(other);
    }
}

impl Unify for CommentLinksT {
    fn unify(&mut self, other: Self) {
        self.extend(other);
    }
}

// Cannot use `.unify` for object properties, because we do not want to keep
// duplicate keys (even if value is different).
fn object_properties_unify(object_properties: &mut KeysValues, other_properties: KeysValues) {
    let keys: BTreeSet<String> = object_properties.iter().map(|(k, _)| k).cloned().collect();
    for (k, v) in other_properties {
        if !keys.contains(&k) {
            object_properties.insert((k, v));
        }
    }
}

impl Unify for Network {
    fn unify(&mut self, other: Self) {
        self.codes.unify(other.codes);
    }
}

impl Unify for Line {
    fn unify(&mut self, other: Self) {
        self.codes.unify(other.codes);
        self.comment_links.unify(other.comment_links);
        object_properties_unify(&mut self.object_properties, other.object_properties);
    }
}

impl Unify for Route {
    fn unify(&mut self, other: Self) {
        self.codes.unify(other.codes);
        self.comment_links.unify(other.comment_links);
        object_properties_unify(&mut self.object_properties, other.object_properties);
    }
}

impl Unify for StopPoint {
    fn unify(&mut self, other: Self) {
        self.codes.unify(other.codes);
        self.comment_links.unify(other.comment_links);
        object_properties_unify(&mut self.object_properties, other.object_properties);
    }
}

impl Unify for StopArea {
    fn unify(&mut self, other: Self) {
        self.codes.unify(other.codes);
        self.comment_links.unify(other.comment_links);
        object_properties_unify(&mut self.object_properties, other.object_properties);
    }
}

impl Unify for StopLocation {
    fn unify(&mut self, other: Self) {
        self.comment_links.unify(other.comment_links);
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
    impl Unify for Object {
        fn unify(&mut self, other: Self) {
            // Only keep the name of `other` if source's name is empty
            if self.name.is_empty() {
                self.name = other.name;
            }
        }
    }

    #[test]
    fn unify_collection() {
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
        collection1.unify(collection2);
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
    fn unify_collection_with_id() {
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
        collection1.unify(collection2);
        let mut values = collection1.values();
        let object = values.next().unwrap();
        assert_eq!("Object 1", object.name);
        let object = values.next().unwrap();
        assert_eq!("Object 2", object.name);
        assert_eq!(None, values.next());
    }

    #[test]
    fn unify_line_object_codes() {
        let oc1_0 = ("key1".to_string(), "value1-0".to_string());
        let oc1_1 = ("key1".to_string(), "value1-1".to_string());
        let oc1_2 = ("key1".to_string(), "value1-2".to_string());
        let oc2 = ("key2".to_string(), "value2".to_string());
        let oc3 = ("key3".to_string(), "value3".to_string());
        let mut keys_values1 = KeysValues::new();
        keys_values1.insert(oc1_0.clone());
        keys_values1.insert(oc1_1.clone());
        keys_values1.insert(oc2.clone());
        let mut keys_values2 = KeysValues::new();
        keys_values2.insert(oc1_0.clone());
        keys_values2.insert(oc1_2.clone());
        keys_values2.insert(oc3.clone());
        let mut expected = KeysValues::new();
        expected.insert(oc1_0);
        expected.insert(oc1_1);
        expected.insert(oc1_2);
        expected.insert(oc2);
        expected.insert(oc3);
        let mut collection1 = CollectionWithId::new(vec![Line {
            id: String::from("line_01"),
            codes: keys_values1,
            ..Default::default()
        }])
        .unwrap();
        let collection2 = CollectionWithId::new(vec![Line {
            id: String::from("line_01"),
            codes: keys_values2,
            ..Default::default()
        }])
        .unwrap();
        collection1.unify(collection2);
        let mut values = collection1.values();
        let object = values.next().unwrap();
        assert_eq!(5, object.codes.len());
        assert_eq!(expected, object.codes);
    }

    #[test]
    fn unify_line_object_properties() {
        let op1_0 = ("key1".to_string(), "value1-0".to_string());
        let op1_1 = ("key1".to_string(), "value1-1".to_string());
        let op2 = ("key2".to_string(), "value2".to_string());
        let op3 = ("key3".to_string(), "value3".to_string());
        let mut keys_values1 = KeysValues::new();
        keys_values1.insert(op1_0.clone());
        keys_values1.insert(op2.clone());
        let mut keys_values2 = KeysValues::new();
        keys_values2.insert(op1_1);
        keys_values2.insert(op3.clone());
        let mut expected = KeysValues::new();
        expected.insert(op1_0);
        expected.insert(op2);
        expected.insert(op3);
        let mut collection1 = CollectionWithId::new(vec![Line {
            id: String::from("line_01"),
            object_properties: keys_values1,
            ..Default::default()
        }])
        .unwrap();
        let collection2 = CollectionWithId::new(vec![Line {
            id: String::from("line_01"),
            object_properties: keys_values2,
            ..Default::default()
        }])
        .unwrap();
        collection1.unify(collection2);
        let mut values = collection1.values();
        let object = values.next().unwrap();
        assert_eq!(3, object.object_properties.len());
        assert_eq!(expected, object.object_properties);
    }

    #[test]
    fn unify_line_comment_links() {
        let mut keys_values1 = CommentLinksT::new();
        keys_values1.insert("1".to_string());
        keys_values1.insert("2".to_string());
        let mut keys_values2 = CommentLinksT::new();
        keys_values2.insert("1".to_string());
        keys_values2.insert("3".to_string());
        let mut expected = CommentLinksT::new();
        expected.insert("1".to_string());
        expected.insert("2".to_string());
        expected.insert("3".to_string());
        let mut collection1 = CollectionWithId::new(vec![Line {
            id: String::from("line_01"),
            comment_links: keys_values1,
            ..Default::default()
        }])
        .unwrap();
        let collection2 = CollectionWithId::new(vec![Line {
            id: String::from("line_01"),
            comment_links: keys_values2,
            ..Default::default()
        }])
        .unwrap();
        collection1.unify(collection2);
        let mut values = collection1.values();
        let object = values.next().unwrap();
        assert_eq!(3, object.comment_links.len());
        assert_eq!(expected, object.comment_links);
    }
}
