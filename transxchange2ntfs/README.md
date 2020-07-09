# `transxchange2ntfs`

Command-Line Interface converting [TransXChange] data format into [NTFS] data
format.

## Installation

As `transxchange2ntfs` is not pushed to crates.io, you can install it by cloning
`tartare-tools`.

```bash
git clone https://github.com/CanalTP/tartare-tools
cd tartare-tools
cargo install --path transxchange2ntfs
```

## Usage

```bash
transxchange2ntfs \
	--input /path/to/transxchange/ \
	--naptan /path/to/naptan/ \
	--output /path/to/ntfs/
```

* `--input` is the path to a folder containing [TransXChange] data format
* `--naptan` is the path to a folder containing [NaPTAN] data format
* `--output` is the path to a folder for the resulting [NTFS] data format

Get more information about the available options with `transxchange2ntfs --help`.

## Specifications

For more information about how to use `transxchange2ntfs`, see the
[TransXChange] specifications.

[NaPTAN]: https://confluence.kisio.org/x/LYaiAw
[NTFS]: https://github.com/CanalTP/ntfs-specification/blob/master/ntfs_fr.md
[TransXChange]: https://confluence.kisio.org/x/LYaiAw
