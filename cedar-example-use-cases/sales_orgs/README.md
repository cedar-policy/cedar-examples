# Sales org policies

In this example, ABC company has a contracted sales force, and sales folk have access to resources based on their role and the type of resource.

For more information about exploring this example using the Cedar CLI, see [Cedar Example Use Cases](https://github.com/cedar-policy/cedar-examples/tree/release/4.0.x/cedar-example-use-cases).

## Use-case

ABC has one principal type, which is a `User`. Users are distinguished by their `job` (an attribute), which is an enumeration (enum). The job can be _internal_,  _distributor_, _customer_, or _other_. A customer is assigned to a particular distributor if it shares the distributor's customer ID. Users can be a member of zero or more `Market`s.

ABC has two protected resources, `Presentation` and `Template` (unfortunate name clash). The creator of a resource is its `owner`, who is permitted to carry out any action on the resource. Other users are granted _direct_ access to a resource by being added to an ACL as one of two roles, _viewer_ or _editor_. A `User` can also be granted access to a `Template` via a `Market` the user is a member of, again as either a viewer or editor. In other words, a `Template` has viewer and editor users, directly, but also viewer and editor markets, which grant access to the users within them. The permissions gained by being a viewer/editor depend on whether the `User` in question has an internal or external job.

There are some rules limiting how access can be shared. Only _distributor_ `User`s can share with _customer_ `User`s, and in particular those with their customer ID. And only _internal_ `User`s can be granted editor access to resources. 

## Approaches

The `static/` directory contains policies and a schema for the _viewer_ and _editor_ relations on resources, both for presentations and templates, as `Set`-typed attributes `viewers` and `editors` on the resources.

The `templated/` directory uses Cedar templates instead. We drop the `viewers` and `editors` attributes and follow a simple pattern: Whenever you would add a `User` to _resource_`.viewers`, instead link a template with `?principal` as the user and `?resource` as the viewer. Do likewise for editors. And, do similarly with viewer/editor status of `Market`s on ABC (not Cedar) `Template` resources.

These are the only differences in the approaches.

Expand the approach below for more details.

<details>

<summary>Static</summary>

### Entities

#### `Job`
An attribute that defines the user's job.

#### `User`
Represents a user that has access to resources.

#### `Market`
A way of grouping users.

#### `Presentation` 
Represents a sales presentation resource that is owned by the `User` who created it.

#### `Template`
Represents a presentation template resource. A `User` gets access to a `Template` by being a member of the `Market` the `Template` was created in.

### Actions
There are unique actions for both presentations and templates.

<details>

<summary>Presentations</summary>

#### `InternalPrezViewActions`
An action group that includes the following actions:
* `viewPresentation`
* `removeSelfAccessFromPresentation`
* `duplicatePresentation`

#### `ExternalPrezViewActions`
An action group that includes the following actions:
* `viewPresentation`
* `removeSelfAccessFromPresentation`

#### `PrezEditActions`
An action group that includes the following actions:
* `viewPresentation`
* `removeSelfAccessFromPresentation`
* `duplicatePresentation`
* `editPresentation`
* `grantViewAccessToPresentation`
* `grantEditAccessToPresentation`

</details>

<details>

<summary>Templates</summary>

#### `InternalTemplateViewActions`
An action group that includes the following actions:
* `viewTemplate`
* `duplicateTemplate`
* `removeSelfAccessFromTemplate`

#### `MarketTemplateViewActions`
An action group that includes the following actions:
* `viewTemplate`
* `duplicateTemplate`

#### `TemplateEditActions`
An action group that includes the following actions:
* `viewTemplate`
* `duplicateTemplate`
* `removeSelfAccessFromTemplate`
* `editTemplate`
* `removeOthersAccessToTemplate`
* `grantViewAccessToTemplate`
* `grantEditAccessToTemplate`

</details>

### Context 
Presentations and templates have unique context.

<details>

<summary>Presentations</summary>

#### `target`
A user that is the target of the action. For example, getting view or edit access to a presentation.

</details>

<details>

<summary>Templates</summary>

The template context helps grant view or edit access to a template based on who the user is and what market they are in.

#### `targetMarket?`
A market the template and user are in.

#### `targetUser?`
A user that is the target of the action.

</details>

### Schema

#### Entity types
* `Job`
* `User`:
  * memberOfTypes: `Market`
  * Attributes:
    * `job`: a `Job`
	* `customerId`: a String
* `Market`
* `Presentation`:
  * Attributes
    * `owner`: a `User`
	* `viewers`: a set of `User` entities	
	* `editors`: a set of `User` entities
* `Template`:
  * Attributes
    * `owner`: a `User`
	* `viewers`: a set of `User` entities	
	* `editors`: a set of `User` entities
	* `viewerMarkets`: a set of `Market` entities
	* `editorMarkets`: a set of `Market` entities
	
#### Action types
<details>

<summary>Presentations</summary>

* `InternalPrezViewActions`
* `ExternalPrezViewActions`
* `PrezEditActions`
* `viewPresentation`, `removeSelfAccessFromPresentation`
  * `memberOf`: `InternalPrezViewActions`, `ExternalPrezViewActions`, `PrezEditActions`
  * principals: `User`
  * resources: `Presentation`
* `duplicatePresentation`,
  * `memberOf`: `InternalPrezViewActions`, `PrezEditActions`
  * principals: `User`
  * resources: `Presentation`
* `editPresentation`,
  * `memberOf`: `PrezEditActions`
  * principals: `User`
  * resources: `Presentation`
* `grantViewAccessToPresentation`, `grantEditAccessToPresentation`
  * `memberOf`: `PrezEditActions`
  * principals: `User`
  * resources: `Presentation`
  * context: `target`

</details>

<details>

<summary>Templates</summary>

* `InternalTemplateViewActions`
* `MarketTemplateViewActions`
* `TemplateEditActions`
* `viewTemplate`, `duplicateTemplate`
  * `memberOf`: `InternalTemplateViewActions`, `MarketTemplateViewActions`, `TemplateEditActions`
  * principals: `User`
  * resources: `Template`
* `removeSelfAccessFromTemplate`,
  * `memberOf`: `InternalTemplateViewActions`, `TemplateEditActions`
  * principals: `User`
  * resources: `Template`
* `editTemplate`, `removeOthersAccessToTemplate`
  * `memberOf`: `TemplateEditActions`
  * principals: `User`
  * resources: `Template`
* `grantViewAccessToTemplate`, `grantEditAccessToTemplate`
  * `memberOf`: `TemplateEditActions`
  * principals: `User`
  * resources: `Template`
  * context: `targetMarket?`, `targetUser?`

</details>

### Policies

<details>

<summary>Presentations</summary>

```
@id("external-prez-view")
permit(
  principal,
  action in Action::"ExternalPrezViewActions",
  resource)
when {
  principal in resource.viewers
};

@id("internal-prez-view")
permit(
  principal,
  action in Action::"InternalPrezViewActions",
  resource)
when {
  principal.job == Job::"internal" && 
  principal in resource.viewers
};

// Authorizes edit actions generally, but these limited with forbid policies
@id("prez-edit")
permit(
  principal,
  action in Action::"PrezEditActions",
  resource)
when {
  resource.owner == principal || 
  principal in resource.editors
};

// only permit sharing to non-customers
@id("limit-prez-view-customer")
forbid(
  principal,
  action == Action::"grantViewAccessToPresentation",
  resource)
unless {
  context.target.job != Job::"customer" ||
  (principal.job == Job::"distributor" &&
   principal.customerId == context.target.customerId)
};

// forbid sharing editor access to non-internal users
@id("limit-prez-edit-to-internal")
forbid(
  principal,
  action == Action::"grantEditAccessToPresentation",
  resource)
when {
  context.target.job != Job::"internal"
};
```

</details>

<details>

<summary>Templates</summary>

```
@id("market-template-view")
permit(
  principal,
  action in Action::"MarketTemplateViewActions",
  resource)
when {
  principal in resource.viewerMarkets
};

@id("internal-template-view")
permit(
  principal,
  action in Action::"InternalTemplateViewActions",
  resource)
when {
  principal.job == Job::"internal" && principal in resource.viewers
};

// Authorizes edit actions generally, but these limited with forbid policies
@id("template-edit")
permit(
  principal,
  action in Action::"TemplateEditActions",
  resource)
when {
  resource.owner == principal || 
  principal in resource.editors ||
  principal in resource.editorMarkets
};

// only permit sharing by internal users to non-customers
@id("limit-template-grant-view")
forbid(
  principal,
  action == Action::"grantViewAccessToTemplate",
  resource)
when {
  context has targetUser && context.targetUser.job == Job::"customer" &&
  (principal.job != Job::"distributor" ||
   principal.customerId != context.targetUser.customerId)
};

// forbid sharing editor access to non-internal users
@id("limit-template-grant-edit-internal")
forbid(
  principal,
  action == Action::"grantEditAccessToTemplate",
  resource)
when {
   context has targetUser && context.targetUser.job != Job::"internal"
   // context.targetMarket always Ok, no matter the market
};
```

</details>

</details>

<details>

<summary>Templated</summary>

### Entities

#### `Job`
An attribute that defines the user's job.

#### `User`
Represents a user that has access to resources.

#### `Market`
A way of grouping users.

#### `Presentation` 
Represents a sales presentation resource that is owned by the `User` who created it.

#### `Template`
Represents a presentation template resource. A `User` gets access to a `Template` by being a member of the `Market` the `Template` was created in.

### Actions

There are unique actions for both presentations and templates.

<details>

<summary>Presentations</summary>

#### `InternalPrezViewActions`
An action group that includes the following actions:
* `viewPresentation`
* `removeSelfAccessFromPresentation`
* `duplicatePresentation`

#### `ExternalPrezViewActions`
An action group that includes the following actions:
* `viewPresentation`
* `removeSelfAccessFromPresentation`

#### `PrezEditActions`
An action group that includes the following actions:
* `viewPresentation`
* `removeSelfAccessFromPresentation`
* `duplicatePresentation`
* `editPresentation`
* `grantViewAccessToPresentation`
* `grantEditAccessToPresentation`

</details>

<details>

<summary>Templates</summary>

#### `InternalTemplateViewActions`
An action group that includes the following actions:
* `viewTemplate`
* `duplicateTemplate`
* `removeSelfAccessFromTemplate`

#### `MarketTemplateViewActions`
An action group that includes the following actions:
* `viewTemplate`
* `duplicateTemplate`

#### `TemplateEditActions`
An action group that includes the following actions:
* `viewTemplate`
* `duplicateTemplate`
* `removeSelfAccessFromTemplate`
* `editTemplate`
* `removeOthersAccessToTemplate`
* `grantViewAccessToTemplate`
* `grantEditAccessToTemplate`

</details>

### Context 
Presentations and templates have unique context.

<details>

<summary>Presentations</summary>

#### `target`
A user that is the target of the action. For example, getting view or edit access to a presentation.

</details>

<details>

<summary>Templates</summary>

The template context helps grant view or edit access to a template based on who the user is and what market they are in.

#### `targetMarket?`
A market the template and user are in.

#### `targetUser?`
A user that is the target of the action.

</details>

### Schema

#### Entity types
* `Job`
* `User`:
  * memberOfTypes: `Market`
  * Attributes:
    * `job`: a `Job`
	* `customerId`: a String
* `Market`
* `Presentation`:
  * Attributes
    * `owner`: a `User`
* `Template`:
  * Attributes
    * `owner`: a `User`

#### Action types
<details>

<summary>Presentations</summary>

* `InternalPrezViewActions`
* `ExternalPrezViewActions`
* `PrezEditActions`
* `viewPresentation`, `removeSelfAccessFromPresentation`
  * `memberOf`: `InternalPrezViewActions`, `ExternalPrezViewActions`, `PrezEditActions`
  * principals: `User`
  * resources: `Presentation`
* `duplicatePresentation`,
  * `memberOf`: `InternalPrezViewActions`, `PrezEditActions`
  * principals: `User`
  * resources: `Presentation`
* `editPresentation`,
  * `memberOf`: `PrezEditActions`
  * principals: `User`
  * resources: `Presentation`
* `grantViewAccessToPresentation`, `grantEditAccessToPresentation`
  * `memberOf`: `PrezEditActions`
  * principals: `User`
  * resources: `Presentation`
  * context: `target`

</details>

<details>

<summary>Templates</summary>

* `InternalTemplateViewActions`
* `MarketTemplateViewActions`
* `TemplateEditActions`
* `viewTemplate`, `duplicateTemplate`
  * `memberOf`: `InternalTemplateViewActions`, `MarketTemplateViewActions`, `TemplateEditActions`
  * principals: `User`
  * resources: `Template`
* `removeSelfAccessFromTemplate`,
  * `memberOf`: `InternalTemplateViewActions`, `TemplateEditActions`
  * principals: `User`
  * resources: `Template`
* `editTemplate`, `removeOthersAccessToTemplate`
  * `memberOf`: `TemplateEditActions`
  * principals: `User`
  * resources: `Template`
* `grantViewAccessToTemplate`, `grantEditAccessToTemplate`
  * `memberOf`: `TemplateEditActions`
  * principals: `User`
  * resources: `Template`
  * context: `targetMarket?`, `targetUser?`

</details>

### Policies

<details>

<summary>Presentations</summary>

```
// Here, ?principal is a group of users allowed to view ?resource, i.e., resource.viewers above
@createPolicyWhen("Create a template linked policy
                   when a external user is added to a prez as viewer")
@id("external-prez-view")
permit(principal == ?principal,
  action in Action::"ExternalPrezViewActions",
  resource == ?resource)
when {
  principal.job != Job::"internal"
};

// Here, ?principal is a group of users allowed to view ?resource
@createPolicyWhen("Create a template linked policy
                   when a internal user is added to a prez as viewer")
@id("internal-prez-view")
permit(principal == ?principal,
  action in Action::"InternalPrezViewActions",
  resource == ?resource)
when {
  principal.job == Job::"internal"
};

// Here, ?principal is a group of users allowed to edit ?resource, i.e., resource.editors above
@createPolicyWhen("Create a template linked policy
                   when a user is added to a prez as editor")
@id("template-edit for non-owner")                   
permit(
  principal == ?principal,
  action in Action::"PrezEditActions",
  resource == ?resource);

// Presentation owners always allowed to do what they want
@id("template-edit for owner")
permit(
  principal,
  action in Action::"PrezEditActions",
  resource)
when {
  resource.owner == principal
};

// only permit sharing to non-customers
@id("limit-prez-view-customer")
forbid(
  principal,
  action == Action::"grantViewAccessToPresentation",
  resource)
unless {
  context.target.job != Job::"customer" ||
  (principal.job == Job::"distributor" &&
   principal.customerId == context.target.customerId)
};

// forbid sharing editor access to non-internal users
@id("limit-prez-edit-to-internal")
forbid(
  principal,
  action == Action::"grantEditAccessToPresentation",
  resource)
when {
  context.target.job != Job::"internal"
};
```

</details>

<details>

<summary>Templates</summary>

```
@createPolicyWhen("Create a template linked policy
                   when a user is added to a template as market viewer")
@id("market-template-view")
permit(
  principal == ?principal,
  action in Action::"MarketTemplateViewActions",
  resource == ?resource)
when {
  principal.job != Job::"internal"
};


@createPolicyWhen("Create a template linked policy
                   when a internal user is added to a template as viewer")
@id("internal-template-view")
permit(
  principal == ?principal,
  action in Action::"InternalTemplateViewActions",
  resource == ?resource)
when {
  principal.job == Job::"internal"
};

// Authorizes edit actions generally, but these limited with forbid policies
@id("template-edit")
@createPolicyWhen("Create a template linked policy
                   when a user is added to a template as editor")
permit(
  principal == ?principal,
  action in Action::"TemplateEditActions",
  resource == ?resource);

// Permit owners to edit templates 
permit(
  principal,
  action in Action::"TemplateEditActions",
  resource)
when {
  principal == resource.owner 
};

// only permit sharing by internal users to non-customers
@id("limit-template-grant-view")
forbid(
  principal,
  action == Action::"grantViewAccessToTemplate",
  resource)
when {
  context has targetUser && context.targetUser.job == Job::"customer" &&
  (principal.job != Job::"distributor" ||
   principal.customerId != context.targetUser.customerId)
};

// forbid sharing editor access to non-internal users
@id("limit-template-grant-edit-internal")
forbid(
  principal,
  action == Action::"grantEditAccessToTemplate",
  resource)
when {
   context has targetUser && context.targetUser.job != Job::"internal"
   // context.targetMarket always Ok, no matter the market
};
```
</details>

</details>

## Tests

We use the following entities for our tests:
* There are 3 `User` entities, `User::"Alice"`, `User::"Bob"`, or `User::"Charlie"`.
* There is one `Presentation` entity, `Presentation::"proposal"`.

Our three users have the following permissions (expressed in the entities in the static policies, and as links in the templated ones):
* Alice is the owner of the presentation and therefore has full access to the presentation.
* Bob has view permissions to the presentation.
* Charlie has no permissions to the presentation.

Here are some authz requests for these:
* Alice views the presentation: ALLOW, since Alice owns the presentation.
* Bob views the presentation: ALLOW, since Bob is an allowed viewer of the presentation. In the `static/` policies this fact is expressed in the `entities.json` file as part of the `Presentation::"proposal"` entity. In the `templated\` policies this fact is expressed via template links, expressed in the `linked` file.
* Charlie views the presentarion, DENY, since Charlie doesn't have view or edit permissions for the presentation.
