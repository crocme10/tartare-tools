# `merge-stop-areas`

Command-Line Interface regrouping Stop Areas of [NTFS] data format.

## Installation

As `merge-stop-areas` is not pushed to crates.io, you can install it by cloning
`tartare-tools`.

```bash
git clone https://github.com/CanalTP/tartare-tools
cd tartare-tools
cargo install --path merge-stop-areas
```

## Usage

```bash
merge-stop-areas \
	--input /path/to/ntfs/ \
	--distance 300 \
	--output /path/to/ntfs_result/ \
	--report /path/to/report.json
```

* `--input` is the path to a folder containing [NTFS] data format
* `--distance` is the maximum distance under which Stop Areas are merged
* `--output` is the path to a folder for the resulting [NTFS] data format
* `--report` is the path to the JSON report that is produced by the process

Get more information about the available options with `merge-stop-areas --help`.

## Specifications

For more information about how to use `merge-stop-areas`, see the documentation of
[`MergeStopAreas` process].

[`MergeStopAreas` process]: https://confluence.kisio.org/x/lYImAg#Tartare-Listedesprocessusdetraitements-MergeStopAreas
[NTFS]: https://github.com/CanalTP/ntfs-specification/blob/master/ntfs_fr.md
