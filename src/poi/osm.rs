use crate::Result;
use log::info;
use navitia_poi_model::objects::Model;
use osm_utils::{
    poi::{extract_pois as extract_osm_pois, PoiConfig},
    OsmPbfReader,
};
use std::fs::File;
use std::path::Path;

pub fn extract_pois<P: AsRef<Path>>(osm_path: P, matcher: PoiConfig) -> Result<Model> {
    info!("Extracting pois from osm");
    let mut osm_reader = OsmPbfReader::new(File::open(osm_path.as_ref())?);
    let pois = extract_osm_pois(&mut osm_reader, &matcher);

    Ok(Model {
        pois,
        poi_types: matcher
            .poi_types
            .into_iter()
            .map(|poi_type| (poi_type.id.clone(), poi_type))
            .collect(),
    })
}
