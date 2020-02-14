# tartare-tools [![Build Status](https://travis-ci.org/CanalTP/tartare-tools.svg?branch=master)](https://travis-ci.org/CanalTP/tartare-tools)

more coming soon...

## How to compile Proj version 6
To convert coordinates, Proj library is used. Rust requires a version 6+.

Debian based distributions (even the latest Ubuntu), the distributed version is 5. Therefore this library needs to be compiled.


Make sure you don’t have `libproj-dev` installed and run `make install` to run the compilation locally.


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
