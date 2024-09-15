<h1 style="text-align: center;">
    <div align="center">s2-pmtiles</div>
</h1>

<p align="center">
  <img src="https://img.shields.io/github/actions/workflow/status/Open-S2/s2-pmtiles/test.yml?logo=github" alt="GitHub Actions Workflow Status">
  <a href="https://npmjs.org/package/s2-pmtiles">
    <img src="https://img.shields.io/npm/v/s2-pmtiles.svg?logo=npm&logoColor=white" alt="npm">
  </a>
  <a href="https://crates.io/crates/s2-pmtiles">
    <img src="https://img.shields.io/crates/v/s2-pmtiles.svg?logo=rust&logoColor=white" alt="crate">
  </a>
  <a href="https://www.npmjs.com/package/s2-pmtiles">
    <img src="https://img.shields.io/npm/dm/s2-pmtiles.svg" alt="downloads">
  </a>
  <a href="https://bundlejs.com/?q=s2-pmtiles&treeshake=%5B%7B+S2PMTilesReader+%7D%5D">
    <img src="https://deno.bundlejs.com/badge?q=s2-pmtiles&treeshake=[{+S2PMTilesReader+}]" alt="bundle">
  </a>
  <a href="https://open-s2.github.io/s2-pmtiles/">
    <img src="https://img.shields.io/badge/docs-typescript-yellow.svg" alt="docs-ts">
  </a>
  <a href="https://docs.rs/s2-pmtiles">
    <img src="https://img.shields.io/badge/docs-rust-yellow.svg" alt="docs-rust">
  </a>
  <img src="https://raw.githubusercontent.com/Open-S2/s2-pmtiles/master/assets/doc-coverage.svg" alt="doc-coverage">
  <a href="https://coveralls.io/github/Open-S2/s2-pmtiles?branch=master">
    <img src="https://coveralls.io/repos/github/Open-S2/s2-pmtiles/badge.svg?branch=master" alt="code-coverage">
  </a>
  <a href="https://discord.opens2.com">
    <img src="https://img.shields.io/discord/953563031701426206?logo=discord&logoColor=white" alt="Discord">
  </a>
</p>

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

### Example use

```ts
import { S2PMTilesReader, S2PMTilesWriter, TileType } from 's2-pmtiles'
import { FileReader, FileWriter } from 's2-pmtiles/file';

// The File Reader you can run on bun/node/deno
const testFixture1 = new S2PMTilesReader(new FileReader('test/fixtures/test_fixture_1.pmtiles'));
// get an WM tile
let x = 0;
let y = 0;
let z = 0;
let face = 0;
testFixture1.getTile(x, y, z); // undefied | Uint8Array
// get an S2 tile
testFixture1.getTileS2(face, x, y, z); // undefined | Uint8Array

// The File Writer you can run on bun/node/deno
const testFixture2 = new S2PMTilesWriter(new FileWriter('tmpFile.pmtiles'), TileType.Pbf);
// write a tile
testFixture2.writeTileXYZ(x, y, z, Uint8Array.from([]));
// write an S2 tile
testFixture2.writeTileS2(face, x, y, z, Uint8Array.from([]));
// when you finish you commit to build the metadata
testFixture2.commit();


// The File Reader you can run in the browser
import { S2PMTilesReader } from 's2-pmtiles';
// you want to add a true after the path for generic PMTiles, as it ensures 206 byte requests.
const browserFixture = new S2PMTilesReader('https://www.example.com/test.pmtiles', true);
// get an WM tile
browserFixture.getTile(x, y, z); // undefied | Uint8Array
// get an S2 tile
browserFixture.getTileS2(face, x, y, z); // undefined | Uint8Array
```

### Browser Support

Some tsconfigs might need some extra help to see the `s2-pmtiles/file` or `s2-pmtiles/mmap` package.

To fix this update your tsconfig.json with the following:

```json
{
    "compilerOptions": {
        "baseUrl": "./",
        "paths": {
            "s2-pmtiles/file": ["./node_modules/s2-pmtiles/dist/file.d.ts"],
            "s2-pmtiles/mmap": ["./node_modules/s2-pmtiles/dist/mmap.d.ts"]
        }
    }
}
```

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
