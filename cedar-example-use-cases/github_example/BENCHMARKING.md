### Generate Benchmark Files
- Tweak `generate_entities.py` as appropriate
- Generate the entities file: `python3 generate_entities.py gen_entities_1000_1000_p05_teams1.json`
- Run some query. E.g.,
```shell
cedar authorize \
--policies policies.cedar \
--entities gen_entities_1000_1000_p05_teams1.json \
--request-json query_random_pull.json
```
