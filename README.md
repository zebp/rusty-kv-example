# rusty-kv-example

An example project of how to use Workers KV with [`workers-rs`](https://github.com/cloudflare/workers-rs).

## Usage

With `wrangler`, you can build, test, and deploy your Worker with the following commands:

```sh
# compiles your project to WebAssembly and will warn of any issues
$ npm run build

# run your Worker in an ideal development workflow (with a local server, file watcher & more)
$ npm run dev

# deploy your Worker globally to the Cloudflare network (update your wrangler.toml file for configuration)
$ npm run deploy
```

Read the latest `worker` crate documentation here: https://docs.rs/worker

## WebAssembly

`workers-rs` (the Rust SDK for Cloudflare Workers used in this template) is meant to be executed as compiled WebAssembly, and as such so **must** all the code you write and depend upon. All crates and modules used in Rust-based Workers projects have to compile to the `wasm32-unknown-unknown` triple.

Read more about this on the [`workers-rs`](https://github.com/cloudflare/workers-rs) project README.

## Rust

[`workers-rs`](https://github.com/cloudflare/workers-rs) requires a Rust version of `1.62.0` or higher, you can use the command `rustup update` to update if you have installed your Rust toolchain via [rustup](https://rustup.rs/). Some Linux distrobutions will ship a Rust toolchain with their provided package manager, but this might not be up to date with the versions `rustup` provides. Switching to `rustup` is recommended if you are experiencing issues with your current rust version which can be checked with `rustc --version`.

## Issues

If you have any problems with the `worker` crate, please open an issue on the upstream project issue tracker on the [`workers-rs` repository](https://github.com/cloudflare/workers-rs).
