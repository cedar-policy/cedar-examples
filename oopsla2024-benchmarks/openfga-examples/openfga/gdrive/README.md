# OpenFGA Google Drive Sample Store

* **Title**: **Google Drive** 
* **Documentation**: https://openfga.dev/docs/modeling/advanced/gdrive
* **Playground**: https://play.fga.dev/sandbox/?store=gdrive

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

> Note: This is only a contrived subset of Google Drive's permissions meant to showcase how a system like Google Drive could be modeled

### Requirements

- There are users, groups, folders and documents
- Users can be members of groups
- Folders have owners
- Folders can have parent folders
- Folders have viewers, a folder's viewers are whoever has been directly granted viewer access, the viewers of the document's parent folders, or the owners of the document
- Users need to have the create file permission on a folder in order to create a file inside it (only the folder owner has this permission)
- Documents have owners
- Documents can have parent folders
- Documents can have viewers
- Only a document's direct owner can change ownership
- A document's owner or owners of the document's parent folder can share the document
- A document's owner or owners of the document's parent folder can write to the document
- The users who can read a document are those with viewer access to the document, those with viewer access to the document's parent folder or the document's owner

### Scenario

- Anne is a member of the Contoso group
- Beth is a member of the Contoso group
- Charles is a member of the Fabrikam group
- The "Product 2021" folder contains the "Public Roadmap" document
- The "Product 2021" folder contains the "2021 Roadmap" document
- Members of the Fabrikam group are viewers of the "Product 2021" folder
- Anne is an owner of the "Product 2021" folder
- Beth is a viewer of the "2021 Roadmap" document
- Every user is a viewer of the "Public Roadmap" document

### Expected Outcomes

- Anne can write to the "2021 Roadmap" document
- Beth **cannot** change the owner of the "2021 Roadmap" document
- Charles can read the "2021 Roadmap" document
- Charles **cannot** write to the "2021 Roadmap" document
- Daniel **cannot** read the "2021 Roadmap" document
- Daniel can read the "Public Roadmap" document
- Anne can write to the "Public Roadmap" document
- Charles **cannot** write the "Public Roadmap" document

## Modeling in OpenFGA
### Model

```python
model
  # We are using the 1.1 schema with type restrictions
  schema 1.1
  
# There are users
type user

# there are groups
type group
  relations
    # a group can have members who are of type user
    define member: [user]

# there are folders
type folder
  relations
    # folders can have owners (of type user)
    define owner: [user]
    # folders can have parent folders
    define parent: [folder]
    # folders can have viewers; viewers are:
    # - those with whom the folder has been directly shared, these can be
    #   - objects of type user
    #   - all users (the special `type:*` syntax means all objects of that type)
    #   - members of groups (`group#member` is specifying that sets of users who are related to groups as members can be related as viewers),
    #     e.g. group:marketing#member, means members of the marketing group
    # - those who are ownwers of the folder (owner)
    # - those who are viewers of the parent of the folder (viewer from parent)
    define viewer: [user, user:*, group#member] or owner or viewer from parent
    # folders have the create file permissions; only owners can have this permission and it cannot be directly granted
    define can_create_file: owner
    
# there are documents
type doc
  relations
    # documents have owners (of type user)
    define owner: [user]
    # documents have parent folders
    define parent: [folder]
    # documents have viewers
    define viewer: [user, user:*, group#member]
    # documents have the change owner permission; only owners can have this permission and it cannot be directly granted
    define can_change_owner: owner
    # documents have the share permission; only owners or the owners of the parent folder (owner from parent) have this permissions and it cannot be directly granted
    define can_share: owner or owner from parent
    # documents have the write permission; only owners or the owners of the parent folder (owner from parent) have this permissions and it cannot be directly granted
    define can_write: owner or owner from parent
    # documents have the read permission; only direct viewers, direct owners or viewers of the parent folder have this permissions and it cannot be directly granted
    define can_read: viewer or owner or viewer from parent
```

> Note: The OpenFGA API accepts a JSON syntax for the authorization model that is different from the DSL shown above
>       To switch between the two syntaxes, you can use the [@openfga/syntax-transformer npm package](https://www.npmjs.com/package/@openfga/syntax-transformer) or the [Auth0 FGA Playground](https://play.fga.dev)

You can see a representation of this model in the JSON syntax accepted by the OpenFGA API in [authorization-model.json](./authorization-model.json).

### Tuples

| User                  | Relation | Object              | Description                                                            |
|-----------------------|----------|---------------------|------------------------------------------------------------------------|
| anne                  | member   | group:contoso       | Anne is a member of the Contoso group                                  |
| beth                  | member   | group:contoso       | Beth is a member of the Contoso group                                  |
| charles               | member   | group:fabrikam      | Charles is a member of the Fabrikam group                              |
| folder:product-2021   | parent   | doc:public-roadmap  | The "Product 2021" folder contains the "Public Roadmap" document       |
| folder:product-2021   | parent   | doc:2021-roadmap    | The "Product 2021" folder contains the "2021 Roadmap" document         |
| group:fabrikam#member | viewer   | folder:product-2021 | Members of the Fabrikam group are viewers of the "Product 2021" folder |
| anne                  | owner    | folder:product-2021 | Anne is an owner of the "Product 2021" folder                          |
| beth                  | viewer   | doc:2021-roadmap    | Beth is a viewer of the "2021 Roadmap" document                        |
| user:*                | viewer   | doc:public-roadmap  | Every user is a viewer of the "Public Roadmap" document                |

These are represented in this file: [tuples.json](./tuples.json).

### Assertions

| User    | Relation     | Object             | Allowed? |
|---------|--------------|--------------------|----------|
| anne    | write        | doc:2021-roadmap   | Yes      |
| beth    | change_owner | doc:2021-roadmap   | No       |
| charles | read         | doc:2021-roadmap   | Yes      |
| charles | write        | doc:2021-roadmap   | No       |
| daniel  | read         | doc:2021-roadmap   | No       |
| daniel  | read         | doc:public-roadmap | Yes      |
| anne    | write        | doc:public-roadmap | Yes      |
| charles | write        | doc:public-roadmap | No       |

These are represented in this file: [assertions.json](./assertions.json).

## Try It Out

Use `gdrive` as the SAMPLE_STORE, and follow the rest of the instructions on [Try it out section in the main README](https://github.com/openfga/sample-stores#try-it-out).
