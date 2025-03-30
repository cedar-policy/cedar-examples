# Document Cloud Drive Use-Case

This examnple explores the authorizazation model for a cloud-based sharing system.

For more information about exploring this example using the Cedar CLI, see [Cedar Example Use Cases](https://github.com/cedar-policy/cedar-examples/tree/release/4.0.x/cedar-example-use-cases).

## Use-case

Envision a cloud-based document sharing system, like Google Drive or Dropbox. This system can be used by a single user who is working on documents across multiple computers, by multiple users who are collaborating on a shared set of documents, or by the public as a hosting solution. Users need to be able to upload, delete, and modify the sharing permissions on their documents. Users also need to be able to view, comment on, and modify documents that they have access to. The system enforces correct access control logic. Since this is a multi-tenant system, it must have  a mechanism to protect against cross-user abuse. This system includes a blocklist feature to accomplish this.

## Entities

### `User`
Users are the main principals of the system. They are the ones who view/edit/delete documents, as well as the ones who control sharing permissions on documents. They may also block other users. For instance, if Alice blocks Bob, then Bob should not be able to view any documents Alice owns or share anything with Alice.

### `Group`
For convenience, users can be organized into groups. Documents can be shared with entire groups at once. We’ll borrow from Unix and say that every user also has a group containing only them.

### `Public`
A principal that represents an unauthenticated user.

### `Document`

Documents are the core resource of the system. Every document has an owner, which is the user who created it. Documents have 3 methods of sharing

* Private: only the owner can view/edit/comment/delete the document
* Access-control List (ACL): List of users/groups that are allowed view, groups allowed to comment, groups allowed to edit, groups allowed to manage.
* Public: The public can view/edit/comment on the document.

It is always enforced that only the owner can delete or edit the sharing state of a document.

## Actions

### `CreateDocument`
Create a new document in the system. Any authenticated user can do this.

### `ViewDocument`
Users must be on the ACL for the document.

### `DeleteDocument`
Only the owner of the document can do this.

### `CommentOnDocument`
Users must be on the ACL for the document.

### `ModifyDocument`
Users must be on the ACL for the document.

### `EditIsPrivate`
Only the owner of the document can do this.

### `AddToShareACL`
Users who have manage access can do this. The owner can never have their access be revoked.

### `EditPublic`
Anyone who has edit access can do this.

### `CreateGroup`
Any authenticated user can do this.

### `ModifyGroup`
Only the owner of the document can do this.

### `DeleteGroup`
Only the owner of the document can do this.

## Context
### `is_authenticated`
Whether or not the request is from an authenticated user

## Schema

### Entity Types:

* `DocumentShare`
    * A group-entity. All children of this entity are `group`s that are allowed to perform some action on a document.
* `User`
    * attributes
        * `personalGroup` a `Group`, links to the group containing exactly this user
        * `blocked` a set of  `User` entities.
    * memberOfTypes: `Group`, can be a member of any group. 
    * Invariants: 
        * for any `User` `u`, `u in u.personalGroup` should always be true
            * as well as `u == u.personalGroup.owner`
* `Group`
    * attributes:
        * `owner`: a `User`
    * memberOfTypes: `DocumentShare`
* `Document`
    * attributes:
        * `owner`: a `User`
        * `isPrivate`: a boolean
        * `publicAccess`: a string, one of: `none`, `view`, or `edit`
        * `viewACL` a `DocumentShare`
        * `modifyACL` a `DocumentShare`
        * `manageACL` a `DocumentShare`
* `Public`
    * There is exactly one instance of `public` that represents the un-authenticated user.
    * memberOfTypes: `DocumentShare`
* `Drive`
    * A “container entity” that represents the entire application.

### Action Types

* `CreateDocument`: Create a new document in the system. Any authenticated user can do this.
    * principals: `User`
    * resources: `Drive`
* `ViewDocument`: Users must be on the ACL for the document.
    * principals: `User`
    * resources: `Document`
* `DeleteDocument`: Only the owner of the document can do this.
    * principals: `User`
    * resources: `Document`
* `ModifyDocument`: Users must be on the ACL for the document.
    * principals: `User`
    * resources: `Document`
* `EditIsPrivate`: Only the owner of the document can do this.
    * principals: `User`
    * resources: `Document`
* `AddToShareACL`: Users who have manage access can do this. The owner can never have their access be revoked.
    * principals: `User`
    * resources: `Document`
* `EditPublicAccess`: Anyone who has edit access can do this.
    * principals: `User`
    * resources: `Document`
* `CreateGroup`: Any authenticated user can do this.
    * principals: `User`
    * resources: `Drive`
* `ModifyGroup`: Only the owner of the document can do this.
    * principals: `User`
    * resources: `Group`
* `DeleteGroup`: Only the owner of the document can do this.
    * principals: `User`
    * resources: `Group`

### Context

* `is_authenticated`
    * Attributes:
      * type: a boolean

## Policies

### Creating Documents:


```
permit (
  principal,
  action == Action::"CreateDocument",
  resource == Drive::"drive"
);
```

### Viewing Documents

The owner should always be able to view the document.

```
permit (
  principal,
  action == Action::"ViewDocument",
  resource
)
when { principal == resource.owner };
```

Anyone who is in the view ACL should be able to view the document, when it’s not private

```
permit (
  principal,
  action == Action::"ViewDocument",
  resource
)
when { principal in resource.viewACL }
unless { resource.isPrivate };
```

An un-authenticated principal can view only when explicitly permitted

```
permit (
  principal == Public::"public",
  action == Action::"ViewDocument",
  resource
)
when { resource.publicAccess == "view" || resource.publicAccess == "edit" }
unless { resource.isPrivate };
```

### Delete Document

Simiarly, only the owner can do this

```
permit (
  principal,
  action == Action::"deleteDocument",
  resource
)
when { principal == resource.owner };
```

### Modify Document

Very similar to viewing, just different ACL:

```
permit (
  principal,
  action == Action::"ModifyDocument",
  resource
)
when { principal == resource.owner };

permit (
  principal,
  action == Action::"ModifyDocument",
  resource
)
when { principal in resource.modifyACL }
unless { resource.isPrivate };

permit (
  principal == Public::"public",
  action == Action::"ViewDocument",
  resource
)
when { resource.publicAccess == "edit" }
unless { resource.isPrivate };
```

### Document Management

```
permit (
  principal,
  action in
    [Action::"EditIsPrivate",
     Action::"AddToShareACL",
     Action::"EditPublicAccess"],
  resource
)
when { principal == resource.owner };

permit (
  principal,
  action in [Action::"AddToShareACL", Action::"EditPublicAccess"],
  resource
)
when { principal in resource.manageACL };
```

### Group Management

```
permit (
  principal,
  action == Action::"CreateGroup",
  resource == Drive::"drive"
);

permit (
  principal,
  action in [Action::"ModifyGroup", Action::"DeleteGroup"],
  resource
)
when { principal == resource.owner };
```

### Blocking

If you’ve blocked someone, they can’t see any of your documents and you can’t see any of theirs. 
**Note**: We need a constraint `principal has blocked` for this policy to pass validation because `Action::"ViewDocument"` applies to either `User` or `Public` whereas the blocked list only contains `User` entities. This constraint rules out the scenario where `principal` is a `Public`.

```
forbid (
  principal,
  action in
    [Action::"ViewDocument",
     Action::"ModifyDocument",
     Action::"EditIsPrivate",
     Action::"AddToShareACL",
     Action::"EditPublicAccess",
     Action::"DeleteDocument"],
  resource
)
when
{
  principal has blocked &&
  (resource.owner.blocked.contains(principal) ||
   principal.blocked.contains(resource.owner))
};
```

### Guard Rails

This forbid policy is a guard rail. We could enforce it at runtime, or prove that the other policies implement it:

```
forbid (principal, action, resource)
when
{
  resource has owner &&
  principal != resource.owner &&
  resource has isPrivate &&
  resource.isPrivate
};
```

### Authentication Context

This forbid policy requires that requests contain a valid authentication context:

```
forbid (
  principal,
  action,
  resource
)
when { !context.is_authenticated };
```

## Tests

We use the following entities for our tests, included in the `entities.json` file:

* There are 3 `User` entities, `User::"alice"`, `User::"bob"`, `User::"charlie"`.
  *  Alice is a member of the `alice_personal` user group and has blocked Bob.
  *  Bob is a member of the `bob_personal` user group.
  *  Charlie is a member of the `charlie_personal` user group.
* There are 3 `UserGroup` entities, `alice_personal`, `bob_personal`, and `charlie_personal`.
* There is 1 `Drive` entity, `Drive::"drive"`.
* There is 1 `DocumentShare` entity, `alice_public_view`. Both `bob_personal` and `charlie_personal` are children of this entity.  
**Note**: `alice_personal` doesn't need to be a child of this because Alice owns the documents she creates by default.
* There is 1 `Document` entity, `alice_public`. 

Here are some authz requests to test, included in the `ALLOW` and `DENY` folders:

* Alice tries to create a document in drive: ALLOW because she is authenticated.
* Alice tries to view alice_public: ALLOW because she owns the document.
* Charlie tries to view alice_public: ALLOW because `charlie_personal` is in `alice_public_view`.
* Alice tries to create a document in drive: DENY because she is not authenticated.
* Bob tries to view alice_public: DENY because, even though `bob_personal` is in `alice_public_view`, Alice has blocked him.

