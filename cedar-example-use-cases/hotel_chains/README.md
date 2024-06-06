# Hotel chains policies

This example considers a hotel chain, called ABC, which has franchises, properties, etc. It uses a role-based approach for managing hierarchically structured resources.

## Use-case

ABC has one principal type, which is a `User`.

ABC has a hierarchy of resources. A `Hotel` represents a hotel chain or franchise. Chains can be nested in the hierarchy, i.e., a chain could have sub-chains. A `Property` represents an individual hotel location. Each property has resources associated with it, such as `Reservation`s, `PaymentDetails`, `Rates`, etc. These resources have the relevant `Property` as a parent in the hierarchy. In our sample encoding, we just formalize `Reservation`, but indicate how to expand to include other types.

A `User` is assigned at most one of three _roles_ for accessing the resources associated with a `Hotel` or `Property`: _viewer_, _member_ (basically, an editor), and _admin_. The first two roles are scoped to _particular resources_ at that `Property`/`Hotel`, e.g., a user Alice might only be allowed to view `Reservations` at a particular `Property` or `Hotel` chain, and not `PaymentDetails`. Someone with _admin_ role is permitted to access all associated resources. If a user has a role for particular resources at a property/hotel, they inherit that role for the property/hotel itself. For example, a _member_ of hotel `Ibis` for `Reservations` is allowed to perform edit operations on `Ibis` itself. (The latter was not spelled out in the use-case writeup I saw, so I am extrapolating.)

## Approaches

The `static/` directory contains policies and a schema for that associates with each `User` (i.e., as a `User` attribute) a series of records called `PermissionsMap`s. The user has one permissions map for its viewer roles and one for its member roles. Each map has an attribute for each sort of resource type (reservations, payment details, etc.). That attribute's type is a set of `Hotel` or `Property` objects. Membership in the resource-type's set means the user with that permissions map can is a viewer/member on resources in that `Hotel`/`Property`. Since _admin_ role gives access to all resources, the `User` has a separate `Hotel`/`Property` set for it.

The `templated/` directory uses Cedar templates instead. Each policy in the `static/` directory has a corresponding template policy. We drop the permissions maps entirely. When a user is given viewer/member/admin role on resources for a particular `Property`/`Hotel`, we link a template for each resource to which the user is given access for the `Property`/`Hotel`, and we link a template for access to the `Property`/`Hotel` itself.

These are the only differences in the encodings.

## Tests

We use the following entities for our tests:
- There are two `Hotel` chains, named `G` and `R`.
- There are three `Property` elements: `Green` and `Gray` are part of the `G` chain, and `Red` is part of the `R` chain.
- There is one `Reservation`: `Green-Res1`, `Gray-Res1`, and `Red-Res1`.
- There are two `User`s, named `Alice` and `Bob`. 

Our two users have the following permissions (expressed in the entities in the static policies, and as links in the templated ones):
  - Alice has _view_ permissions for reservations in hotel `G`, and _member_ permissions for reservations in property `Green`
  - Bob has _admin_ permissions for hotel `R` and property `Green`.

Here are some authz requests for these:
- Alice views Gray-Res1: Allow, since Alice has view permissions on G
- Alice updates Green-Res1: Allow, since Alice has member permissions on Green
- Alice updates Gray-Res1: Deny, since Alice does not have member permissions on Gray, only on Green
- Bob updates Gray (Property): Deny, since Bob has admin permissions on Green, not Gray
- Bob updates Red (Property): Allow, since Bob has admin permissions on R
- Bob views Green-Res1: Allow, since Bob has admin permissions on Green

## Variations / wishes

It would be nice if you could **define action group memberships with the group**, rather than the action. If we could do that, we'd adjust the schema to add these definitions:
```
action memberReservations = [viewReservation,updateReservation,createReservation];
action adminReservations = [memberReservations,grantAccessReservation];
```
Then we see the relationship between the groups (notice that `adminReservations` includes `memberReservations`), and we can write more compact policies like the following.
```
permit(
  principal, 
  action in Action::"adminReservations",
  resource)
when {
  resource in principal.hotelAdminPermissions ||
  resource in principal.propertyAdminPermissions
};
```
Without such a way to define groups, it would complicate the definition of individual actions, in terms of making them harder to understand:
```
action adminReservations;
action memberReservations in [adminReservations];
action viewReservation, updateReservation in [memberReservations]
  appliesTo {
    principal: User,
    resource: Reservation,
  };
action grantAccessReservation in [adminReservations]
  appliesTo {
    principal: User,
    resource: Reservation,
  };
action createReservation in [memberReservations]
  appliesTo {
    principal: User,
    resource: Property,
  };

action viewProperty, updateProperty, grantAccessProperty ...
```
We have to split up actions that have the same signatures, and we have to figure out the grouping relationships from `in`.