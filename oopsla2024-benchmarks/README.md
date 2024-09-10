# OOPSLA 2024 benchmarks for Cedar

This folder contains the Cedar, Rego and OpenFGA policies, along with benchmarking code and scripts, used for the performance evaluation of the [OOPSLA2024 paper on Cedar](https://dl.acm.org/doi/10.1145/3649835).
For the exact version used in the paper, use the `release/3.1.x` branch.

## Getting started

We recommend you use Docker for benchmarking, as our Dockerfile automatically
pulls in all the necessary dependencies.

1. Build the Docker image with `docker build -t cedar-benchmarks .` from this
directory.
2. Launch the Docker image in interactive mode with `docker run --rm -it cedar-benchmarks`.
3. Inside the Docker image, run `cargo run --release -- help bench`. (Note the
space between `--` and `help`.) This should show the help message (usage
instructions) for our benchmarking CLI. Seeing this message confirms that you
can build Cedar and our benchmarking harness.

## Directory structure

* `benches/`: Cedar, OpenFGA, and Rego policies used for benchmarking. See the README in that folder for more details.
* `generators/`: Python scripts that generate entity data for benchmarking.
* `opa-harness/`: Go wrapper to run OPA.
* `src/`: Rust code that performs the benchmarking. `src/main.rs` is what runs when you run `cargo run`.
* `Dockerfile`: Builds the test environment (see instructions above).
* `openfga-timing.patch`: OpenFGA patch to apply to the version used in our experiments.
* `plot.py`: Python script to plot results.
* `validate_all_benchmarks.sh`: Bash script to check that all Cedar policies validate against the provided schemas.

## Running benchmarks with the settings used for the paper

Inside the Docker image (see above), run

```shell
cargo run --release -- bench \
  --app gdrive,github,tiny-todo,gdrive-templates,github-templates \
  --engine cedar,open-fga,rego,cedar-opt \
  --num-hierarchies 200 \
  --num-requests 500 \
  --num-entities 5,10,15,20,30,40,50
```

This repeatedly starts and stops an OpenFGA server, which produces a lot of
repetitive warnings about "authentication is disabled" and "TLS is disabled" as
well as some configuration errors ("config '...' should not be higher than
'...'). These are normal. Warnings about "Exhausted randomness, retrying" are
also part of normal operation.

With the settings above, the command will take perhaps 1-2 hours to complete,
depending on your hardware specs etc.
For faster (but more noisy) results, reduce the `--num-hierarchies` or
`--num-requests` parameters.

When the command finishes, it produces:

* Summary outputs (overall averages) on stdout. These correspond to the overall
averages reported in Section 5 and also in the Introduction of the paper.
* Graphs, in the `output/` subdirectory. View these by using `docker cp` to copy
the container's `/cedar-benchmarks/output` to your local machine. (For instance,
in a separate terminal outside the container,
`docker cp confident_jepsen:/cedar-benchmarks/output ./output`. Replace
`confident_jepsen` with your own running container's name, found using
`docker ps`. Note that if `output` already existed on your local machine, this
will create a new `output` directory inside your existing `output` directory,
which is probably not what you want.)
  * `output/gdrive/times_vs_num_entities.pdf` corresponds to Fig 14a
  * `output/github/times_vs_num_entities.pdf` corresponds to Fig 14b
  * `output/tinytodo/times_vs_num_entities.pdf` corresponds to Fig 14c
  * `output/gdrive-slicing/times_vs_num_entities.pdf` corresponds to Fig 15a
  * `output/github-slicing/times_vs_num_entities.pdf` corresponds to Fig 15b
