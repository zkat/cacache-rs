# cacache

A Rust port of [`cacache` for Node.js](https://npm.im/cacache).

A high-performance, concurrent, content-addressable disk cache.

## Install

Using [`cargo-edit`](https://crates.io/crates/cargo-edit)

`$ cargo add cacache`

## Documentation

* [API Docs](https://docs.rs/cacache)

## Features

* Extraction by key or by content address (shasum, etc)
* [Subresource Integrity](#integrity) web standard support
* Multi-hash support - safely host sha1, sha512, etc, in a single cache
* Automatic content deduplication
* Fault tolerance (immune to corruption, partial writes, process races, etc)
* Consistency guarantees on read and write (full data verification)
* Lockless, high-concurrency cache access
* Large file support
* Pretty darn fast
* Arbitrary metadata storage
* Punches nazis

## Contributing

The cacache team enthusiastically welcomes contributions and project participation! There's a bunch of things you can do if you want to contribute! The [Contributor Guide](CONTRIBUTING.md) has all the information you need for everything from reporting bugs to contributing entire new features. Please don't hesitate to jump in if you'd like to, or even ask us questions if something isn't clear.

All participants and maintainers in this project are expected to follow [Code of Conduct](CODE_OF_CONDUCT.md), and just generally be excellent to each other.

Happy hacking!

## License

Copyrights in this project are retained by their contributors. No copyright
assignment is required to contribute to this project.

For full authorship information, see the version control history.

This project is licensed under the [Mozilla Public License, v2](http://mozilla.org/MPL/2.0/).
