# Cedar Example Use Cases

This repository contains Cedar policies that model the authorization logic of two example applications.
The [document cloud drive example](./document_cloud) models a cloud-based document sharing system, like Google Drive or Dropbox. The [GitHub example](./github_example/) mimics GitHub's repository access permissions.

## Quick Start

You can validate example policies and perform authorization requests using the [Cedar CLI](https://github.com/cedar-policy/cedar/tree/main/cedar-policy-cli).
Simply build the CLI following the instructions in the README, and then add the resulting executable (`cedar`) to your path.
This executable will end up in `/path/to/cedar-policy/cedar/target/debug/`.

```shell
# validate the document_cloud policies
cedar validate \
      --policies document_cloud/policies.cedar \
      --schema document_cloud/document_cloud.cedarschema.json

# perform an authorization request with the document_cloud policies
cedar authorize \
      --policies document_cloud/policies.cedar \
      --entities document_cloud/entities.json \
      --request-json document_cloud/allow_requests/alice_view_alice_public.json
```

Use the `run.sh` script to validate the policies and run every authorization test for both examples.

## Subfolder Organization

| File  | Description |
| ------------- | ------------- |
| `policies.cedar`  | Cedar policies  |
| `entities.json`  | Sample entity store  |
| `policies.cedarschema` | Cedar schema |
| `allow_requests` | Allowed requests |
| `deny_requests` | Denied requests |
| `README.md` | A tutorial walking through the application |
