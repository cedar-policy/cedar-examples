# gdrive benchmark (static policies)
cedar validate --policies openfga-examples/gdrive/policies.cedar --schema openfga-examples/gdrive/gdrive.cedarschema.json

# gdrive benchmark (templates)
cedar validate --policies openfga-examples/gdrive-templates/policies.cedar --schema openfga-examples/gdrive-templates/gdrive-templates.cedarschema.json

# gdrive benchmark (static policies)
cedar validate --policies openfga-examples/github/policies.cedar --schema openfga-examples/github/github.cedarschema.json

# gdrive benchmark (templates)
cedar validate --policies openfga-examples/github-templates/policies.cedar --schema openfga-examples/github-templates/github-templates.cedarschema.json

# tinytodo benchmark
cedar validate --policies tinytodo/tinytodo.cedar --schema tinytodo/tinytodo.cedarschema.json
