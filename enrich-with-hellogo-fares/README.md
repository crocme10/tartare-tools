# `enrich-with-hellogo-fares`

Command-Line Interface reading [HelloGo Fares] data format (based on [NeTEx]) to
merge it inside [NTFS] data format.

## Installation

As `enrich-with-hellogo-fares` is not pushed to crates.io, you can install it by
cloning `tartare-tools`.

```bash
git clone https://github.com/CanalTP/tartare-tools
cd tartare-tools
cargo install --path enrich-with-hellogo-fares
```

## Usage

```bash
enrich-with-hellogo-fares \
	--input /path/to/ntfs/ \
	--fares /path/to/hellogo_fares/ \
	--output /path/to/ntfs_result/
```

* `--input` is the path to a folder containing [NTFS] data format
* `--fares` is the path to a folder containing [HelloGo Fares] data format
* `--output` is the path to a folder for the resulting [NTFS] data format

Get more information about the available options with `enrich-with-hellogo-fares
--help`.

## Specifications

For more information about how to use `enrich-with-hellogo-fares`, see the
[HelloGo Fares] specification.

[HelloGo Fares]: https://confluence.kisio.org/x/o4eiAw
[NeTEx]: http://netex-cen.eu
[NTFS]: https://github.com/CanalTP/ntfs-specification/blob/master/ntfs_fr.md
