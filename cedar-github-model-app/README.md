# Cedar GitHub Model App

This repo is used to benchmark GitHub repository access permissions modeled in Cedar.


## Usage

### Build

```shell
cargo build
```

### Generate Random Entities

```shell
(cd ../cedar-example-use-cases/github_example &&
python3 generate_entities.py gen_entities_1000_1000_p05.json)
```

### Benchmark

```shell
cargo run -- benchmark --policies ../cedar-example-use-cases/github_example/policies.cedar --entities ../cedar-example-use-cases/github_example/gen_entities_1000_1000_p05.json --num-queries 10 --timing
```
