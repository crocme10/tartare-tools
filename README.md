# tartare-tools

[![Github Actions Status][github_actions_badge]][github_actions]

[github_actions_badge]: https://github.com/CanalTP/tartare-tools/workflows/Continuous%20Integration/badge.svg
[github_actions]: https://github.com/CanalTP/tartare-tools/actions?query=workflow%3A%22Continuous+Integration%22

**`tartare-tools`** is a collection of Rust crates to manage, convert, and
enrich transit data.  This is done by implementing the [NTFS] model (used in
[navitia]).

This repository regroups crates that offer enabler-libraries and binaries to
convert and enrich transit data.

`tartare-tools` is very similar to `transit_model` in its general purpose.
`transit_model` is open-source, and `tartare-tools` is using `transit_model` to
provide additional functionalities that Kisio Digital chose to keep
closed-source.

Additionally, `tartare-tools` is itself a library providing various
functionalities. Please refer to the code to discover them.

Please check documentation attached to each crate:

* binary [**apply-rules**](apply-rules/README.md) provides different ways of
  altering [NTFS] data format.
* binary [**enrich-with-hellogo-fares**](enrich-with-hellogo-fares/README.md)
  reads [HelloGo Fares] data format (based on [NeTEx]) to merge it inside [NTFS]
  data format.
* binary [**extract-osm-pois**] extracts [Navitia POI] from an [OpenStreetMap]
  data format.
* binary [**filter-ntfs**](filter-ntfs/README.md) filters data (by extracting or
  by removing selected Public Transport objects) from [NTFS] data format.
* binary [**improve-stop-positions**] improves the geolocation of Stop Points
  using [OpenStreetMap] data format.
* binary [**kv12ntfs**](kv12ntfs/README.md) converts [KV1] data format into
  [NTFS] data format.
* binary [**map-ntfs-with-osm**] adds Network object codes from [OpenStreetMap] to
  the [NTFS] data format.
* binary [**merge-ntfs**](merge-ntfs/README.md) merges multiple [NTFS] datasets
  into one [NTFS] dataset.
* binary [**merge-pois**] merges multiple [Navitia POI] datasets into one
  [Navitia POI] data format.
* binary [**merge-stop-areas**](merge-stop-areas/README.md) regroups Stop Areas
  of [NTFS] data format.
* library [**navitia-poi-model**](navitia-poi-model/README.md) defines the Rust
  model for [Navitia POI] with tools to load/write from/to files.
* binary [**netexidf2ntfs**](netexidf2ntfs/README.md) converts [NeTEx IDFM] data
  format into [NTFS] data format.
* library [**osm-utils**](osm-utils/README.md) provides helpers to work with
  [OpenStreetMap] (OSM) data like extracting [Navitia POI].
* binary [**read-shapes-from-osm**] adds Geometries from [OpenStreetMap] to [NTFS]
  data format.
* binary [**sytral2navitia-pois**] extracts Point-Of-Interest (POI) from Sytral
  data format into a [Navitia POI] data format.
* binary [**transfers**](transfers/README.md) generates missing `transfers` on
  [NTFS] data format.
* binary [**transxchange2ntfs**](transxchange2ntfs/README.md) converts
  [TransXChange] data format into [NTFS] data format.

## Setup
For setting up `tartare-tools`, please refer to [`README.md` in
`transit_model`](https://github.com/CanalTP/transit_model/blob/master/README.md)
which contains all the needed instructions from setting up your Rust environment
to installing [PROJ].

[**extract-osm-pois**]: https://confluence.kisio.org/x/lYImAg#Tartare-Listedesprocessusdetraitements-ExtractOSMPOIs
[**improve-stop-positions**]: https://confluence.kisio.org/x/lYImAg#Tartare-Listedesprocessusdetraitements-ImproveStopsPositionWithOSM
[HelloGo Fares]: https://confluence.kisio.org/x/o4eiAw
[KV1]: https://confluence.kisio.org/x/OoWiAw
[**map-ntfs-with-osm**]: https://confluence.kisio.org/x/lYImAg#Tartare-Listedesprocessusdetraitements-MapNTFSWithOSM
[**merge-pois**]: https://confluence.kisio.org/x/lYImAg#Tartare-Listedesprocessusdetraitements-MergePOIs
[navitia]: https://github.com/CanalTP/navitia
[Navitia POI]: https://confluence.kisio.org/x/85Ui
[NeTEx]: http://netex-cen.eu
[NeTEx IDFM]: https://confluence.kisio.org/x/BAYCAw
[NTFS]: https://github.com/CanalTP/ntfs-specification/blob/master/ntfs_fr.md
[OpenStreetMap]: https://www.openstreetmap.org/
[PROJ]: https://proj.org
[**read-shapes-from-osm**]: https://confluence.kisio.org/x/lYImAg#Tartare-Listedesprocessusdetraitements-ReadShapesFromOSM
[**sytral2navitia-pois**]: https://confluence.kisio.org/x/lYImAg#Tartare-Listedesprocessusdetraitements-SytralPOIs2NavitiaPOIs(SpécifiqueSytral)
[TransXChange]: https://confluence.kisio.org/x/LYaiAw
