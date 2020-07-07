# `transfers`

Command-Line Interface generating missing `transfers` on [NTFS] data format.

## Installation

As `transfers` is not pushed to crates.io, you can install it by cloning
`tartare-tools`.

```bash
git clone https://github.com/CanalTP/tartare-tools
cd tartare-tools
cargo install --path transfers
```

## Usage

```bash
transfers \
	--input /path/to/ntfs/ \
	--output /path/to/ntfs_result/
```

* `--input` is the path to a folder containing [NTFS] data format
* `--output` is the path to a folder for the resulting [NTFS] data format

Get more information about the available options with `transfers --help`.

## Specifications

For more information about how to use `transfers`, see the documentation of [`Transfers` process].

[`Transfers` process]: https://confluence.kisio.org/x/lYImAg#Tartare-Listedesprocessusdetraitements-Transfers
[NTFS]: https://github.com/CanalTP/ntfs-specification/blob/master/ntfs_fr.md
