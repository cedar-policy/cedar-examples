# gdrive benchmark (static policies)
cedar validate --policies cedar-benchmarks/benches/gdrive/cedar/policies.cedar --schema cedar-benchmarks/benches/gdrive/cedar/gdrive.cedarschema.json

# gdrive benchmark (templates)
cedar validate --policies cedar-benchmarks/benches/gdrive-templates/cedar/policies.cedar --schema cedar-benchmarks/benches/gdrive-templates/cedar/gdrive-templates.cedarschema.json

# gdrive benchmark (static policies)
cedar validate --policies cedar-benchmarks/benches/github/cedar/policies.cedar --schema cedar-benchmarks/benches/github/cedar/github.cedarschema.json

# gdrive benchmark (templates)
cedar validate --policies cedar-benchmarks/benches/github-templates/cedar/policies.cedar --schema cedar-benchmarks/benches/github-templates/cedar/github-templates.cedarschema.json

# tinytodo benchmark
cedar validate --policies cedar-benchmarks/benches/tinytodo/cedar/tinytodo.cedar --schema cedar-benchmarks/benches/tinytodo/cedar/tinytodo.cedarschema.json
