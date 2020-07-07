# `kv12ntfs`

Command-Line Interface converting [KV1] data format into [NTFS] data format.

## Installation

As `kv12ntfs` is not pushed to crates.io, you can install it by cloning
`tartare-tools`.

```bash
git clone https://github.com/CanalTP/tartare-tools
cd tartare-tools
cargo install --path kv12ntfs
```

## Usage

```bash
kv12ntfs \
	--input /path/to/kv1/ \
	--output /path/to/ntfs/ 
```

* `--input` is the path to a folder containing [KV1] data format
* `--output` is the path to a folder for the resulting [NTFS] data format

Get more information about the available options with `kv12ntfs --help`.

## Specifications

For more information about how to use `kv12ntfs`, see the [KV1] specifications.

[KV1]: https://confluence.kisio.org/x/OoWiAw
[NTFS]: https://github.com/CanalTP/ntfs-specification/blob/master/ntfs_fr.md
