# Cedar GitHub Model App

This package shows how you can perform entity slicing for an authorization model. It uses an encoding of a subset of [Github's authorization model in Cedar](https://github.com/cedar-policy/cedar-examples/tree/main/cedar-example-use-cases/github_example). 

For large applications, passing all entities becomes prohibitively expensive. This code is based on the [CedarCLI](https://github.com/cedar-policy/cedar/tree/main/cedar-policy-cli). It allows you to pass in a generated set of entities and benchmark `--num-queries` authorization requests.


## Usage

### Build

```shell
cargo build
```

To generate entities, we use the python script from the [Cedar Github Model](https://github.com/cedar-policy/cedar-examples/tree/main/cedar-example-use-cases/github_example)
### Generate Random Entities

```shell
(cd ../cedar-example-use-cases/github_example &&
python3 generate_entities.py gen_entities_1000_1000_p05.json)
```
You can tweak the constants in `generate_entities.py` to change the number of users, number of repos, probablity a user is given access to a repo etc.
### Benchmark
Once the entites JSON file is generated, you can specify a number of queries. For `N` queries, the benchmarking is deterministic by taking queries `0,k,2k,...` where `k=num_users*num_repos/N`.
This is ok because our entity generation script sets attributes randomly, but you should be careful if you want to modify that script and benchmark.

```shell
cargo run -- benchmark --policies ../cedar-example-use-cases/github_example/policies.cedar --entities ../cedar-example-use-cases/github_example/gen_entities_1000_1000_p05.json --num-queries 10 --timing
```

## Security

See [CONTRIBUTING](CONTRIBUTING.md#security-issue-notifications) for more information.

## License

This project is licensed under the Apache-2.0 License.

