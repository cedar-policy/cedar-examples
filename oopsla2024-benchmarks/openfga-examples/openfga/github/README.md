# OpenFGA GitHub Sample Store

* **Title**: **GitHub** 
* **Documentation**: https://openfga.dev/docs/modeling/advanced/github
* **Playground**: https://play.fga.dev/sandbox/?store=github

## Table of Content
- [Use-Case](#use-case)
  - [Requirements](#requirements)
  - [Scenario](#scenario)
  - [Expected Outcomes](#expected-outcomes)
- [Modeling in OpenFGA](#modeling-in-openfga)
  - [Model](#model)
  - [Tuples](#tuples)
  - [Assertions](#assertions)
- [Try It Out](#try-it-out)

## Use-Case

### Requirements

This model is based on GitHub's permission model according to their [documentation](https://docs.github.com/en/organizations/managing-access-to-your-organizations-repositories/repository-roles-for-an-organization#repository-roles-for-organizations).

* Users can be admins, maintainers, writers, triagers or readers of repositories (each level inherits all access of the level lower than it. e.g. admins inherit maintainer access and so forth)
* Teams can have members
* Organizations can have members
* Organizations can own repositories
* Users can have repository admin access on organizations, and thus have admin access to all repositories owned by that organization

### Scenario

There are users, organizations, teams and repositories

- There are five users: Anne, Beth, Charles, Diane and Erik
- There is an OpenFGA organization that owns the openfga/openfga repository
- There is an openfga/core team and a openfga/backend team
- Members of the openfga/backend team are members of the openfga/core team
- Members of the openfga/core team are admins of the openfga/openfga repository
- Erik is a member of the OpenFGA organization
- Diane is a member of the openfga/backend team
- Charles is a member of the openfga/core team
- Anne is a reader on the openfga/openfga repository
- Beth is a writer on the openfga/openfga repository
- The OpenFGA organization has been configured with the "repository admin" base permission, which means all the organization members have the admin role on all the repositories the organization owns 

### Expected Outcomes

- Anne is a reader on the openfga/openfga repository
- Anne **is not** a triager on the openfga/openfga repository
- Diane is an admin on the openfga/openfga repository
- Erik is a reader on the openfga/openfga repository
- Charles is a writer on the openfga/openfga repository
- Beth **is not** an admin on the openfga/openfga repository

## Modeling in OpenFGA

### Model

```python
model
  # We are using the 1.1 schema with type restrictions
  schema 1.1

# There are users
type user

# there are organizations
type organization
  relations
    # Organizations can have users who own them
    define owner: [user]
    # Organizations can have members (any owner of the organization is automatically a member)
    define member: [user] or owner
    # Organizations has a set of base permissions, such as repository admin, writer and reader
    define repo_admin: [user, organization#member]
    define repo_reader: [user, organization#member]
    define repo_writer: [user, organization#member]

# there are teams
type team
  relations
    # teams have members
    define member: [user, team#member]

# there are repositories
type repo
  relations
    # repositories have organizations who own them
    define owner: [organization]
    # repository have admins, they can be assigned or inherited (anyone who has the repository admin role on the owner organization is an owner on the repo)
    define admin: [user, team#member] or repo_admin from owner
    # maintainers on a repo are anyone who is directly assigned or anyone who is an owner on the repo
    define maintainer: [user, team#member] or admin
    # repo writers are users who are directly assigned, anyone who is a maintainer or anyone who has the repository writer role on the owner organization
    define writer: [user, team#member] or maintainer or repo_writer from owner
    # triagers on a repo are anyone who is directly assigned or anyone who is a writer on the repo
    define triager: [user, team#member] or writer
    # repo readers are users who are directly assigned, anyone who is a triager or anyone who has the repository reader role on the owner organization
    define reader: [user, team#member] or triager or repo_reader from owner
```

> Note: The OpenFGA API accepts a JSON syntax for the authorization model that is different from the DSL shown above
>       To switch between the two syntaxes, you can use the [@openfga/syntax-transformer npm package](https://www.npmjs.com/package/@openfga/syntax-transformer) or the [Auth0 FGA Playground](https://play.fga.dev)

You can see a representation of this model in the JSON syntax accepted by the OpenFGA API in [authorization-model.json](./authorization-model.json).

### Tuples

| User                        | Relation   | Object               | Description                                                                                     |
|-----------------------------|------------|----------------------|-------------------------------------------------------------------------------------------------|
| organization:openfga        | owner      | repo:openfga/openfga | The OpenFGA organization is the owner of the openfga/openfga repository                         |
| organization:openfga#member | repo_admin | organization:openfga | Members of the OpenFGA organization have a repository admin base permission on the organization |
| user:erik                   | member     | organization:openfga | Erik is a member of the OpenFGA organization                                                    |
| team:openfga/core#member    | admin      | repo:openfga/openfga | The openfga/core team members are admins on the openfga/openfga repository                      |
| user:anne                   | reader     | repo:openfga/openfga | Anne is a reader on the openfga/openfga repository                                              |
| user:beth                   | writer     | repo:openfga/openfga | Beth is a writer on the openfga/openfga repository                                              |
| user:charles                | member     | team:openfga/core    | Charles is a member of the openfga/core team                                                    |
| team:openfga/backend#member | member     | team:openfga/core    | Members of the openfga/backend team are members of the openfga/core team                        |
| user:diane                  | member     | team:openfga/backend | Diane is a member of the openfga/backend team                                                   |

These are represented in this file: [tuples.json](./tuples.json).

### Assertions

| User         | Relation | Object               | Allowed? |
|--------------|----------|----------------------|----------|
| user:anne    | reader   | repo:openfga/openfga | Yes      |
| user:anne    | triager  | repo:openfga/openfga | No       |
| user:diane   | admin    | repo:openfga/openfga | Yes      |
| user:erik    | reader   | repo:openfga/openfga | Yes      |
| user:charles | writer   | repo:openfga/openfga | Yes      |
| user:beth    | admin    | repo:openfga/openfga | No       |

These are represented in this file: [assertions.json](./assertions.json).

## Try It Out

Use `github` as the SAMPLE_STORE, and follow the rest of the instructions on [Try it out section in the main README](https://github.com/openfga/sample-stores#try-it-out).
