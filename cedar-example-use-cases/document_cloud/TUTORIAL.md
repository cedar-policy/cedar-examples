# Document Cloud Drive Use-Case

Envision a cloud-based document sharing system, like Google Drive or Dropbox. This system can be used by a single user, who is working on documents across multiple of their personal computers, by multiple users, who are collaborating on a shared set of documents, or by the public as a hosting solution. Users need to be able upload, delete, and modify the sharing permissions on their documents. Users also need to be able to view, comment on, and and modify documents that they have access to, while the system enforces correct access control logic. Since this is a multi-tenant system, it must be robust to cross-user abuse. This system includes a blocklist feature to prevent that.

We first define the entities involved in this system.
## Entities

### `User`
`User`s are the main principals of the system. They are the ones who view/edit/delete documents, as well as the ones who control sharing permissions on documents. `User`s may also block other users. For instance, if Alice blocks Bob, then Bob should not be able to view any documents Alice owns, or share anything with Alice.

### `Group`
For convenience, `User`s can be organized into `Group`s. Documents can be shared with entire `Group`s at once. We’ll borrow from Unix and say that every `User` also has a `Group` containing only them.

### `Public`
A principal that represents an un-authenticated user.

### `Document`

`Document`s are the core resource of the system. Every document has an owner, which is the `User` who created it. Documents have 3 axis of sharing

* Private: only the owner can view/edit/comment/delete the document
* ACL: List of users/groups that are allowed view, groups allowed to comment, groups allowed to edit, groups allowed to manage.
* Public: Can the public view/edit/comment on the document

It is always enforced that only the owner can delete or edit the sharing state of a document

Next we define the actions that principals can perform.

## Actions

* `CreateDocument`: Create a new document in the system. Any authenticated user can do this.
* `ViewDocument`: Must pass ACL check
* `DeleteDocument`: Only the owner should be able do this.
* `CommentOnDocument`: Must pass ACL check
* `ModifyDocument`: Must pass ACL check
* `EditIsPrivate`: Only the owner can do this
* `AddToShareACL`: Anyone who has manage access can do this. The owner can never have their access be revoked.
* `EditPublic`: Anyone who has edit access can do this.
* `CreateGroup`: Any authenticated user can do this
* `ModifyGroup`: Only the owner of the group can do this
* `DeleteGroup`: Only the owner of the group can do this


Let’s take this and turn it into a concrete schema:

### Entity Types:

* `User`
    * attributes
        * `personalGroup` a EUID of type `Group`, links to the group containing exactly this user
        * `blocked` a set of EUIDs of type `User`.
    * memberOf: `Group`, can be a member of any group. 
    * Invariants: 
        * for any `User` `u`, `u in u.personalGroup` should always be true
            * as well as `u == u.personalGroup.owner`
* `Group`
    * attributes:
        * `owner`: a EUID of type `User`, denoting who can manage this group
    * memberOf: `DocumentShare`
* `Document`
    * attributes:
        * `owner` an EUID of type `User`
        * `isPrivate` a boolean
        * `publicAccess` a string, one of: `none`, `view`, or `edit`
        * `viewACL` an EUID of type `DocumentShare`
        * `modifyACL` an EUID of type `DocumentShare`
        * `manageACL` an EUID of type `DocumentShare`
* `DocumentShare`
    * A group-entity. All children of this entity are `group`s that are allowed to perform some action on a document
* `Public`
    * There is exactly one instance of `public` that represents the un-authenticated user.
    * memberOf: `DocumentShare`
* `Drive`
    * A “container entity” that represents the entire application.

### Action Types

* `CreateDocument`: Create a new document in the system. Any authenticated user can do this.
    * principals: `User`
    * resources: `Drive`
* `ViewDocument`: Must pass ACL check
    * principals: `User`
    * resources: `Document`
* `DeleteDocument`: Only the owner should be able do this.
    * principals: `User`
    * resources: `Document`
* `ModifyDocument`: Must pass ACL check
    * principals: `User`
    * resources: `Document`
* `EditIsPrivate`: Only the owner can do this
    * principals: `User`
    * resources: `Document`
* `AddToShareACL`: Anyone who has manage access can do this. The owner can never have their access be revoked.
    * principals: `User`
    * resources: `Document`
* `EditPublicAccess`: Anyone who has manage access can do this.
    * principals: `User`
    * resources: `Document`
* `CreateGroup`: Any authenticated user can do this
    * principals: `User`
    * resources: `Drive`
* `ModifyGroup`: Only the owner of the group can do this
    * principals: `User`
    * resources: `Group`
* `DeleteGroup`: Only the owner of the group can do this
    * principals: `User`
    * resources: `Group`


Finally, let's look at the policies for permission management.
## Policies

### Creating Documents:


```
permit (
  principal,
  action == Action::"CreateDocument",
  resource == Drive::"drive"
);
```

Any authenticated user should be able to make a document. Since the only valid principal-type here is `User`, this accomplishes that. However, the “authenticated” part isn’t anywhere *in* the policy, and isn’t checked at runtime.
There are a couple of solutions here:

1. Create an entity `Users::"AllUsers"` that every user is a part of. This makes the graph rather big, but maybe we don’t care.
2. An `is` operator, ex: `principal is User`
3. Runtime enforcement of action types.

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

If you’ve blocked someone, they can’t see any of your documents and you can’t see any of theirs. Note that we need a constraint `principal has blocked` for this policy to pass validation because `Action::"ViewDocument"` applies to either `User` or `Public` whereas the blocked list only contains `User`s. This constraint rules out the scenario where `principal` is a `Public`.

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

This forbid policy is a guard-rail. We could enforce it at runtime, or prove that the other policies implement it:

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