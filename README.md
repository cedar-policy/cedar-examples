# Cedar Examples

This repository contains examples demonstrating the use of [Cedar](https://github.com/cedar-policy/cedar), a policy language for writing and enforcing authorization policies in your applications.  The following table summarizes relevant information about the applications. Please refer to the `README.md` files in the subfolders for details about how to build and run them.

| Example                               | Languages    | Description                                                                                                                                                                          |
|---------------------------------------|--------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| [`tinytodo`][]                        | Rust, Python | A simple application for managing task lists that uses Cedar for authorization demonstrating the usage of the [Cedar Rust APIs][]                                                    |
| [`tinytodo-go`][]                     | Go, Python   | A simple application for managing task lists that uses Cedar for authorization demonstrating the usage of the [Cedar Go APIs][]                                                      |                                                                                                     |
| [`cedar-java-hello-world`][]          | Java         | A simple application demonstrating the usage of the [Cedar Java APIs][]                                                                                                              |
| [`cedar-rust-hello-world`][]          | Rust         | A simple application demonstrating the usage of the [Cedar Rust APIs][]                                                                                                              |
| [`cedar-wasm-example`][]              | TypeScript   | A simple application demonstrating the usage of the [Cedar Wasm APIs][]                                                                                                              |
| [`cedar-policy-language-in-action`][] | Cedar        | Cedar policies and schemas for the [Cedar policy language in action](https://catalog.workshops.aws/cedar-policy-language-in-action) workshop                                         |
| [`cedar-example-use-cases`][]         | Cedar        | Cedar policies and schemas for two example applications                                                                                                                              |
| [`oopsla2024-benchmarks`][]           | Various      | Cedar policies and schemas, along with benchmarking code and scripts, used for the performance evaluation of the [OOPSLA2024 paper on Cedar](https://dl.acm.org/doi/10.1145/3649835) |
| [`cedar-java-partial-evaluation`][]   | Java         | A simple application demonstrating partial evaluation capabilities using the [Cedar Java APIs][] |
## Security

See [CONTRIBUTING](CONTRIBUTING.md#security-issue-notifications) for more information.

## License

This project is licensed under the Apache-2.0 License.

[Cedar Rust APIs]: https://docs.rs/cedar-policy/latest/cedar_policy
[Cedar Go APIs]: https://github.com/cedar-policy/cedar-go
[Cedar Java APIs]: https://github.com/cedar-policy/cedar-java
[Cedar Wasm APIs]: https://github.com/cedar-policy/cedar/tree/main/cedar-wasm
[`cedar-example-use-cases`]: ./cedar-example-use-cases
[`cedar-java-hello-world`]: ./cedar-java-hello-world
[`cedar-rust-hello-world`]: ./cedar-rust-hello-world
[`cedar-wasm-example`]: ./cedar-wasm-example
[`cedar-policy-language-in-action`]: ./cedar-policy-language-in-action
[`oopsla2024-benchmarks`]: ./oopsla2024-benchmarks
[`tinytodo`]: ./tinytodo
[`tinytodo-go`]: ./tinytodo-go
[`cedar-java-partial-evaluation`]: ./cedar-java-partial-evaluation
