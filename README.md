cargo-check-external-types
==========================

`cargo-check-external-types` is a static analysis tool for Rust library authors
to set and verify which types from other libraries are allowed to be are exposed in
their public API. This is useful for ensuring that a breaking change to a dependency
doesn't force a breaking change in the library that's using it.

The tool has two output formats to cover different use-cases:
- `errors` (the default): Output error messages for each type that is exposed in
  the public API and exit with status 1 if there is at least one error. This is useful
  for continuous integration.
- `markdown-table`: Output the places types are exposed as a Markdown table. This is intended
  as a discovery tool for established projects.

The tool has an optional configuration file where types can by explicitly allowed.

Example Output
--------------

The test suite has a Rust library that [relies on some external types](test-workspace/test-crate/src/lib.rs).
When the tool is run against this library without any configuration,
[it emits errors](tests/default-config-expected-output.txt)
for each occurrence of an external type in the public API.

When [a config file](tests/allow-some-types.toml) is provided,
the allowed external types [no longer show up in the output](tests/allow-some-types-expected-output.txt).

When the output format is set to `markdown-table`, then
a [table of external types](tests/output-format-markdown-table-expected-output.md) is output.

How to Use
----------

_Important:_ This tool requires a nightly build of Rust to be installed since it relies on
the [rustdoc JSON output](https://github.com/rust-lang/rust/issues/76578), which hasn't been
stabilized yet. It was last tested against `nightly-2023-05-31`.

To install, run the following from this README path:

```bash
cargo install --locked cargo-check-external-types
```

Then, in your library crate path, run:
```bash
cargo +nightly check-external-types
```

This will produce errors if any external types are used in a public API at all. That's not terribly useful
on its own, so the tool can be given a config file to allow certain types. For example, we can allow
any type in `bytes` with:

```toml
allowed_external_types = [
    "bytes::*",
]
```

Save that file somewhere in your project (in this example, we choose the name `external-types.toml`), and then
run the command with:

```bash
cargo +nightly check-external-types --config external-types.toml
```

## Security

See [CONTRIBUTING](CONTRIBUTING.md#security-issue-notifications) for more information.

## License

This project is licensed under the Apache-2.0 License.
