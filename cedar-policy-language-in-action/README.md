# Cedar policy language in action

[Cedar policy language in action](https://catalog.workshops.aws/cedar-policy-language-in-action) is hosted on workshops.aws and provides a Cedar tutorial and a set of challenge problems.  This is free to use and leverages the [Cedar Playground](https://www.cedarpolicy.com/en/playground).

The challenge problems let you apply your knowledge to create the Cedar policies, schema and entity data which implement simple versions of a Photo Application and a Source Code Management Application.  Hints are available in the workshop, and the solutions are also viewable at [PhotoApp](PhotoApp) and [GitApp](GitApp).

## Automated testing

This example includes a test script (`run.sh`) which uses the [Cedar CLI](https://github.com/cedar-policy/cedar/tree/main/cedar-policy-cli) to run test cases matching the problem requirements.

1. It uses the `validate` command against the `.cedar` and `.cedarschema.json` files.
1. It uses the `authorize` command runs against all `.json` files in `ALLOW` and `DENY` folders.

The script outputs PASS or FAIL for each test case and includes any determining policy ids (based on the `@id()` policy attribute):

```text
Using cedar-policy-cli 2.4.0

Testing PhotoApp...
 Running validation on photoapp.cedar
  ✅ PASS: validate succeeded
 Running authorization on photoapp.cedar
  ✅ PASS: decision "ALLOW" for JaneDoe-delete-JaneDoe.json determined by policy id(s):  Photo.owner
  ✅ PASS: decision "ALLOW" for JaneDoe-edit-JaneDoe.json determined by policy id(s):  Photo.owner
  ✅ PASS: decision "ALLOW" for JaneDoe-view-JaneDoe.json determined by policy id(s):  Photo.owner;  DoeFamily
  ✅ PASS: decision "ALLOW" for JaneDoe-view-JohnDoe.json determined by policy id(s):  DoeFamily
  ✅ PASS: decision "ALLOW" for JaneDoe-view-nightclub.json determined by policy id(s):  Photo.owner
  ✅ PASS: decision "ALLOW" for JaneDoe-view-sunset.json determined by policy id(s):  Photo.owner
  ✅ PASS: decision "ALLOW" for JohnDoe-view-JaneDoe.json determined by policy id(s):  DoeFamily
  ✅ PASS: decision "ALLOW" for JohnDoe-view-JohnDoe.json determined by policy id(s):  DoeFamily;  Photo.owner
  ✅ PASS: decision "ALLOW" for JohnDoe-view-sunset.json determined by policy id(s):  JaneVacation
  ✅ PASS: decision "ALLOW" for JorgeSouza-view-Judges.json determined by policy id(s):  Photo.subjects
  ✅ PASS: decision "ALLOW" for PhotoJudge-view-sunset.json determined by policy id(s):  PhotoJudge
  ✅ PASS: decision "DENY" for JohnDoe-delete-JaneDoe.json determined by policy id(s):
  ✅ PASS: decision "DENY" for JohnDoe-edit-JaneDoe.json determined by policy id(s):
  ✅ PASS: decision "DENY" for JohnDoe-view-nightclub.json determined by policy id(s):  label_private
  ✅ PASS: decision "DENY" for JorgeSouza-view-JaneDoe.json determined by policy id(s):
  ✅ PASS: decision "DENY" for PhotoJudge-view-sunset.json determined by policy id(s):

Testing GitApp...
 Running validation on gitapp.cedar
  ✅ PASS: validate succeeded
 Running authorization on gitapp.cedar
  ✅ PASS: decision "ALLOW" for JaneDoe-addRepoAdmin-CodeRepo1.json determined by policy id(s):  resource.admins_Repo
  ✅ PASS: decision "ALLOW" for JohnDoe-addIssue-CodeRepo1.json determined by policy id(s):  resource.admins_Issue
  ✅ PASS: decision "ALLOW" for JohnDoe-addRepoAdmin-CodeRepo1.json determined by policy id(s):  resource.admins_Repo
  ✅ PASS: decision "ALLOW" for JohnDoe-editIssue-CodeRepo1.json determined by policy id(s):  resource.admins_Issue
  ✅ PASS: decision "ALLOW" for JorgeSouza-editIssue-CodeRepo2.json determined by policy id(s):  resource.contributors_Issue
  ✅ PASS: decision "ALLOW" for RichardRoe-addIssue-CodeRepo1.json determined by policy id(s):  resource.reporters
  ✅ PASS: decision "ALLOW" for RichardRoe-addRepoAdmin-CodeRepo2.json determined by policy id(s):  resource.admins_Repo
  ✅ PASS: decision "ALLOW" for RichardRoe-editIssue-CodeRepo2.json determined by policy id(s):  resource.admins_Issue
  ✅ PASS: decision "DENY" for RichardRoe-addRepoAdmin-CodeRepo1.json determined by policy id(s):
  ✅ PASS: decision "DENY" for RichardRoe-editIssue-CodeRepo1.json determined by policy id(s):
```
