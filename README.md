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

### Example use

```ts
import { PMTilesReader, PMTilesWriter } from 's2-pmtiles'

// The File Reader you can run on bun/node/deno
const testFixture1 = new PMTilesReader(`test/fixtures/test_fixture_1.pmtiles`);
// get an WM tile
let x = 0;
let y = 0;
let z = 0;
let face = 0;
testFixture1.getTile(x, y, z); // undefied | Uint8Array
// get an S2 tile
testFixture1.getTileS2(face, x, y, z); // undefined | Uint8Array

// The File Writer you can run on bun/node/deno
const testFixture2 = new PMTilesWriter(`tmpFile.pmtiles`);
// write a tile
testFixture2.writeTileXYZ(x, y, z, Uint8Array.from([]));
// write an S2 tile
testFixture2.writeTileS2(face, x, y, z, Uint8Array.from([]));
// when you finish you commit to build the metadata
testFixture2.commit();

// The File Reader you can run in the browser
import { S2PMTilesReader } from 's2-pmtiles/browser';
// you want to add a true after the path for generic PMTiles, as it ensures 206 byte requests.
const browserFixture = new S2PMTilesReader(`https://www.example.com/test.pmtiles`, true);
// get an WM tile
browserFixture.getTile(x, y, z); // undefied | Uint8Array
// get an S2 tile
browserFixture.getTileS2(face, x, y, z); // undefined | Uint8Array
```

### Browser Support

Some tsconfigs might need some extra help to see the `s2-pmtiles/browser` package.

To fix this update your tsconfig.json with the following:

```json
{
    "compilerOptions": {
        "baseUrl": "./",
        "paths": {
            "s2-pmtiles/browser": ["./node_modules/s2-pmtiles/dist/browser.d.ts"]
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
