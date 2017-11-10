Compactmap - Vec-based map that uses usize as key type and maintains internal linked list for removed nodes.

You don't choose the key when inserting a new value. You can remove any entry.

Based on [this post](https://play.rust-lang.org/?gist=599f79559d6f18cc0266&version=stable) by [eddyb](https://github.com/eddyb).

TODO:

* Methods: drain, compact, expand, clear, keys, values, ...
* Serde

License is MIT or Apache, like for Rust itself.
