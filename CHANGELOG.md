# Snax Changelog

## [Unreleased]

## 0.3.0 (2019-09-29)
- Updated to `proc-macro2` and `quote` 1.0 ([#8](https://github.com/LPGhatguy/snax/pull/8))
- Fixed tokenizer types leaking into public interface
- Implemented `PartialEq` for most types

## 0.2.0 (2019-02-18)
- Shuffled around crate names to make more sense
	- Syntax crate is now `snax`, in this repository
	- Renamed what was `snax` and `snax_impl` to `ritz` and `ritz_impl` and moved them to a [new repository](https://github.com/LPGhatguy/ritz)

## 0.1.0 (2019-02-18)
- Initial release