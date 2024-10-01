# TinyTodo-Go

This is a Go implementation of [TinyTodo](../tinytodo/).

It relies on [`cedar-go`](https://github.com/cedar-policy/cedar-go).

## Usage

TinyTodo-Go's usage is similar to [TinyTodo](../tinytodo/) - you are encouraged to use an identical [Python CLI tool](./tinytodo.py) to interact with the HTTP REST APIs offered by TinyTodo-Go (which are identical to the APIs offered by TinyTodo).

To run with logging, set the environment variable `GO_LOG` to level `info`:

```SHELL
> GO_LOG=info python -i tinytodo.py
```

See [TinyTodo's README](../tinytodo/README.md) for more information.

## Build

You need Python3 and Go (1.22 or later).

See [TinyTodo's README](../tinytodo/README.md) for more information.

## Comparison with TinyTodo

TinyTodo-Go is constrained by the features of [`cedar-go`](https://github.com/cedar-policy/cedar-go). Refer to [this README](https://github.com/cedar-policy/cedar-go/tree/v0.3.2?tab=readme-ov-file#comparison-to-the-rust-implementation) to learn about the missing features.