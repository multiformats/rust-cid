# [v0.10.0](https://github.com/multiformats/rust-cid/compare/v0.9.0...v0.10.0) (2022-12-22)


### chore

* upgrade to Rust edition 2021 and set MSRV ([#130](https://github.com/multiformats/rust-cid/issues/130)) ([91fd35e](https://github.com/multiformats/rust-cid/commit/91fd35e06f8ae24d66f6ba4598830d8dbc259c8a))


### Features

* add `encoded_len` and written bytes ([#129](https://github.com/multiformats/rust-cid/issues/129)) ([715771c](https://github.com/multiformats/rust-cid/commit/715771c48fd47969e733ed1faad8b82d9ddbd7ca))


### BREAKING CHANGES

* Return `Result<usize>` (instead of `Result<()>`) now from `Cid::write_bytes`.
* Rust edition 2021 is now used
