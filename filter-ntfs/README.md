# `filter-ntfs`

Command-Line Interface filtering data (by extracting or by removing selected
Public Transport objects) from [NTFS] data format.

## Installation

As `filter-ntfs` is not pushed to crates.io, you can install it by cloning
`tartare-tools`.

```bash
git clone https://github.com/CanalTP/tartare-tools
cd tartare-tools
cargo install --path filter-ntfs
```

## Usage

```bash
filter-ntfs \
	extract \
	--networks "network_id:RATP" \
	--lines "line_code:M1" \
	--lines "line_code:M2" \
	--input /path/to/ntfs/ \
	--output /path/to/ntfs_result/
```

* an action which is `extract` or `remove` depending if you want to keep or
  remove what is selected
* `--input` is the path to a folder containing [NTFS] data format
* `--networks` (`--lines`) selects a Public Transport object with a format
  `property:value` (see [`FilterNTFS` process] for available properties)
* `--output` is the path to a folder for the resulting [NTFS] data format

Get more information about the available options with `filter-ntfs --help`.

## Specifications

For more information about how to use `filter-ntfs`, see the documentation of [`FilterNTFS` process].

[`FilterNTFS` process]: https://confluence.kisio.org/x/lYImAg#Tartare-Listedesprocessusdetraitements-FilterNTFS
[NTFS]: https://github.com/CanalTP/ntfs-specification/blob/master/ntfs_fr.md
