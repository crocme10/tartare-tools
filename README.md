# tartare-tools [![Build Status](https://travis-ci.org/CanalTP/tartare-tools.svg?branch=master)](https://travis-ci.org/CanalTP/tartare-tools)

more coming soon...

## How to compile
To convert coordinates, Proj library is used. Rust requires a version 6+, so this library needs to be compiled.
Run `make install` to run the compilation locally.

## How to install
First, add `${HOME}/.cargo/bin` to your `PATH`.
```
export PATH=${PATH}:${HOME}/.cargo/bin
```

Then install them with the following command
```
cargo install --path . -f
```

You should then be able to run the binaries with
```
gtfs2ntfs -h
```

## License

Licensed under [GNU General Public License v3.0](LICENSE)
