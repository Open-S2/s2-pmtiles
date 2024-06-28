# Open Vector Tile Spec Changelog

## 1.0.0

Initial release.

Features added/changed from the [v3 pmtiles-spec](https://github.com/protomaps/PMTiles/blob/main/spec/v3/spec.md)

* Internal Compression (IC) is deprecated.
* changed the header format to include 5 more root and leaf directories.
* Magic Number is now encoded utf-8 `S2`.
* vector metadata MUST contain a key of `vector_layers` (replace SHOULD)
