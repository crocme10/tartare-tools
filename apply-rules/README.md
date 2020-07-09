# `apply-rules`

Command-Line Interface providing different ways of altering [NTFS] data format.

## Installation

As `apply-rules` is not pushed to crates.io, you can install it by cloning
`tartare-tools`.

```bash
git clone https://github.com/CanalTP/tartare-tools
cd tartare-tools
cargo install --path apply-rules
```

## Usage

```bash
apply-rules \
	--input /path/to/ntfs/ \
	--output /path/to/ntfs_result/ \
	--report /path/to/report.json
```

* `--input` is the path to a folder containing [NTFS] data format
* `--output` is the path to a folder for the resulting [NTFS] data format
* `--report` is the path to the JSON report that is produced by the process

Get more information about the available options with `apply-rules --help`.

## Specifications

For more information about how to use `apply-rules`, see the documentation of
[`ApplyRules` process].

[`ApplyRules` process]: https://confluence.kisio.org/x/lYImAg#Tartare-Listedesprocessusdetraitements-ApplyRules
[NTFS]: https://github.com/CanalTP/ntfs-specification/blob/master/ntfs_fr.md
