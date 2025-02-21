# Hotel chain policies

This example details an authorization model with a role-based approach to managing a hotel chain, ABC.
For more information about exploring this example using the Cedar CLI, see [Cedar Example Use Cases](https://github.com/cedar-policy/cedar-examples/tree/release/4.0.x/cedar-example-use-cases).

## Use-case

ABC has one principal type, which is a `User`.

ABC has a hierarchy of resources. A `Hotel` represents a hotel chain or franchise. Chains can be nested in the hierarchy, i.e., a chain could have sub-chains. A `Property` represents an individual hotel location. Each property has resources associated with it, such as `Reservation`s, `PaymentDetails`, `Rates`, etc. These resources have the relevant `Property` as a parent in the hierarchy. In this example, we only include `Reservation`, but explain how to expand to include other types.

A `User` is assigned at most one of three _roles_ for accessing the resources associated with a `Hotel` or `Property`: _viewer_, _member_ (basically, an editor), and _admin_. The first two roles are scoped to _particular resources_ at that `Property`/`Hotel`. For example, a user Alice might only be allowed to view `Reservations` at a particular `Property` or `Hotel`, and not `PaymentDetails`. Someone with _admin_ role is permitted to access all associated resources. If a user has a role for particular resources at a property/hotel, they inherit that role for the whole property/hotel itself. For example, a _member_ of hotel `G` for `Reservations` is allowed to perform edit operations on `G` itself.

## Approaches

The `static/` directory contains policies and a schema that associates with each `User` (i.e., as a `User` attribute) a series of records called `PermissionsMap`s. The user has one permissions map for its viewer roles and one for its member roles. Each map has an attribute for each sort of resource type (reservations, payment details, etc.). That attribute's type is a set of `Hotel` or `Property` objects. Membership in the resource-type's set means the user with that permissions map is a viewer/member on resources in that property/hotel. Since _admin_ role gives access to all resources, the `User` has a separate property/hotel set for it.

The `templated/` directory uses Cedar templates instead. Each policy in the `static/` directory has a corresponding template policy. We drop the permissions maps entirely. When a user is given a viewer/member/admin role on resources for a particular property/hotel, we link a template for each resource to which the user is given access for the property/hotel, and we link a template for access to the property/hotel itself.

These are the only differences in the approaches.

Expand the approach below for more details.

<details>

<summary>Static</summary>

### Entities

#### `User`
Represents a user that has permissions for a property/hotel.

#### `Property`
Represents a single property in a hotel chain.

#### `Hotel`
Represents a hotel chain that's made up of one or more `Property` entities.

#### `Reservation`
Represents a reservation at a property.

### Actions

#### `viewReservation`, `updateReservation`, `grantAccessReservation`
Actions that can be taken on a reservation.
#### `createReservation`, `viewProperty`, `updateProperty`, `grantAccessProperty`
Actions that can be taken on a property.
#### `createProperty`, `createHotel`, `viewHotel`, `updateHotel`, `grantAccessHotel`
Actions that can taken on a hotel.

### Schema

#### Entity types
* `User`:
  * Attributes:
    * `viewPermissions`: a `PermissionsMap`
	* `memberPermissions`: `PermissionsMap`
	* `hotelAdminPermissions`: a set of `Hotel` entities
	* `propertyAdminPermissions`: a set of `Property` entities
* `Property`:
  * memberOfTypes: `Hotel`
* `Hotel`:
  * memberOfTypes: `Hotel`
* `Reservation`:
  * memberOfTypes: `Property`
  
#### Action types
* `viewReservation`, `updateReservation`, `grantAccessReservation`: Actions that can be taken on a reservation
  * principals: `User`
  * resources: `Reservation`
* `createReservation`, `viewProperty`, `updateProperty`, `grantAccessProperty`: Actions that can be taken on a property.
* principals: `User`
  * resources: `Property`
* `createProperty`, `createHotel`, `viewHotel`, `updateHotel`, `grantAccessHotel`: Actions that can taken on a hotel.
* principals: `User`
  * resources: `Hotel`

### Policies

#### Reservations
```
// Viewer permissions
permit(
  principal, 
  action in [Action::"viewReservation"],
  resource)
when {
   resource in principal.viewPermissions.hotelReservations ||
   resource in principal.viewPermissions.propertyReservations
};
```
```
// Member permissions
permit(
  principal, 
  action in [Action::"viewReservation",
             Action::"updateReservation",
             Action::"createReservation"],
  resource)
when {
  resource in principal.memberPermissions.hotelReservations ||
  resource in principal.memberPermissions.propertyReservations
};
```
```
// Admin permissions 
permit(
  principal, 
  action in [Action::"viewReservation",
             Action::"updateReservation",
             Action::"createReservation",
             Action::"grantAccessReservation"
             ],             
  resource)
when {
  resource in principal.hotelAdminPermissions ||
  resource in principal.propertyAdminPermissions
};
```

##### Hotels and properties
```
// Viewer permissions
permit(
  principal, 
  action in [Action::"viewProperty",
             Action::"viewHotel"],
  resource)
when {
  resource in principal.viewPermissions.hotelReservations ||
  resource is Property && resource in principal.viewPermissions.propertyReservations
// || resource in principal.viewPermissions.inventory ... for other resource types
};
```
```
// Member permissions
permit(
  principal, 
  action in [Action::"viewProperty",
             Action::"updateProperty",
             Action::"createProperty",
             Action::"viewHotel",
             Action::"updateHotel",
             Action::"createHotel"],
  resource)
when {
  resource in principal.memberPermissions.hotelReservations ||
  resource is Property && resource in principal.memberPermissions.propertyReservations
// || resource in principal.memberPermissions.inventory ... for other resource types
};
```
```
Admin permissions
permit(
  principal,
  action in [Action::"viewProperty",
             Action::"updateProperty",
             Action::"createProperty",
             Action::"grantAccessProperty",
             Action::"viewHotel",
             Action::"updateHotel",
             Action::"createHotel",
             Action::"grantAccessHotel"],
  resource)
when {
  resource in principal.hotelAdminPermissions ||
  resource is Property && resource in principal.propertyAdminPermissions
};
```

</details>

<details>

<summary>Templated</summary>

### Entities

#### `User`
Represents a user that has permissions for a property/hotel.

#### `Property`
Represents a single property in a hotel chain.

#### `Hotel`
Represents a hotel chain that's made up of one or more `Property` entities.

#### `Reservation`
Represents a reservation at a property.

### Actions

#### `viewReservation`, `updateReservation`, `grantAccessReservation`
Actions that can be taken on a reservation.
#### `createReservation`, `viewProperty`, `updateProperty`, `grantAccessProperty`
Actions that can be taken on a property.
#### `createProperty`, `createHotel`, `viewHotel`, `updateHotel`, `grantAccessHotel`
Actions that can taken on a hotel.

### Schema

#### Entity types

* `User`
* `Property`:
  * memberOfTypes: `Hotel`
* `Hotel`:
  * memberOfTypes: `Hotel`
* `Reservation`:
  * memberOfTypes: `Property`
  
#### Action types
* `viewReservation`, `updateReservation`, `grantAccessReservation`: Actions that can be taken on a reservation
  * principals: `User`
  * resources: `Reservation`
* `createReservation`, `viewProperty`, `updateProperty`, `grantAccessProperty`: Actions that can be taken on a property.
* principals: `User`
  * resources: `Property`
* `createProperty`, `createHotel`, `viewHotel`, `updateHotel`, `grantAccessHotel`: Actions that can taken on a hotel.
* principals: `User`
  * resources: `Hotel`

### Policies

#### Reservations

```
@id("ViewReservation")
permit(
  principal == ?principal,
  action in [Action::"viewReservation"],
  resource in ?resource);
  
@id("MemberReservation")
permit(
  principal == ?principal, 
  action in [Action::"viewReservation",
             Action::"updateReservation",
             Action::"createReservation"],
  resource in ?resource);

@id("AdminReservation")
permit(
  principal == ?principal,
  action in [Action::"viewReservation",
             Action::"updateReservation",
             Action::"createReservation",
             Action::"grantAccessReservation"],
  resource in ?resource);
  ```

##### Hotels and properties

```
@id("ViewPropertyOrHotel")
permit(
  principal == ?principal,
  action in [Action::"viewHotel",
             Action::"viewProperty"],
  resource in ?resource);
  
@id("MemberPropertyOrHotel")
permit(
  principal == ?principal, 
  action in [Action::"viewHotel",
             Action::"updateHotel",
             Action::"createHotel",
             Action::"viewProperty",
             Action::"updateProperty",
             Action::"createProperty"],
  resource in ?resource);

@id("AdminPropertyOrHotel")
permit(
  principal == ?principal,
  action in [Action::"viewHotel",
             Action::"updateHotel",
             Action::"createHotel",
             Action::"grantAccessHotel",
             Action::"viewProperty",
             Action::"updateProperty",
             Action::"createProperty",
             Action::"grantAccessProperty"],
  resource in ?resource);
  ```

</details>


## Tests

We use the following entities for our tests:
- There are two `Hotel` chains, named `G` and `R`.
- There are three `Property` elements: `Green` and `Gray` are part of the `G` chain, and `Red` is part of the `R` chain.
- There are three `Reservation`s: `Green-Res1`, `Gray-Res1`, and `Red-Res1`.
- There are two `User`s, named `Alice` and `Bob`. 

Our two users have the following permissions (expressed in the entities in the static policies, and as links in the templated ones):
  - Alice has _view_ permissions for reservations in hotel `G`, and _member_ permissions for reservations in property `Green`
  - Bob has _admin_ permissions for hotel `R` and property `Green`.

Here are some authz requests for these:
- Alice views Gray-Res1: ALLOW, since Alice has view permissions on G
- Alice updates Green-Res1: ALLOW, since Alice has member permissions on Green
- Alice updates Gray-Res1: DENY, since Alice does not have member permissions on Gray, only on Green
- Bob updates Gray (Property): DENY, since Bob has admin permissions on Green, not Gray
- Bob updates Red (Property): ALLOW, since Bob has admin permissions on R
- Bob views Green-Res1: ALLOW, since Bob has admin permissions on Green

<!--## Variations / wishes

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
We have to split up actions that have the same signatures, and we have to figure out the grouping relationships from `in`.-->
