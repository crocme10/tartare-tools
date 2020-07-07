# `merge-ntfs`

Command-Line Interface merging multiple [NTFS] datasets into one [NTFS] dataset.

## Installation

As `merge-ntfs` is not pushed to crates.io, you can install it by cloning
`tartare-tools`.

```bash
git clone https://github.com/CanalTP/tartare-tools
cd tartare-tools
cargo install --path merge-ntfs
```

## Usage

```bash
merge-ntfs \
	/path/to/ntfs1/ \
	/path/to/ntfs2/ \
	/path/to/ntfs3/ \
	--output /path/to/ntfs_result/
```

* a list of [NTFS] data format to be merged
* `--output` is the path to a folder for the resulting [NTFS] data format

Get more information about the available options with `merge-ntfs --help`.

## Specifications

For more information about how to use `merge-ntfs`, see the documentation of
[`MergeNTFSWithTransfers` process].

[`MergeNTFSWithTransfers` process]: https://confluence.kisio.org/x/lYImAg#Tartare-Listedesprocessusdetraitements-MergeNTFSWithTransfers
[NTFS]: https://github.com/CanalTP/ntfs-specification/blob/master/ntfs_fr.md
