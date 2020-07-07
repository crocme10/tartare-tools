# `netexidf2ntfs`

Command-Line Interface converting [NeTEx IDFM] data format into [NTFS] data
format.

## Installation

As `netexidf2ntfs` is not pushed to crates.io, you can install it by cloning
`tartare-tools`.

```bash
git clone https://github.com/CanalTP/tartare-tools
cd tartare-tools
cargo install --path netexidf2ntfs
```

## Usage

```bash
netexidf2ntfs \
	--input /path/to/netexidf/ \
	--output /path/to/ntfs/ 
```

* `--input` is the path to a folder containing [NeTEx IDFM] data format
* `--output` is the path to a folder for the resulting [NTFS] data format

Get more information about the available options with `netexidf2ntfs --help`.

## Specifications

For more information about how to use `netexidf2ntfs`, see the [NeTEx IDFM]
specifications.

[NeTEx IDFM]: https://confluence.kisio.org/x/BAYCAw
[NTFS]: https://github.com/CanalTP/ntfs-specification/blob/master/ntfs_fr.md
