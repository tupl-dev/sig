[![CICD](https://github.com/tupl-dev/sig/actions/workflows/cicd.yml/badge.svg)](https://github.com/tupl-dev/sig/actions/workflows/cicd.yml)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

# Sig
Byte pattern signature tool. Can be used to help with [Cheat Engine](https://cheatengine.org/) AOB features.

## Installing
```shell
cargo install sig --git https://github.com/tupl-dev/sig
```

## Usage
### Format Signature
```
sig format <signature>
```

For example:
```
sig format 0x001234ABCD
00 12 34 AB CD
```

### Count Signature
```
sig count <signature>
```
For example:
```
sig count 0x001234ABCD
5
```

### Merge Signatures
```
sig merge [signatures] [--file <file>]
```

The optional `--file` parameter allows for providing more signatures stored in a file, one signature per line. For example:
```
sig merge 0xABCDEF 'AB CC EF 12'
AB C? EF ??
```

## TODO
- [ ] More features
