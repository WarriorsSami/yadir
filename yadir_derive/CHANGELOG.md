# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.4](https://github.com/WarriorsSami/yadir/compare/yadir_derive-v0.1.3...yadir_derive-v0.1.4) - 2024-07-21

### Other
- *(yadir_derive)* remove redundant proc-macro-error deps
- *(yadir_derive)* report syn::Error instead of panicking
- *(yadir)* show complete examples for rustdocs in di manager and update yadir version in README.md

## [0.1.3](https://github.com/WarriorsSami/yadir/compare/yadir_derive-v0.1.2...yadir_derive-v0.1.3) - 2024-07-20

### Added
- *(yadir)* add lifetime specifiers (singleton, transient) and improve registering/resolving deps api for di manager

### Fixed
- *(yadir_derive)* remove redundant dev dependency upon yadir

## [0.1.2](https://github.com/WarriorsSami/yadir/compare/yadir_derive-v0.1.1...yadir_derive-v0.1.2) - 2024-07-20

### Other
- updated the following local packages: yadir

## [0.1.1](https://github.com/WarriorsSami/yadir/compare/yadir_derive-v0.1.0...yadir_derive-v0.1.1) - 2024-07-20

### Added
- *(proc_macro)* add support for specifying the output type for a struct when deriving DIBuilder

### Other
- *(proc_macro)* add example for #[derive(DIBuilder)]
- update README according to the new versions of yadir and yadir_derive
- *(proc_macro)* add support for multiple build methods from inputs
