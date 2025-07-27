# GitHub Model

In this example, GitHub's repository permissions are implemented.

For more information about exploring this example using the Cedar CLI, see [Cedar Example Use Cases](https://github.com/cedar-policy/cedar-examples/tree/release/4.0.x/cedar-example-use-cases).

## Use-case
There are three “views” of GitHub permissions that correspond to *Personal*, *Organization* and *Enterprise* accounts. We’ll focus on the *Organizational* view. Because this is an example, our model won’t be complete.

## Entities

### `User`, `Team`

Represents GitHub users and teams, respectively.

### `UserGroup`

Represents a set of users. Both users and teams can be in a user group. User groups can also form a hierarchy.

### `Repository`

Represents GitHub repositories and is the main resource of our model. A repository has five access roles:
* reader
* triager
* writer
* maintainer
* admin

### `Issue`

Represents an issue in a repository. Users can assign, create, and delete an issue.

### `Org`

Represents a GitHub organization that has two groups of users representing its members and owners, respectively.

## Actions
### `pull`
Pull a repository.

### `push`
Push a repository.

### `fork`
Fork a repository.

### `add_{reader, triager, writer, maintainer, admin}`
Add a user to the specified role of a repository.

### `{create, delete, assign}_issue`
Perform the specified operation on an issue.

##  Schema

### Entity Types
* `User`:
  * memberOfTypes: `UserGroup`, `Team`

* `UserGroup`:
  * memberOfTypes: `UserGroup`

* `Repository`:
  * Attributes:
    * `readers` a `UserGroup`
    * `triagers`: a `UserGroup`
    * `writers`: a `UserGroup`
    * `maintainers`: a `UserGroup`
    * `admins`: a `UserGroup`

* `Issue`:
  * Attributes:
    * repo: a `Repository`
    * reporter: a `User`

* `Org`:
  * Attributes:
    * members: a `UserGroup`
    * owners: a `UserGroup`

* `Team`:
    * memberOfTypes: `UserGroup`

### Action Types
* `pull`, `push`, `fork`, `add_reader`, `add_triager`, `add_writer`, `add_maintainer`, `add_admin`
  * principals: `User`
  * resources: `Repository`
* `delete_issue`, `edit_issue`, `assign_issue`
  * principals: `User`
  * resources: `Issue`

## Policies

### Actions for Readers:

`User` entities with a role of `Reader` should be able to fork and pull repositories. They can also delete and edit an issue provided they are its reporter.

```
permit (
  principal,
  action == Action::"pull",
  resource
)
when { principal in resource.readers };

permit (
  principal,
  action == Action::"fork",
  resource
)
when { principal in resource.readers };

permit (
  principal,
  action == Action::"delete_issue",
  resource
)
when { principal in resource.repo.readers && principal == resource.reporter };

permit (
  principal,
  action == Action::"edit_issue",
  resource
)
when { principal in resource.repo.readers && principal == resource.reporter };
```

#### Actions for Triagers:

`User` entities with a role of `Triager` should be able to assign an issue to any user.

```
permit (
  principal,
  action == Action::"assign_issue",
  resource
)
when { principal in resource.repo.triagers };
```

#### Action for Writers:

`User` entities with a role of `Writer` should be able to push to a repository and also edits its issues.

```
permit (
  principal,
  action == Action::"push",
  resource
)
when { principal in resource.writers };

permit (
  principal,
  action == Action::"edit_issue",
  resource
)
when { principal in resource.repo.writers };
```

#### Actions for Maintainers:

`User` entities with a role of `maintainer` can only delete issues.

```
permit (
  principal,
  action == Action::"delete_issue",
  resource
)
when { principal in resource.repo.maintainers };
```

#### Actions for Admins:

`User` entities with a role of `admin` can add users to the roles of a repository.
```
permit (
  principal,
  action in
    [Action::"add_reader",
     Action::"add_triager",
     Action::"add_writer",
     Action::"add_maintainer",
     Action::"add_admin"],
  resource
)
when { principal in resource.admins };
```

## Tests

We use the following entities for our tests, included in the `entities.json` file:
* There are 3 `User` entities, `User::"alice"`, `User::"bob"`, `User::"jane"`.
  *  Alice is a member of the `common_knowledge_writers` and `uncommon_knowledge_writers` user groups.
  *  Bob is a member of the `tiny_corp_owners` organization.
  *  Jane is a member of the `common_knowledge_maintainers` user group and the `team_that_can_read_everything` team.
* There are 3 `Repository` entities, `common_knowledge`, `uncommon_knowledge`, and `secret`.
* There are 3 `UserGroup` entities, `common_knowledge_writers`, `uncommon_knowledge_writers`, and `common_knowledge_maintainers` that are children of the `common_knowledge` and `uncommon_knowledge` repositories, respectively.
* The 1 `Organization` entity, `tiny_corp_owners`, has admin priveleges to all 3 repositories.
* The 1 `Team` entity, `team_that_can_read_everything`, has read priveleges to all 3 repositories.

Here are some authz requests to test, included in the `ALLOW` and `DENY` folders:
* Alice tries to pull common_knowledge: ALLOW because she has read permissions through the `common_knowledge_writers` user group.
* Alice tries to pull uncommon_knowledge: ALLOW because she has read permissions through the `uncommon_knowledge_writers` user group.
* Alice tries to push uncommon_knowledge: ALLOW because she has write permissions through the `uncommon_knowledge_writers` user group.
* Bob tries to push secret: ALLOW because he has write permissions through the `tiny_corp_owners` organization.
* Jane tries to pull secret: ALLOW because she has read permissions through the `team_that_can_read_everything` team.
* Alice tries to pull secret: DENY because she doesn't have read permissions.
* Alice tries to push secret: DENY because she doesn't have write permissions.
