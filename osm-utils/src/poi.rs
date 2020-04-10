// Copyright 2017 Kisio Digital and/or its affiliates.
//
// This program is free software: you can redistribute it and/or
// modify it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see
// <http://www.gnu.org/licenses/>.

use super::{get_relation_coord, get_way_coord, OsmPbfReader};
use crate::Result;
use failure::{bail, format_err};
use log::warn;
use navitia_poi_model::objects;
use serde_derive::Deserialize;
use serde_json;
use std::collections::BTreeMap;
use std::io;

#[derive(Deserialize, Debug)]
struct OsmTagsFilter {
    key: String,
    value: String,
}
#[derive(Deserialize, Debug)]
struct Rule {
    osm_tags_filters: Vec<OsmTagsFilter>,
    poi_type_id: String,
}
#[derive(Deserialize, Debug)]
pub struct PoiConfig {
    pub poi_types: Vec<objects::PoiType>,
    rules: Vec<Rule>,
}
impl Default for PoiConfig {
    fn default() -> Self {
        let res: PoiConfig =
            serde_json::from_str(include_str!("default_pois_config.json")).unwrap();
        res.check().unwrap();
        res
    }
}
impl PoiConfig {
    pub fn from_reader<R: io::Read>(r: R) -> Result<PoiConfig> {
        let res: PoiConfig = serde_json::from_reader(r)?;
        res.check()?;
        Ok(res)
    }
    pub fn is_poi(&self, tags: &osmpbfreader::Tags) -> bool {
        self.get_poi_type(tags).is_some()
    }
    pub fn get_poi_id(&self, tags: &osmpbfreader::Tags) -> Option<&str> {
        self.get_poi_type(tags).map(|poi_type| poi_type.id.as_str())
    }
    pub fn get_poi_type(&self, tags: &osmpbfreader::Tags) -> Option<&objects::PoiType> {
        self.rules
            .iter()
            .find(|rule| {
                rule.osm_tags_filters
                    .iter()
                    .all(|f| tags.get(&f.key).map_or(false, |v| v == &f.value))
            })
            .and_then(|rule| {
                self.poi_types
                    .iter()
                    .find(|poi_type| poi_type.id == rule.poi_type_id)
            })
    }
    pub fn check(&self) -> Result<()> {
        use std::collections::BTreeSet;
        let mut ids = BTreeSet::<&str>::new();
        for poi_type in &self.poi_types {
            if !ids.insert(&poi_type.id) {
                bail!("poi_type_id {:?} present several times", poi_type.id);
            }
        }
        let mut poi_type_ids = BTreeSet::<&str>::new();
        for rule in &self.rules {
            poi_type_ids.insert(rule.poi_type_id.as_str());

            if !ids.contains(rule.poi_type_id.as_str()) {
                bail!("no poi type associated to rule {:?}", rule.poi_type_id);
            }
        }

        for poi_type in &self.poi_types {
            if !poi_type_ids.contains(poi_type.id.as_str()) {
                bail!("no rule associated to poi_type_id {:?}", poi_type.id);
            }
        }
        Ok(())
    }
}

fn make_properties(tags: &osmpbfreader::Tags) -> Vec<objects::Property> {
    tags.iter()
        .map(|property| objects::Property {
            key: property.0.to_string(),
            value: property.1.to_string(),
        })
        .collect()
}

fn parse_poi(
    osmobj: &osmpbfreader::OsmObj,
    obj_map: &BTreeMap<osmpbfreader::OsmId, osmpbfreader::OsmObj>,
    matcher: &PoiConfig,
) -> Result<objects::Poi> {
    let poi_type = matcher.get_poi_type(osmobj.tags()).ok_or_else(|| {
        format_err!(
            "The poi {:?} has no tags even if it passes the filters",
            osmobj.id()
        )
    })?;
    let (id, coord) = match *osmobj {
        osmpbfreader::OsmObj::Node(ref node) => (
            format_poi_id("node", node.id.0),
            objects::Coord::new(node.lon(), node.lat()),
        ),
        osmpbfreader::OsmObj::Way(ref way) => {
            (format_poi_id("way", way.id.0), get_way_coord(obj_map, way)?)
        }
        osmpbfreader::OsmObj::Relation(ref relation) => (
            format_poi_id("relation", relation.id.0),
            get_relation_coord(obj_map, relation)?,
        ),
    };

    let name = osmobj.tags().get("name").unwrap_or(&poi_type.name);

    if coord.is_default() {
        bail!(
            "The poi {} is rejected, cause: could not compute coordinates.",
            id
        );
    }

    Ok(objects::Poi {
        id,
        name: name.to_string(),
        coord,
        poi_type_id: poi_type.id.clone(),
        properties: make_properties(osmobj.tags()),
        visible: true,
        weight: 0,
    })
}

fn format_poi_id(osm_type: &str, id: i64) -> String {
    format!("osm:{}:{}", osm_type, id)
}

/// Extract POIs from an OSM pbf.
pub fn extract_pois(pbf: &mut OsmPbfReader, matcher: &PoiConfig) -> BTreeMap<String, objects::Poi> {
    let objects = pbf.get_objs_and_deps(|o| matcher.is_poi(o.tags())).unwrap();
    objects
        .iter()
        .filter(|&(_, obj)| matcher.is_poi(obj.tags()))
        .filter_map(|(_, obj)| match parse_poi(obj, &objects, matcher) {
            Ok(poi) => Some((poi.id.clone(), poi)),
            Err(err) => {
                warn!("Error parsing POI {:?}: {}", obj.id(), err);
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    fn tags(v: &[(&str, &str)]) -> osmpbfreader::Tags {
        v.iter().map(|&(k, v)| (k.into(), v.into())).collect()
    }

    fn from_str(s: &str) -> Result<PoiConfig> {
        PoiConfig::from_reader(io::Cursor::new(s))
    }

    #[test]
    fn default_test() {
        let c = PoiConfig::default();
        assert!(c.get_poi_id(&tags(&[])).is_none());
        for s in &[
            "college",
            "university",
            "theatre",
            "hospital",
            "post_office",
            "bicycle_rental",
            "bicycle_parking",
            "parking",
            "police",
            "townhall",
        ] {
            assert_eq!(
                format!("amenity:{}", s),
                c.get_poi_id(&tags(&[("amenity", s)])).unwrap()
            );
        }
        for s in &["garden", "park"] {
            assert_eq!(
                format!("leisure:{}", s),
                c.get_poi_id(&tags(&[("leisure", s)])).unwrap()
            );
        }
        assert_eq!(
            "shop:ticket".to_string(),
            c.get_poi_id(&tags(&[("shop", "ticket")])).unwrap()
        );
    }

    #[test]
    fn parsing_errors() {
        from_str("").unwrap_err();
        from_str("{}").unwrap_err();
        from_str("42").unwrap_err();
        from_str("{").unwrap_err();
        from_str(r#"{"poi_types": []}"#).unwrap_err();
        from_str(r#"{"rules": []}"#).unwrap_err();
        from_str(r#"{"poi_types": [], "rules": []}"#).unwrap();
        from_str(r#"{"poi_types": [{"id": "foo"}], "rules": []}"#).unwrap_err();
        from_str(r#"{"poi_types": [{"name": "bar"}], "rules": []}"#).unwrap_err();
    }

    #[test]
    fn check_tests() {
        from_str(
            r#"{
            "poi_types": [
                {"id": "bob", "name": "Bob"},
                {"id": "bob", "name": "Bobitto"}
            ],
            "rules": []
        }"#,
        )
        .unwrap_err();
        from_str(
            r#"{
            "poi_types": [{"id": "bob", "name": "Bob"}],
            "rules": [
                {
                    "osm_tags_filters": [{"key": "foo", "value": "bar"}],
                    "poi_type_id": "bobette"
                }
            ]
        }"#,
        )
        .unwrap_err();
        from_str(
            r#"{
            "poi_types": [{"id": "bob", "name": "Bob"}, {"id": "bobette", "name": "Bobette"}],
            "rules": [
                {
                    "osm_tags_filters": [{"key": "foo", "value": "bar"}],
                    "poi_type_id": "bob"
                }
            ]
        }"#,
        )
        .unwrap_err();
    }

    #[test]
    fn check_attach_2_osm_categories_to_1_poi_type() {
        from_str(
            r#"{
            "poi_types": [{"id": "amenity:public_building", "name": "Public building"}],
            "rules": [
                {
                    "osm_tags_filters": [{"key": "building", "value": "public"}],
                    "poi_type_id": "amenity:public_building"
                },
                {
                    "osm_tags_filters": [{"key": "amenity", "value": "public_building"}],
                    "poi_type_id": "amenity:public_building"
                }
            ]
        }"#,
        )
        .unwrap();
    }

    #[test]
    fn check_with_colon() {
        let json = r#"{
            "poi_types": [
                {"id": "amenity:bicycle_rental", "name": "Station VLS"},
                {"id": "amenity:parking", "name": "Parking"}
            ],
            "rules": [
                {
                    "osm_tags_filters": [
                        {"key": "amenity:bicycle_rental", "value": "true"}
                    ],
                    "poi_type_id": "amenity:bicycle_rental"
                },
                {
                    "osm_tags_filters": [
                        {"key": "amenity", "value": "parking:effia"}
                    ],
                    "poi_type_id": "amenity:parking"
                }
            ]
        }"#;
        let c = from_str(json).unwrap();
        assert_eq!(
            Some("amenity:bicycle_rental"),
            c.get_poi_id(&tags(&[("amenity:bicycle_rental", "true")]))
        );
        assert_eq!(
            Some("amenity:parking"),
            c.get_poi_id(&tags(&[("amenity", "parking:effia")]))
        );
    }
    #[test]
    fn check_all_tags_first_match() {
        let json = r#"{
            "poi_types": [
                {"id": "bob_titi", "name": "Bob is Bobette and Titi is Toto"},
                {"id": "bob", "name": "Bob is Bobette"},
                {"id": "titi", "name": "Titi is Toto"},
                {"id": "foo", "name": "Foo is Bar"}
            ],
            "rules": [
                {
                    "osm_tags_filters": [
                        {"key": "bob", "value": "bobette"},
                        {"key": "titi", "value": "toto"}
                    ],
                    "poi_type_id": "bob_titi"
                },
                {
                    "osm_tags_filters": [
                        {"key": "bob", "value": "bobette"}
                    ],
                    "poi_type_id": "bob"
                },
                {
                    "osm_tags_filters": [
                        {"key": "titi", "value": "toto"}
                    ],
                    "poi_type_id": "titi"
                },
                {
                    "osm_tags_filters": [
                        {"key": "foo", "value": "bar"}
                    ],
                    "poi_type_id": "foo"
                }
            ]
        }"#;
        let c = from_str(json).unwrap();
        assert_eq!(
            Some("bob"),
            c.get_poi_id(&tags(&[
                ("bob", "bobette"),
                ("titi", "tata"),
                ("foo", "bar"),
            ],))
        );
        assert_eq!(
            Some("titi"),
            c.get_poi_id(&tags(&[
                ("bob", "bobitta"),
                ("titi", "toto"),
                ("foo", "bar"),
            ],))
        );
        assert_eq!(
            Some("bob_titi"),
            c.get_poi_id(&tags(&[
                ("bob", "bobette"),
                ("titi", "toto"),
                ("foo", "bar"),
            ],))
        );
        assert_eq!(
            Some("foo"),
            c.get_poi_id(&tags(&[
                ("bob", "bobitta"),
                ("titi", "tata"),
                ("foo", "bar"),
            ],))
        );
    }
}
