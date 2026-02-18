# Changelog

## [0.1.7](https://github.com/typester/ranma/compare/v0.1.6...v0.1.7) (2026-02-18)


### Bug Fixes

* remove_node now recursively removes all descendants (transitive closure) ([#17](https://github.com/typester/ranma/issues/17)) ([1193645](https://github.com/typester/ranma/commit/11936452e1e6ae7c581c80f5caaf6fedd98d32e7))

## [0.1.6](https://github.com/typester/ranma/compare/v0.1.5...v0.1.6) (2026-02-17)


### Features

* add style variant examples for unified bar ([#14](https://github.com/typester/ranma/issues/14)) ([dd9a0bd](https://github.com/typester/ranma/commit/dd9a0bd4f8edf5bd211f9fd48f43a3b3f21bcf87))


### Bug Fixes

* resolve display migration architectural issues ([#16](https://github.com/typester/ranma/issues/16)) ([75efb30](https://github.com/typester/ranma/commit/75efb30c024c7474d7fd095723dce5b8dbe850bd))

## [0.1.5](https://github.com/typester/ranma/compare/v0.1.4...v0.1.5) (2026-02-17)


### Features

* add `tree` subcommand to display node hierarchy ([#13](https://github.com/typester/ranma/issues/13)) ([aa5964e](https://github.com/typester/ranma/commit/aa5964e18baee764768e5a2c4901de05cfac4cbf))


### Bug Fixes

* add retry logic to detect-release step in release workflow ([#10](https://github.com/typester/ranma/issues/10)) ([b258ed1](https://github.com/typester/ranma/commit/b258ed1b7196e06b1d8ace4f51f5eee691161159))

## [0.1.4](https://github.com/typester/ranma/compare/v0.1.3...v0.1.4) (2026-02-17)


### Features

* hide bar on fullscreen spaces ([#9](https://github.com/typester/ranma/issues/9)) ([ecaf2b2](https://github.com/typester/ranma/commit/ecaf2b2fb06102ee091006658670df609e47bc65))


### Bug Fixes

* prevent bar from appearing at wrong position on display changes ([#7](https://github.com/typester/ranma/issues/7)) ([8508b81](https://github.com/typester/ranma/commit/8508b817a5eb7444fffa4e5a48e7104271643bb9))

## [0.1.3](https://github.com/typester/ranma/compare/v0.1.2...v0.1.3) (2026-02-17)


### Bug Fixes

* use correct path for ranma-server in release packaging ([#3](https://github.com/typester/ranma/issues/3)) ([e352d58](https://github.com/typester/ranma/commit/e352d5886cecee302692d64a6c62e737b61e7f4f))

## [0.1.2](https://github.com/typester/ranma/compare/v0.1.1...v0.1.2) (2026-02-17)


### Bug Fixes

* create CRanmaCore/include directory before copying headers in build script ([85a1841](https://github.com/typester/ranma/commit/85a18418fa73854520ca0aea0bf78d1369f203c2))

## [0.1.1](https://github.com/typester/ranma/compare/v0.1.0...v0.1.1) (2026-02-17)


### Features

* add container/item tree architecture, styling, and release management ([0fc4bb1](https://github.com/typester/ranma/commit/0fc4bb1de8e1e424f6d9a6814c9b771b48055ee1))
* add hover/click interaction, window positioning fix, examples, and yashiki redesign ([5ee7d9f](https://github.com/typester/ranma/commit/5ee7d9fc0ddbbaea71d11cab1c6974b064331f40))
* add item-level styling, gap/width properties, init script, and refresh batching ([7ce6673](https://github.com/typester/ranma/commit/7ce6673281060c6a4be29dbb3fabaae6ce3cd8be))
* add notch-aware positioning with per-container notch_align (left/right) ([4a976bc](https://github.com/typester/ranma/commit/4a976bc3226207e55df5ac65d3228494f656536a))
* add status bar example plugins and Dynamic Island style variant ([3350137](https://github.com/typester/ranma/commit/335013711559cf1f6a4b1b94ba56e751fc7c0798))


### Bug Fixes

* clean up windows on display detach and restore on re-attach ([8484588](https://github.com/typester/ranma/commit/8484588a3c26cc055ef9e07a623c32e0f8b0eacf))
* use simple release-type for release-please to fix workspace version detection ([0cf943e](https://github.com/typester/ranma/commit/0cf943e705d0ed4ab9978f72b0e662a828a49bee))

## Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
