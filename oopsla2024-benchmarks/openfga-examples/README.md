# Cedar-OpenFGA Experiments

This repository reproduces OpenFGA's `gdrive` and `github` examples in Cedar and Rego.
The OpenFGA examples are from <https://github.com/openfga/sample-stores>, and are copied into the `openfga` directory for quick reference.
The Rego encodings are in the `rego` directory.
All other directories contain Cedar encodings, either using static policies or templates.

## Running Cedar

To run validation and sample authorization requests on the Cedar policies, use `./run.sh`

This script requires the executable for the Cedar CLI (`cedar`) to be on your `PATH`.
In general, this executable will end up in
`/path/to/cedar-policy/cedar/target/debug/` or
`/path/to/cedar-policy/cedar/target/release/`.
