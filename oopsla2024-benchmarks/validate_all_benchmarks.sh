BENCHES="${BENCHES:-benches}"

# gdrive benchmark (static policies)
cedar validate --policies $BENCHES/gdrive/cedar/policies.cedar --schema $BENCHES/gdrive/cedar/gdrive.cedarschema.json

# gdrive benchmark (templates)
cedar validate --policies $BENCHES/gdrive-templates/cedar/policies.cedar --schema $BENCHES/gdrive-templates/cedar/gdrive-templates.cedarschema.json

# gdrive benchmark (static policies)
cedar validate --policies $BENCHES/github/cedar/policies.cedar --schema $BENCHES/github/cedar/github.cedarschema.json

# gdrive benchmark (templates)
cedar validate --policies $BENCHES/github-templates/cedar/policies.cedar --schema $BENCHES/github-templates/cedar/github-templates.cedarschema.json

# tinytodo benchmark
cedar validate --policies $BENCHES/tinytodo/cedar/tinytodo.cedar --schema $BENCHES/tinytodo/cedar/tinytodo.cedarschema.json
