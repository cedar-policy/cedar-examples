# Cedar Rust Hello World

This repository contains a simple hello world program demonstrating typical usage of the [Cedar Rust APIs](https://github.com/cedar-policy/cedar/tree/main/cedar-policy). 

The file `src/main.rs` provides example code for the key steps an application takes to use Cedar. These include (1) parsing policies from text; (2) creating access requests and asking whether the request is authorized by the policies; (3) making requests that have an optional `context` (in addition to a principal, action, and resource); (4) providing _entities_ with a request, which are application data relevant to the request; and (5) validating that policies are consistent with a provided schema.

The following are functions that demonstrate elements of the above.

* `parse_policy`: parsing a policy and extract some of its components
* `json_context`: constructing a request `context` from JSON values
* `entity_json`: constructing entities from JSON
* `entity_objects`: constructing entities from Rust objects
* `validate`: validating a policy

### Build and Run
```shell
cargo run
```
