# s2-pmtiles ![GitHub Actions Workflow Status][test-workflow] [![npm][npm-image]][npm-url] [![crate][crate-image]][crate-url] [![downloads][downloads-image]][downloads-url] [![bundle][bundle-image]][bundle-url] [![docs-ts][docs-ts-image]][docs-ts-url] [![docs-rust][docs-rust-image]][docs-rust-url] ![doc-coverage][doc-coverage-image] ![code-coverage][code-coverage-image] [![Discord][discord-image]][discord-url]

[test-workflow]: https://img.shields.io/github/actions/workflow/status/Open-S2/s2-pmtiles/test.yml?logo=github
[npm-image]: https://img.shields.io/npm/v/s2-pmtiles.svg?logo=npm&logoColor=white
[npm-url]: https://npmjs.org/package/s2-pmtiles
[crate-image]: https://img.shields.io/crates/v/s2-pmtiles.svg?logo=rust&logoColor=white
[crate-url]: https://crates.io/crates/s2-pmtiles
[bundle-image]: https://deno.bundlejs.com/badge?q=s2-pmtiles/browser&treeshake=[{+default+}]
[bundle-url]: https://bundlejs.com/?q=s2-pmtiles%2Fbrowser&treeshake=%5B%7B+default+%7D%5D
[downloads-image]: https://img.shields.io/npm/dm/s2-pmtiles.svg
[downloads-url]: https://www.npmjs.com/package/s2-pmtiles
[docs-ts-image]: https://img.shields.io/badge/docs-typescript-yellow.svg
[docs-ts-url]: https://open-s2.github.io/s2-pmtiles/
[docs-rust-image]: https://img.shields.io/badge/docs-rust-yellow.svg
[docs-rust-url]: https://docs.rs/s2-pmtiles
[doc-coverage-image]: https://raw.githubusercontent.com/Open-S2/s2-pmtiles/master/assets/doc-coverage.svg
[code-coverage-image]: https://raw.githubusercontent.com/Open-S2/s2-pmtiles/master/assets/code-coverage.svg
[discord-image]: https://img.shields.io/discord/953563031701426206?logo=discord&logoColor=white
[discord-url]: https://discord.opens2.com

## About

A Modified TypeScript implementation of the [PMTiles](https://github.com/protomaps/PMTiles) library. It is backwards compatible but offers support for the S2 Projection.

## Read The Spec

[s2-pmtiles-spec](/s2-pmtiles-spec/1.0.0/README.md)

For now this spec supports deflating metadata/directories inside the browser, but it will be removed in the future.

## Install

```bash
#bun
bun add s2-pmtiles
# pnpm
pnpm add s2-pmtiles
# yarn
yarn add s2-pmtiles
# npm
npm install s2-pmtiles

# cargo
cargo install s2-pmtiles
```

### NOTE

Everything after this needs to be updated.

### Example use

```ts
const fs = from 'fs'
import { VectorTile } from 's2-pmtiles'

// assume you can read (.pbf | .mvt | .ovt)
const fixture = fs.readFileSync('./x-y-z.vector.pbf')
// Bun const fixture = new Uint8Array(await Bun.file('./x-y-z.vector.pbf').arrayBuffer())
// load the protobuf parsing it directly
const tile = new VectorTile(fixture)

console.log(tile)

// example layer
const { landuse } = tile.layers

// grab the first feature
console.log(landuse.feature(0))
console.log(landuse.feature(0).loadGeometry())
```

## General Purpose API

### Tile

---

## Development

### Requirements

You need the tool `tarpaulin` to generate the coverage report. Install it using the following command:

```bash
cargo install cargo-tarpaulin
```

The `bacon coverage` tool is used to generate the coverage report. To utilize the [pycobertura](https://pypi.org/project/pycobertura/) package for a prettier coverage report, install it using the following command:

```bash
pip install pycobertura
```

### Running Tests

To run the tests, use the following command:

```bash
# TYPESCRIPT
## basic test
bun run test
## live testing
bun run test:dev

# RUST
## basic test
cargo test
# live testing
bacon test
```

### Generating Coverage Report

To generate the coverage report, use the following command:

```bash
cargo tarpaulin
# bacon
bacon coverage # or type `l` inside the tool
```
