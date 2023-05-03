# Cedar Example Use Cases

This repo contains two example applications encoded in the Cedar policy language.
[Document cloud drive example](./document_cloud) models a cloud-based document sharing system, like Google Drive or Dropbox. [GitHub example](./github_example/) minics GitHub repository access management.

To run example queries or perform validation, use the [cedar-policy-cli](https://github.com/cedar-policy/cedar/tree/main/cedar-policy-cli).


## Document Cloud Drive Examples
You can validate example policies and perform authorization requests.

```shell
# validate the document_cloud policies
cedar validate --policies document_cloud/policies.cedar --schema document_cloud/schema.json

# perform a request with the document_cloud policies
cedar authorize --policies document_cloud/policies.cedar --entities document_cloud/entities.json --request-json document_cloud/allow_queries/alice_view_alice_public.json
```


## Github Examples:
- Add cedar-cli executable dir to your `PATH` or define `CEDAR_BIN_DIR`.
- Run unit tests with run_all.sh
- (Note, tests are split between `allow_queries` and `deny_queries` folders)

To generate benchmark files:
- Tweak `generate_entities.py` as appropriate
- Generate the entities file: `python3 gen_entities_1000_1000_p05_teams1.json`
- Run some query. E.g.,
```
cedar authorize --policies policies.cedar --entities gen_entities_1000_1000_p05_teams1.json --query-json query_random_pull.json
```
