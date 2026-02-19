# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2026-02-19

- Make `Key` public
- Added a changelog
- Added SemVer checks to CI
- Added MSRV checks to CI
- Make `ValidateContext`...
  - more accessible (methods use `_ctx` suffix so they don't clash with `Validate` methods,
    `Accumulator` methods have `_ctx` counterparts)
  - more consistent (`_ctx` methods always have the context as the last argument)
  - tested

## [0.3.1] - time immemorial
