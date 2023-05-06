# Cedar Example Use Cases

We present how Cedar language enables authorization for two example applications.
[Document cloud drive example](./document_cloud) models a cloud-based document sharing system, like Google Drive or Dropbox. [GitHub example](./github_example/) minics GitHub repository access permissions.

## Quick Start
You can validate example policies and perform authorization requests using [Cedar CLI](https://github.com/cedar-policy/cedar/tree/main/cedar-policy-cli).

```shell
# validate the document_cloud policies
cedar validate --policies document_cloud/policies.cedar --schema document_cloud/schema.json

# perform a request with the document_cloud policies
cedar authorize --policies document_cloud/policies.cedar --entities document_cloud/entities.json --request-json document_cloud/allow_requests/alice_view_alice_public.json
```

## Folder Organization

| File  | Description |
| ------------- | ------------- |
| `policies.cedar`  | Cedar policies for authorization management  |
| `entities.json`  | Example entity store  |
| `schema.json` | Example schema |
| `allow_requests` | Allowed requests |
| `deny_requests` | Denied requests |
| `README.md` | Instructions specific to the example |
