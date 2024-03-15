# OOPSLA 2024 benchmarks for Cedar

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

## Running benchmarks with the settings used for the paper

Inside the Docker image (see above), run

```
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
