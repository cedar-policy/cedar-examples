# Tag & role policies

Hypothetical example for a company ABC Technologies.

## Basic model

ABC Tech principals are `User`s. Users have one or more `Role`s. This is indicated in the entity hierarchy (the `User` is `in` each of their `Role`s). A role has a corresponding `Action` group whose name mentions the roles; the member `Action`s of the group are the actions that members of the role can carry out. For each role the user is a member of, the user has a set of associated _tags_. The associated tags are collected as optional attributes in the `User` entity, as discussed below.

ABC Tech resources are `Workspace` objects. Each of these objects has associated tags, also stored as optional attributes, similarly to users. At a high level: If a user is a member of a role, and their tags associated with the role _match_ the tags on a resource, then they are permitted to carry out any of the role's associated actions on that resource.

## Tag matching

ABC Tech tags are organized in to _tag groups_ -- the group name is effectively a tag key. Each group has a set of associated string values. For example, the tag group `country` might have potential values `Italy`, `Germany` and `USA`, and a workspace or user with this tag may have any or all of these potential values. The tag value `ALL` is special: It says that the associated tag should be considered as having all possible tag values. For example, if `country` had tag value `[ "ALL" ]` then it matches all of `Italy`, `Germany`, and `USA`.

A `User`'s tags are collected by its `allowedTagsForRole` attribute, which is a record. The record has one attribute for each possible role in the system; these attributes are optional since not all users will be in all roles. Each role attribute is itself a record, whose attributes are the names of all possible tag groups; as with roles, these attributes are optional since not all users will have all tags. Each tag group attribute is associated with a set of strings, i.e., the values for that tag.

A `Workspace`'s tags are collected in its `tags` attribute, which is itself a record whose attributes are the names of all possible tag groups. As with `User`s, each tag group attribute is optional, and is a set of strings that enumerate the tag's values.

A `User`'s tags for a particular role _match_ the tags on a `Workspace` when for all of the `User`'s tags, if the resource also has that tag then the resource's tag values are _all_ included in the `User`'s tag value. The tag value `"ALL"` is treated specially here: if either `User` or `Workspace` has it in a tag's values, then the tag is considered to match. Note that that this logic implicitly treats the case that the `User` does not have a tag as equivalent to having the tag mapped to `"ALL"`. (This is weird logic, yes, but that's what they want!)

## Policy and schema management

To implement this approach we use _dynamically created_ policies. In particular:

Each time we **create a new role**:
1. We update the schema to define a new optional attribute for that role in the `allowedTagsForRole` attribute in a `User`. We must also update the `action` definitions in the schema to include memberships in a newly created action group for the new role.
2. We add a new policy to the policy store, specialized to the new role. The policy is exactly the same as all existing policies, but with references to a role made to the newly created role name.

If we delete the role, we remove the corresponding policy and action and tags definitions in the schema.

Each time we **create a new tag**:
1. We update the schema to define that tag as an optional attribute in each role in the `allowedTagsForRole` attribute in a `User`, and in the `tags` attribute in a `Workspace`.
2. We update all existing policies to add an additional `when` clause that performs the tag matching logic for that tag. 

If we delete the tag we remove it from the schema, and we drop its `when` clause on all existing policies.

Creating, modifying, or deleting users, workspaces, and tag values has no effect on the existing policies or schemas.

## Examples

The example authorizations are for two users, Bob and Alice, on a single resource. Two are in the ALLOW subdirectory, and the third is in the DENY subdirectory:
- The authz `joe_read.json` is ALLOW per policy0 because Joe is a member of Role-A and the ReadWorkspace action is as well, and the tags match. Note that policy1 is not satisfied because even though ReadWorkspace is also included in Role-B's actions, Joe's tags for Role-B do not match.
- The authz `alice_read.json` is ALLOW per policy1 because Alice is a member of Role-B and the ReadWorkspace action is as well, and the tags match.
- The last authz `alice_update.json` is DENY by default because the action UpdateWorkspace action is not a Role-B action, and that's the only role Alice is in.

## Variations

If we added some features to Cedar, we could potentially make these policies easier to manage and/or easier to read.

Of all the changes presented below, the most compelling is **generalized templates**. With them, we can simplify the management of policies when tags change. In particular, we can have one template policy which is linked for each created role. When tags change, we simply update that template and all of the corresponding linked policies with also be updated. The other proposed changes are more minor improvements.

### Maps

If we had map types in Cedar, it would slightly simplify schema management. 

In particular, the `allowedTagsForRole` attribute in `User` would become a map of maps, rather than a bunch of nested, optional attributes. Likewise, `tags` in `Workspace` would be simplified:
```
entity User in [Role] {
  allowedTagsForRole: Map<String,Map<String,Set<String>>>,
    // Role -> (TagGroupName -> Values)
  ...
};
entity Workspace {
  tags: Map<String,Set<String>>,
  ...
};
```
With this change, you would not have to update the schema every time you added/removed a new new tag. 

But this might not be worth it: You still have to update policies when you add/remove a tag, and you still have to update both policies and the schema when you add/remove a role.

### Macros

With macros, the logic for tag matching could be abstracted, making policies easier to read. For example, here are the two policies in [policies.cedar](policies.cedar), rewritten to use macros:
```
def tagmatch(?role,?tag)
  principal.allowedTagsForRole has ?role &&
  (if principal.allowedTagsForRole[?role] has ?tag then
    if resource.tags has ?tag then
      principal.allowedTagsForRole[?role][?tag].contains("ALL") ||
      resource.tags[?tag].contains("ALL") ||
      principal.allowedTagsForRole[?role][?tag].containsAll(resource.tags[?tag])
    else true
  else true)
;

@id("Role-A policy")
permit (
    principal in Role::"Role-A",
    action in [Action::"Role-A Actions"],
    resource
) when {
    tagmatch("Role-A","production_status")
} when {
    tagmatch("Role-A","country")
} when {
    tagmatch("Role-A","stage")
};

@id("Role-B policy")
permit (
    principal in Role::"Role-B",
    action in [Action::"Role-B Actions"],
    resource
) when {
    tagmatch("Role-B","production_status")
} when {
    tagmatch("Role-B","country")
} when {
    tagmatch("Role-B","stage")
};
```

But this might not be worth it:

While the produced policies are easier to read, reading them would not happen often -- there could be little reason to look at them in console, since they are created on the fly. Moreover, it is unlikely that macros will help avoid mistakes. That's because policies already need to be constructed on the fly, and the host application can store `tagmatch` as a host application function that is called to generate the proper `when` clauses.

#### Generalized templates

We mentioned generalized templates above. Here's what the template looks like, for the macro version of the policy (if we didn't have macros, you could just expand the macros out to get the same effect):

```
@id("Role policy")
permit (
    principal in ?principal,
    action,
    resource
) when {
    action in ?action
} when {
    tagmatch(?role,"production_status")
} when {
    tagmatch(?role,"country")
} when {
    tagmatch(?role,"stage")
};
```
Here, the `?principal` is linked against the role group (e.g., `Role::"Role-A"`), the `?action` is linked against the role's action group (e.g., `Action::"Role-A Actions"`), and `?role` is linked against the string `"Role-A"`. If you updated this template, e.g., to update `when` clause to deal with a new tag, all policies linked against the template would be updated.

This is likely to be of real benefit, even without macros and/or maps.

### Generalized `any?`/`all?`

The three extensions mentioned above naturally "stack" to incrementally improve policies: Using maps means you only need to update schemas when you add/remove a role; using macros makes policies easier to read; and using generalized templates means that when you update tags, it's easier to update the relevant policies by updating a single template (and the schema).

A completely different way of approving the problem is possible with generalized iteration operators. TBD!