Rustymine
=========

Barebones Minecraft server written in Rust.
Players can query the server for information (number of players, ping, etc...)
and connect (with immediate disconnection).

All information is logged to the standard output.
Queries will log the users IP address.
Connection attempts will log the users IP address and username.

Code is released under BSD

Building
--------

Get [Cargo](http://crates.io/) and run:

```bash
cargo build
./target/rustymine
```

Alternatively you can use [Rust Nightly](http://www.rust-lang.org/install.html) or build from source.  Once you have a working `rustc` command:

```bash
rustc rustymine.rs
./rustymine
```

