# GitHub Model

In this tutorial, we show how you can implement GitHub's repository permission access in Cedar. There are three “views” of GitHub permissions, corresponding to *Personal*, *Organization* and *Enterprise* accounts. We’ll focus on the *Organizational* view. Our model won’t be complete since this is a tutorial. We’ll aim for a subset of features that you are probably familiar with.

## Entities

### `User`, `Team`, and `UserGroup`

`User` and `Team` represent GitHub users and teams, respectively. `UserGroup` denotes a set of `User`s, so a `User` or a `Team` can be in a `UserGroup`. Entities of type `UserGroup` can also form a hierarchy.

### `Repository`

`Repository` represents GitHub repositories and is the main resource of our model. A repository has five access roles:
* reader
* triager
* writer
* maintainer
* admin

### `Issue`

An issue of a `Repository` is reported by a `User`. A `User` can assign, create, and delete an issue.

### `Org`

It reprents a GitHub organization that has two groups of `User`s representing its members and owners, respectively.

## Actions
* `pull`: pull a repository
* `push`: push a repository
* `fork`: fork a repository
* `add_{reader, triager, writer, maintainer, admin}`: add a user to the specified role of a repository
* `{create, delete, assign}_issue`: perform the specified operation to an issue

Putting these together, we have the follwing schema.
##  Schema

### Entity Types
* `User`:
  * memberOfTypes: `UserGroup`, `Team`

* `UserGroup`:
  * memberOfTypes: `UserGroup`

* `Repository`:
  * attributes:
    * `readers` a `UserGroup`
    * `traigers`: a `UserGroup`
    * `writers`: a `UserGroup`
    * `maintainers`: a `UserGroup`
    * `admins`: a `UserGroup`

* `Issue`:
  * attributes:
    * repo: a `Repository`
    * reporter: a `User`
  
* `Org`:
  * attributes:
    * members: a `UserGroup`
    * owners: a `UserGroup`

* `Team`:
    * memberOfTypes: `UserGroup`

### Action Types
* `pull`, `push`, `fork`, `add_reader`, `add_triager`, `add_writer`, `add_maintainer`, `add_admin`: operations on repositories
    * principals: `User`
    * resources: `Repository`
* `delete_issue`, `edit_issue`, `assign_issue`: operations on issues
    * principals: `User`
    * resources: `Issue`

Finally, let's look at policies we can write to manage repository permissions.

## Policies

#### Actions for Readers:

`User`s of role `Reader` should be able to fork and pull repositories. They can also delete and edit an issue provided they are its reporter.

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

A `User` of role `Triager` should be able to assign an issue to any user.

```
permit (
  principal,
  action == Action::"assign_issue",
  resource
)
when { principal in resource.repo.triagers };
```

#### Action for Writers:

A `User` of role `Writer` should be able to push to a repository and also edits its issues.

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

A maintainer can only delete issues.

```
permit (
  principal,
  action == Action::"delete_issue",
  resource
)
when { principal in resource.repo.maintainers };
```

#### Actions for Admins:

An admin can add users to the roles of a repository.
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