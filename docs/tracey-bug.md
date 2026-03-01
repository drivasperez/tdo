# Tracey 1.1.5 Bug: `test_include` files not parsed for refs

## Summary

In tracey 1.1.5, files matched by the `test_include` pattern are discovered and counted (visible in `--log-level trace` output as `test_files=N`) but are **not parsed** for `r[verify ...]` annotations. This means verification references in test files are silently ignored, causing `tracey query status` to report 0 verified requirements even when test files contain correct `r[verify ...]` annotations.

## Reproduction

Given this config:

```styx
@schema {id crate:tracey-config@1, cli tracey}

specs (
  {
    name my-spec
    include (docs/spec/**/*.md)
    impls (
      {
        name rust
        include (src/**/*.rs)
        exclude (target/**)
        test_include (tests/**/*.rs)
      }
    )
  }
)
```

And a test file `tests/integration.rs` containing:

```rust
// r[verify cmd.inbox]
#[test]
fn test_inbox() { /* ... */ }
```

Running `tracey query status` reports 0/N verified, despite the test file existing and containing correct annotations.

## Root cause (hypothesis)

Looking at the tracey source code (`crates/tracey-core/src/scan.rs`), `scan_impl_files` is called for both `include` and `test_include` patterns. The function walks the file paths and collects them, but the code path for `test_include` files appears to only record file metadata (for `test_files` counting) without actually parsing the file contents for ref annotations. The refs are only extracted from files matched by `include`.

## Workaround

Add test file patterns to **both** `include` and `test_include`:

```styx
impls (
  {
    name rust
    include (src/**/*.rs tests/**/*.rs)
    exclude (target/**)
    test_include (tests/**/*.rs)
  }
)
```

With this config, test files are parsed for refs (via `include`) and also recognized as test files (via `test_include`), so `r[verify ...]` annotations are correctly detected and bare `r[...]` annotations in test files are flagged as validation errors (`ImplInTestFile`).

## Impact

Without the workaround, projects using tracey cannot track verification coverage if test files are only listed in `test_include`. The `tracey query untested` and `tracey query status` commands will always show 0% verification regardless of how many `r[verify ...]` annotations exist in test files.

## Environment

- tracey 1.1.5 (installed via `cargo install tracey`)
- macOS (Darwin 25.2.0)
- Rust 2024 edition
