# Document Cloud Drive Use-Case

Envision a cloud-based document sharing system, like Google Drive or Dropbox. This system can be used by a single user, who is working on documents across multiple of their personal computers, by multiple users, who are collaborating on a shared set of documents, or by the public as a hosting solution. Users need to be able upload, delete, and modify the sharing permissions on their documents. Users also need to be able to view, comment on, and and modify documents that they have access to, while the system enforces correct access control logic. Since this is a multi-tenant system, it must be robust to cross-user abuse. This system includes a block list feature to prevent that.

We first define the entities involved in this system.
## Entities

### `User`
`User`s are the main principals of the system. They are the ones who view/edit/delete documents, as well as the ones who control sharing permissions on documents. `User`s may also block other users. For instance, if Alice blocks Bob, then Bob should not be able to view any documents Alice owns, or share anything with Alice.

We assume that all `User`s are authenticated before requesting any authorization.

### `Group`
For convenience, `User`s can be organized into `Group`s. Documents can be shared with entire `Group`s at once. We’ll borrow from Unix and say that every `User` also has a `Group` containing only them.

### `Document`

`Document`s are the core resource of the system. Every document has an owner, which is the `User` who created it. `Document` uses access control lists (ACLs) to manage permissions for action types such as view, edit, and manage.
An ACL is modeled as a `Group`.

It is always enforced that only the owner can delete or edit the sharing state of a document

Next we define the actions that principals can perform.

## Actions

* `createDocument`: Create a new document in the system.
* `viewDocument`: View a document.
* `deleteDocument`: Delete a document.
* `modifyDocument`: Modify a document.
* `addToShareACL`: Share a document.
* `createGroup`: Create a group.
* `modifyGroup`: Modify a group.
* `deleteGroup`: Delete a group.


Let’s take this and turn it into a concrete schema:

### Entity Types:

* `User`
    * attributes
        * `blocked` a set of EUIDs of type `User`.
    * memberOf: `Group`, can be a member of any group. 
* `Group`
    * attributes:
        * `owner`: a EUID of type `User`, denoting who can manage this group
* `Document`
    * attributes:
        * `owner` an EUID of type `User`
        * `publicAccess` a string, one of: `none`, `view`, or `edit`
        * `viewACL` an EUID of type `Group`
        * `modifyACL` an EUID of type `Group`
        * `manageACL` an EUID of type `Group`
* `Drive`
    * A “container entity” that represents the entire application.

### Action Types

* `createDocument`: Create a new document in the system. Any authenticated user can do this.
    * principals: `User`
    * resources: `Drive`
* `viewDocument`: Must pass ACL check
    * principals: `User`
    * resources: `Document`
* `deleteDocument`: Only the owner should be able do this.
    * principals: `User`
    * resources: `Document`
* `modifyDocument`: Must pass ACL check
    * principals: `User`
    * resources: `Document`
* `addToShareACL`: Anyone who has manage access can do this. The owner can never have their access be revoked.
    * principals: `User`
    * resources: `Document`
* `createGroup`: Any authenticated user can do this
    * principals: `User`
    * resources: `Drive`
* `modifyGroup`: Only the owner of the group can do this
    * principals: `User`
    * resources: `Group`
* `deleteGroup`: Only the owner of the group can do this
    * principals: `User`
    * resources: `Group`


Finally, let's look at the policies for permission management.
## Policies

### Creating Documents:


```
@id("drive-owner")
permit (
    principal,
    action,
    resource is Drive
)
when { resource.owner == principal };
```

Any `User` should be able to perform any action on a `Drive`, including `createDocument`.

### Viewing Documents

The owner should always be able to perform any action on a `Document`, including `viewDocument`.
```
@id("document-owner")
permit (
    principal,
    action,
    resource is Document
)
when { principal == resource.owner };
```

Anyone who is in the view ACL should be able to view the document

```
@id("viewACL")
permit (
    principal,
    action == Action::"viewDocument",
    resource
)
when { principal in resource.viewACL };
```

Any `User` can view a `Document` when it's publicly readable and editable.

```
@id("public-view")
permit (
    principal,
    action == Action::"viewDocument",
    resource
)
when { resource.publicAccess == "view" || resource.publicAccess == "edit" };
```

### Delete Document

Only the owner can perform this action, indicated by the policy with annotation `id@("document-owner")`.

### Modify Document

Very similar to viewing, just different ACL:

```
@id("modifyACL")
permit (
    principal,
    action == Action::"modifyDocument",
    resource
)
when { principal in resource.modifyACL };

@id("public-edit")
permit (
    principal,
    action == Action::"modifyDocument",
    resource
)
when { resource.publicAccess == "edit" };
```

### Document Management

A `User` can share a `Document` if they are in the `manageACL` group.
```
@id("manageACL")
permit (
    principal,
    action in Action::"addToShareACL",
    resource
)
when { principal in resource.manageACL };
```

### Group Management

Similar to `createDocument`, any `User` can perform group management actions as long as they own the group.
```
@id("group-owner")
permit (
    principal,
    action,
    resource is Group
)
when { resource has owner && principal == resource.owner };
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