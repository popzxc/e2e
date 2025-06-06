# `e2e` -- Rust framework for stateful integration tests

`e2e` provides functionality to create test suites that have setup, {`before`/`after`}_{`all`/`each`} hooks, and state
shared across the tests.
It is async-first and, to a degree, inspired by "classic" JavaScript frameworks such as `mocha` and `jest`, while still
being strongly-typed.

`e2e` may be an overkill for unit tests -- for that purpose standard testing harness (potentially extended
with tools like `nextest`) would work much better.
However, if your goal is to write somewhat complex test scenarios that require setup/teardown, this library might be
for you.

## License

`e2e` is dual-licensed under [MIT](LICENSE-MIT) and [Apache 2.0](LICENSE-APACHE) at your choice.
